#![feature(once_cell)]
#![feature(let_chains)]

use std::{
    borrow::Cow,
    cell::LazyCell,
    fs,
    path::{PathBuf, MAIN_SEPARATOR},
};

use clap::{ArgAction, Parser, ValueHint};
use clap2 as clap;
use error_stack::Report;
use fuc_engine::{CopyOp, Error};

/// A zippy alternative to `cp`, a tool to copy files and directories
#[derive(Parser, Debug)]
#[command(version, author = "Alex Saveau (@SUPERCILEX)")]
#[command(infer_subcommands = true, infer_long_args = true)]
#[command(disable_help_flag = true)]
#[command(arg_required_else_help = true)]
#[cfg_attr(test, command(help_expected = true))]
struct Cpz {
    /// The file(s) or directory(ies) to be copied
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
    let args = Cpz::parse();

    copy(args).map_err(|e| {
        let wrapper = CliError::Wrapper(format!("{e}"));
        match e {
            Error::Io { error, context } => Report::from(error)
                .attach_printable(context)
                .change_context(wrapper),
            Error::AlreadyExists { file: _ } => {
                Report::from(wrapper).attach_printable("Use --force to overwrite.")
            }
            Error::Join | Error::BadPath | Error::Internal => Report::from(wrapper),
            Error::PreserveRoot | Error::NotFound { file: _ } => unreachable!(),
        }
    })
}

fn copy(args: Cpz) -> Result<(), Error> {
    let is_into_directory = LazyCell::new(|| args.to.to_string_lossy().ends_with(MAIN_SEPARATOR));
    if args.from.len() > 1 || *is_into_directory {
        fs::create_dir_all(&args.to).map_err(|error| Error::Io {
            error,
            context: format!("Failed to create directory {:?}", args.to),
        })?;
    }

    if args.from.len() > 1 {
        CopyOp::builder()
            .files(args.from.into_iter().map(|path| {
                let to = path
                    .file_name()
                    .map_or_else(|| args.to.clone(), |name| args.to.join(name));
                (Cow::Owned(path), Cow::Owned(to))
            }))
            .force(args.force)
            .build()
            .run()
    } else {
        CopyOp::builder()
            .files([{
                let from = args.from.into_iter().next().unwrap();
                let to = {
                    let is_into_directory = *is_into_directory;
                    let mut to = args.to;
                    if is_into_directory && let Some(name) = from.file_name() {
                        to.push(name);
                    }
                    to
                };

                (Cow::Owned(from), Cow::Owned(to))
            }])
            .force(args.force)
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
        supercilex_tests::help_for_review2(Cpz::command());
    }
}
