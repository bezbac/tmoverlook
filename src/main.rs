use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

mod cache;
mod commands;
mod config;
mod contants;
mod rules;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List {},
    Apply {
        #[arg(short, long, value_name = "FILE")]
        config: Option<String>,

        #[arg(short, long)]
        preview: bool,

        #[arg(short, long)]
        yes: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    Builder::new()
        .format(move |buf, record| {
            if cli.debug && record.level() == LevelFilter::Debug {
                writeln!(buf, "Debug: {}", record.args())
            } else {
                writeln!(buf, "{}", record.args())
            }
        })
        .filter(None, log_level)
        .init();

    match &cli.command {
        Some(Commands::List {}) => commands::list::run(),
        Some(Commands::Apply {
            config: _,
            preview: _,
            yes: _,
        }) => commands::apply::run(&cli.command.unwrap()),
        None => Ok(()),
    }
}
