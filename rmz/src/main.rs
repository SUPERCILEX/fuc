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
pub enum CliError {
    #[error("{0}")]
    Wrapper(String),
}

fn main() -> error_stack::Result<(), CliError> {
    let args = Rmz::parse();

    remove(args).map_err(|e| {
        let wrapper = CliError::Wrapper(format!("{e}"));
        match e {
            Error::Io { error, context } => Report::from(error)
                .attach_printable(context)
                .change_context(wrapper),
            Error::NotFound { file: _ } => {
                Report::from(wrapper).attach_printable("Use --force to ignore.")
            }
            Error::PreserveRoot | Error::Join | Error::BadPath | Error::Internal => {
                Report::from(wrapper)
            }
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
        supercilex_tests::help_for_review2(Rmz::command());
    }
}
