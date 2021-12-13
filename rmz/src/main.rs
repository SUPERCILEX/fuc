use std::{path::PathBuf, process::exit};

use clap::{AppSettings, Parser};

use fuc_core::errors::CliResult;

/// A zippy alternative to rm
#[derive(Parser, Debug)]
#[clap(version, author = "Alex Saveau (@SUPERCILEX)")]
#[clap(global_setting(AppSettings::InferSubcommands))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
#[cfg_attr(test, clap(global_setting(AppSettings::HelpExpected)))]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Rmz {
    // #[clap(flatten)]
    // verbose: Verbosity,
    /// The files to be removed
    #[clap(required = true)]
    files: Vec<PathBuf>,
}

fn main() {
    if let Err(e) = wrapped_main() {
        if let Some(source) = e.source {
            eprintln!("{:?}", source);
        }
        exit(e.code);
    }
}

fn wrapped_main() -> CliResult<()> {
    let args = Rmz::parse();
    // SimpleLogger::new()
    //     .with_level(args.verbose.log_level().unwrap().to_level_filter())
    //     .init()
    //     .unwrap();

    todo!()
}

#[cfg(test)]
mod tests {
    use clap::{ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand, IntoApp};

    use super::*;

    #[test]
    fn verify_app() {
        Rmz::into_app().debug_assert()
    }

    #[test]
    fn empty_args_displays_help() {
        let f = Rmz::try_parse_from(Vec::<String>::new());

        assert!(f.is_err());
        assert_eq!(
            f.unwrap_err().kind,
            DisplayHelpOnMissingArgumentOrSubcommand
        )
    }
}
