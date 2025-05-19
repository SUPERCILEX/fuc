use std::{
    cell::LazyCell,
    fs,
    mem::swap,
    path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, PathBuf},
};

use clap::{ArgAction, Parser, ValueHint};
use error_stack::Report;
use fuc_engine::{CopyOp, Error};

/// A zippy alternative to `cp`, a tool to copy files and directories
#[derive(Parser, Debug)]
#[command(version, author = "Alex Saveau (@SUPERCILEX)")]
#[command(infer_subcommands = true, infer_long_args = true)]
#[command(disable_help_flag = true)]
#[command(arg_required_else_help = true)]
#[command(max_term_width = 100)]
#[cfg_attr(test, command(help_expected = true))]
struct Cpz {
    /// The file(s) or directory(ies) to be copied
    ///
    /// If multiple files are specified, they will be copied into the target
    /// destination rather than to it. The same is true of directory names
    /// (`foo/`, `.`, `..`): that is, `cpz a b/` places `a` inside `b` as
    /// opposed to `cpz a b` which makes `b` become `a`.
    #[arg(required = true)]
    #[arg(value_hint = ValueHint::AnyPath)]
    from: Vec<PathBuf>,

    /// The copy destination
    #[arg(required = true)]
    #[arg(value_hint = ValueHint::AnyPath)]
    to: PathBuf,

    /// Overwrite existing files
    #[arg(short, long, default_value_t = false)]
    force: bool,

    /// Reverse the argument order so that it becomes `cpz <TO> <FROM>...`
    #[arg(short = 't', long, default_value_t = false)]
    reverse_args: bool,

    /// Follow symlinks in the files to be copied rather than copying the
    /// symlinks themselves
    #[arg(short = 'L', long, default_value_t = false)]
    #[arg(aliases = ["follow-symlinks"])]
    // Ensure we don't try to create symlinks by default as doing so is considered a privileged
    // operation: https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_file.html#limitations
    #[cfg_attr(windows, arg(default_value_t = true))]
    dereference: bool,

    /// Create hard links instead of copying file data
    #[arg(short = 'l', long, default_value_t = false)]
    #[arg(aliases = ["hard-link"])]
    link: bool,

    #[arg(short, long, short_alias = '?', global = true)]
    #[arg(action = ArgAction::Help, help = "Print help (use `--help` for more detail)")]
    #[arg(long_help = "Print help (use `-h` for a summary)")]
    help: Option<bool>,
}

#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error("{0}")]
    Wrapper(String),
}

#[cfg(feature = "trace")]
#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
    tracy_client::ProfiledAllocator::new(std::alloc::System, 100);

#[cfg(feature = "trace")]
fn init_trace() {
    use tracing_subscriber::{
        EnvFilter, filter::LevelFilter, fmt::format::DefaultFields, layer::SubscriberExt,
        util::SubscriberInitExt,
    };

    #[derive(Default)]
    struct Config(DefaultFields);

    impl tracing_tracy::Config for Config {
        type Formatter = DefaultFields;

        fn formatter(&self) -> &Self::Formatter {
            &self.0
        }

        fn stack_depth(&self, _: &tracing::Metadata<'_>) -> u16 {
            32
        }

        fn format_fields_in_zone_name(&self) -> bool {
            false
        }
    }

    tracing_subscriber::registry()
        .with(tracing_tracy::TracyLayer::new(Config::default()))
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .from_env_lossy(),
        )
        .init();
}

#[cfg(feature = "progress")]
fn init_progress() {
    use std::time::Duration;

    use indicatif::{ProgressState, ProgressStyle};
    use tracing::level_filters::LevelFilter;
    use tracing_indicatif::IndicatifLayer;
    use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    let indicatif_layer = IndicatifLayer::new()
        .with_progress_style(
            ProgressStyle::with_template(
                "{color_start}{span_child_prefix}{span_fields} -- {span_name} {wide_msg} \
                 {elapsed_subsec}{color_end}",
            )
            .unwrap()
            .with_key(
                "elapsed_subsec",
                |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                    let seconds = state.elapsed().as_secs();
                    let sub_seconds = (state.elapsed().as_millis() % 1000) / 100;
                    let _ = write!(writer, "{}.{}s", seconds, sub_seconds);
                },
            )
            .with_key(
                "color_start",
                |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                    let elapsed = state.elapsed();

                    if elapsed > Duration::from_secs(8) {
                        // Red
                        let _ = write!(writer, "\x1b[{}m", 1 + 30);
                    } else if elapsed > Duration::from_secs(4) {
                        // Yellow
                        let _ = write!(writer, "\x1b[{}m", 3 + 30);
                    }
                },
            )
            .with_key(
                "color_end",
                |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                    if state.elapsed() > Duration::from_secs(4) {
                        let _ = write!(writer, "\x1b[0m");
                    }
                },
            ),
        )
        .with_span_child_prefix_symbol("↳ ")
        .with_span_child_prefix_indent(" ");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}

