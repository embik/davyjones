use clap::{value_parser, Arg, Command};
use std::path::PathBuf;

pub fn cli() -> Command {
    Command::new("davyjones")
        .arg_required_else_help(false)
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::SetTrue)
                .global(true)
                .help("Enable verbose (debug) logging"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .global(true)
                .value_parser(value_parser!(PathBuf))
                .help("Configuration file to start davyjones with"),
        )
}
