use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueHint};
use clap_verbosity_flag::Verbosity;

use fuc_engine::{FsOp, RemoveOp};

/// A zippy alternative to rm
#[derive(Parser, Debug)]
#[clap(version, author = "Alex Saveau (@SUPERCILEX)")]
#[clap(infer_subcommands = true, infer_long_args = true)]
#[clap(next_display_order = None)]
#[clap(max_term_width = 100)]
#[command(disable_help_flag = true)]
#[cfg_attr(test, clap(help_expected = true))]
struct Rmz {
    /// The files to be removed
    #[clap(required = true)]
    #[clap(value_hint = ValueHint::DirPath)]
    files: Vec<PathBuf>,
    #[clap(flatten)]
    verbose: Verbosity,
    #[arg(short, long, short_alias = '?', global = true)]
    #[arg(action = ArgAction::Help, help = "Print help information (use `--help` for more detail)")]
    #[arg(long_help = "Print help information (use `-h` for a summary)")]
    help: Option<bool>,
}

fn main() {
    let args = Rmz::parse();

    RemoveOp::builder()
        .files(args.files.iter().map(AsRef::as_ref))
        .build()
        .run()
        .unwrap();
}

// #[cfg(test)]
// mod tests {
//     use clap::{ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand, IntoApp};
//
//     use super::*;
//
//     #[test]
//     fn verify_app() {
//         Rmz::into_app().debug_assert()
//     }
//
//     #[test]
//     fn empty_args_displays_help() {
//         let f = Rmz::try_parse_from(Vec::<String>::new());
//
//         assert!(f.is_err());
//         assert_eq!(
//             f.unwrap_err().kind,
//             DisplayHelpOnMissingArgumentOrSubcommand
//         )
//     }
// }
