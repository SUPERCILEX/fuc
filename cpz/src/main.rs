#![allow(clippy::multiple_crate_versions)]

use std::{borrow::Cow, path::PathBuf};

use clap::{ArgAction, Parser, ValueHint};
use clap2 as clap;
use error_stack::{Report, Result};
use fuc_engine::{CopyOp, Error};

/// A zippy alternative to `cp`, a tool to copy files and directories
#[derive(Parser, Debug)]
#[command(version, author = "Alex Saveau (@SUPERCILEX)")]
#[command(infer_subcommands = true, infer_long_args = true)]
#[command(disable_help_flag = true)]
#[cfg_attr(test, command(help_expected = true))]
struct Cpz {
    /// The file or directory to be copied
    #[arg(required = true)]
    #[arg(value_hint = ValueHint::AnyPath)]
    from: PathBuf,

    /// The copy destination
    #[arg(required = true)]
    #[arg(value_hint = ValueHint::AnyPath)]
    to: PathBuf,

    /// Overwrite existing files
    #[arg(short, long, default_value_t = false)]
    force: bool,

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
    let args = Cpz::parse();

    CopyOp::builder()
        .files([(Cow::Owned(args.from), Cow::Owned(args.to))])
        .force(args.force)
        .build()
        .run()
        .map_err(|e| {
            let wrapper = CliError::Wrapper(format!("{e}"));
            match e {
                Error::Io { error, context } => Report::from(error)
                    .attach_printable(context)
                    .change_context(wrapper),
                Error::PreserveRoot
                | Error::Join
                | Error::BadPath
                | Error::AlreadyExists
                | Error::Internal => Report::from(wrapper),
            }
        })
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
        supercilex_tests::help_for_review2(Cpz::command());
    }
}
