#![allow(clippy::multiple_crate_versions)]

use std::{borrow::Cow, path::PathBuf};

use clap::{ArgAction, Parser, ValueHint};
use clap_verbosity_flag::Verbosity;
use error_stack::{Report, Result};
use fuc_engine::{Error, RemoveOp};

/// A zippy alternative to `rm`, a tool to remove files or directories
#[derive(Parser, Debug)]
#[clap(version, author = "Alex Saveau (@SUPERCILEX)")]
#[clap(infer_subcommands = true, infer_long_args = true)]
#[clap(max_term_width = 100)]
#[command(disable_help_flag = true)]
#[cfg_attr(test, clap(help_expected = true))]
struct Rmz {
    /// The files and/or directories to be removed
    #[clap(required = true)]
    #[clap(value_hint = ValueHint::AnyPath)]
    files: Vec<PathBuf>,
    /// Ignore non-existent arguments
    #[arg(short, long, default_value_t = false)]
    force: bool,
    /// Allow deletion of `/`
    #[arg(long = "no-preserve-root", default_value_t = true)]
    #[arg(action = ArgAction::SetFalse)]
    preserve_root: bool,
    #[clap(flatten)]
    verbose: Verbosity,
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
        Rmz::command().debug_assert();
    }

    #[test]
    #[cfg_attr(miri, ignore)] // wrap_help breaks miri
    fn help_for_review() {
        let mut command = Rmz::command();

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
