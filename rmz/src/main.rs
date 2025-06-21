use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueHint};
use error_stack::Report;
use fuc_engine::{Error, RemoveOp};

/// A zippy alternative to `rm`, a tool to remove files and directories
#[derive(Parser, Debug)]
#[command(version, author = "Alex Saveau (@SUPERCILEX)")]
#[command(infer_subcommands = true, infer_long_args = true)]
#[command(disable_help_flag = true)]
#[command(arg_required_else_help = true)]
#[command(max_term_width = 100)]
#[cfg_attr(test, command(help_expected = true))]
struct Rmz {
    /// The files and/or directories to be removed
    #[arg(required = true)]
    #[arg(value_hint = ValueHint::AnyPath)]
    files: Vec<PathBuf>,

    /// Ignore non-existent arguments
    #[arg(short, long, default_value_t = false)]
    force: bool,

    /// Allow deletion of `/`
    #[arg(long = "no-preserve-root", default_value_t = true)]
    #[arg(action = ArgAction::SetFalse)]
    preserve_root: bool,

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

fn main() -> Result<(), Report<CliError>> {
    #[cfg(not(debug_assertions))]
    error_stack::Report::install_debug_hook::<std::panic::Location>(|_, _| {});

    #[cfg(feature = "trace")]
    init_trace();
    #[cfg(feature = "progress")]
    init_progress();

    let args = Rmz::parse();

    remove(args).map_err(|e| {
        let wrapper = CliError::Wrapper(format!("{e}"));
        match e {
            Error::Io { error, context } => {
                Report::from(error).attach(context).change_context(wrapper)
            }
            Error::NotFound { file: _ } => Report::from(wrapper).attach("Use --force to ignore."),
            Error::PreserveRoot | Error::BadPath => Report::from(wrapper),
            Error::Join(e) => Report::from(e).change_context(wrapper),
            Error::AlreadyExists { file: _ } => unreachable!(),
        }
    })
}

fn remove(
    Rmz {
        files,
        force,
        preserve_root,
        help: _,
    }: Rmz,
) -> Result<(), Error> {
    RemoveOp::builder()
        .files(files.into_iter())
        .force(force)
        .preserve_root(preserve_root)
        .build()
        .run()
}

#[cfg(test)]
mod cli_tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_app() {
        Rmz::command().debug_assert();
    }

    #[test]
    fn help_for_review() {
        supercilex_tests::help_for_review(Rmz::command());
    }
}
