#![allow(clippy::multiple_crate_versions)]

use std::{borrow::Cow, path::PathBuf};

use clap::{ArgAction, Parser, ValueHint};
use clap2 as clap;
use error_stack::{Report, Result};
use fuc_engine::{Error, RemoveOp};

/// A zippy alternative to `rm`, a tool to remove files and directories
#[derive(Parser, Debug)]
#[command(version, author = "Alex Saveau (@SUPERCILEX)")]
#[command(infer_subcommands = true, infer_long_args = true)]
#[command(disable_help_flag = true)]
#[command(arg_required_else_help = true)]
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

fn main() -> Result<(), CliError> {
    let args = Rmz::parse();

    RemoveOp::builder()
        .files(args.files.into_iter().map(Cow::Owned))
        .force(args.force)
        .preserve_root(args.preserve_root)
        .build()
        .run()
        .map_err(|e| {
            let wrapper = CliError::Wrapper(format!("{e}"));
            match e {
                Error::Io { error, context } => Report::from(error)
                    .attach_printable(context)
                    .change_context(wrapper),
                Error::PreserveRoot | Error::Join | Error::BadPath | Error::Internal => {
                    Report::from(wrapper)
                }
                Error::AlreadyExists { file: _ } => unreachable!(),
            }
        })
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
