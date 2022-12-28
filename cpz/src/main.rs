#![allow(clippy::multiple_crate_versions)]

use std::{borrow::Cow, path::PathBuf};

use clap::{ArgAction, Parser, ValueHint};
use error_stack::{Report, Result};
use fuc_engine::{CopyOp, Error};

/// A zippy alternative to `cp`, a tool to copy files and directories
#[derive(Parser, Debug)]
#[command(version, author = "Alex Saveau (@SUPERCILEX)")]
#[command(infer_subcommands = true, infer_long_args = true)]
#[command(max_term_width = 100)]
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
    #[arg(action = ArgAction::Help, help = "Print help information (use `--help` for more detail)")]
    #[arg(long_help = "Print help information (use `-h` for a summary)")]
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
    use std::fmt::Write;

    use clap::{Command, CommandFactory};
    use expect_test::expect_file;

    use super::*;

    #[test]
    fn verify_app() {
        Cpz::command().debug_assert();
    }

    #[test]
    #[cfg_attr(miri, ignore)] // wrap_help breaks miri
    fn help_for_review() {
        let mut command = Cpz::command();

        command.build();

        let mut long = String::new();
        let mut short = String::new();

        write_help(&mut long, &mut command, LongOrShortHelp::Long);
        write_help(&mut short, &mut command, LongOrShortHelp::Short);

        expect_file!["../command-reference.golden"].assert_eq(&long);
        expect_file!["../command-reference-short.golden"].assert_eq(&short);
    }

    #[derive(Copy, Clone)]
    enum LongOrShortHelp {
        Long,
        Short,
    }

    fn write_help(buffer: &mut impl Write, cmd: &mut Command, long_or_short_help: LongOrShortHelp) {
        write!(
            buffer,
            "{}",
            match long_or_short_help {
                LongOrShortHelp::Long => cmd.render_long_help(),
                LongOrShortHelp::Short => cmd.render_help(),
            }
        )
        .unwrap();

        for sub in cmd.get_subcommands_mut() {
            writeln!(buffer).unwrap();
            writeln!(buffer, "---").unwrap();
            writeln!(buffer).unwrap();

            write_help(buffer, sub, long_or_short_help);
        }
    }
}
