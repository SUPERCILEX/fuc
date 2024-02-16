#![feature(lazy_cell)]
#![feature(let_chains)]

use std::{
    cell::LazyCell,
    fs,
    mem::swap,
    path::{PathBuf, MAIN_SEPARATOR, MAIN_SEPARATOR_STR},
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

fn main() -> error_stack::Result<(), CliError> {
    #[cfg(not(debug_assertions))]
    error_stack::Report::install_debug_hook::<std::panic::Location>(|_, _| {});

    #[cfg(feature = "trace")]
    {
        use tracing_subscriber::{
            fmt::format::DefaultFields, layer::SubscriberExt, util::SubscriberInitExt,
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
            .init();
    };

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
    if from.len() > 1 || *is_into_directory {
        fs::create_dir_all(&to).map_err(|error| Error::Io {
            error,
            context: format!("Failed to create directory {to:?}").into(),
        })?;
    }

    if from.len() > 1 {
        CopyOp::builder()
            .files(from.into_iter().map(|path| {
                let to = path
                    .file_name()
                    .map_or_else(|| to.clone(), |name| to.join(name));
                (path, to)
            }))
            .force(force)
            .build()
            .run()
    } else {
        CopyOp::builder()
            .files([{
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
            .force(force)
            .build()
            .run()
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