fn main() -> error_stack::Result<(), CliError> {
    #[cfg(not(debug_assertions))]
    error_stack::Report::install_debug_hook::<std::panic::Location>(|_, _| {});

    #[cfg(feature = "trace")]
    init_trace();
    #[cfg(feature = "progress")]
    init_progress();

    let args = Cpz::parse();

    copy(args).map_err(|e| {
        let wrapper = CliError::Wrapper(format!("{e}"));
        match e {
            Error::Io { error, context } => Report::from(error)
                .attach_printable(context)
                .change_context(wrapper),
            Error::AlreadyExists { file } => {
                let report = Report::from(wrapper);
                match file.symlink_metadata().map(|m| m.is_dir()) {
                    Ok(true) => {
                        let mut file = file.into_os_string();
                        file.push(MAIN_SEPARATOR_STR);
                        report
                            .attach_printable(format!(
                                "Use the path {file:?} to copy into the directory."
                            ))
                            .attach_printable(
                                "Use --force to merge directories (overwriting existing files).",
                            )
                    }
                    Ok(false) | Err(_) => report.attach_printable("Use --force to overwrite."),
                }
            }
            Error::Join | Error::BadPath | Error::Internal => Report::from(wrapper),
            Error::PreserveRoot | Error::NotFound { file: _ } => unreachable!(),
        }
    })
}

fn copy(
    Cpz {
        mut from,
        mut to,
        force,
        reverse_args,
        dereference,
        link,
        help: _,
    }: Cpz,
) -> Result<(), Error> {
    if reverse_args {
        swap(&mut to, &mut from[0]);
    }
    let from = from;
    let to = to;

    #[allow(clippy::unnested_or_patterns)]
    let is_into_directory = LazyCell::new(|| {
        matches!(
            {
                let path_str = to.to_string_lossy();
                let mut chars = path_str.chars();
                (chars.next_back(), chars.next_back(), chars.next_back())
            },
            (Some(MAIN_SEPARATOR), _, _) // */
                | (Some('.'), None, _) // .
                | (Some('.'), Some(MAIN_SEPARATOR), _) // */.
                | (Some('.'), Some('.'), None) // ..
                | (Some('.'), Some('.'), Some(MAIN_SEPARATOR)) // */..
        )
    });
    if let Some(dir) = if from.len() > 1 || (*is_into_directory && from[0].file_name().is_some()) {
        Some(&*to)
    } else {
        to.parent()
    } {
        fs::create_dir_all(dir).map_err(|error| Error::Io {
            error,
            context: format!("Failed to create directory {to:?}").into(),
        })?;
    }

    macro_rules! run_with_files {
        ($files:expr) => {
            CopyOp::builder()
                .files($files)
                .force(force)
                .follow_symlinks(dereference)
                .hard_link(link)
                .build()
                .run()
        };
    }
    if from.len() > 1 {
        run_with_files!(from.into_iter().map(|path| {
            let to = path
                .file_name()
                .map_or_else(|| to.clone(), |name| to.join(name));
            (path, to)
        }))
    } else {
        run_with_files!([{
            let from = from.into_iter().next().unwrap();
            let to = {
                let is_into_directory = *is_into_directory;
                let mut to = to;
                if is_into_directory && let Some(name) = from.file_name() {
                    to.push(name);
                }
                to
            };

            (from, to)
        }])
    }
}

#[cfg(test)]
mod cli_tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_app() {
        Cpz::command().debug_assert();
    }

    #[test]
    fn help_for_review() {
        supercilex_tests::help_for_review(Cpz::command());
    }
}
