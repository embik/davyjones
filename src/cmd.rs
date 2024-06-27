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
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .value_parser(value_parser!(u16))
                .default_value("8080")
                .help("Port to listen on when starting webhook server"),
        )
        .arg(
            Arg::new("host")
                .long("host")
                .default_value("localhost")
                .help("Host to listen on when starting webhook server"),
        )
}
