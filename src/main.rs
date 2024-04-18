use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use env_logger::Builder;
use environment::{OVERWRITTEN_HOME_DIR, OVERWRITTEN_PREFIX_DIR};
use log::LevelFilter;
use std::io::Write;

mod cache;
mod commands;
mod config;
mod contants;
mod environment;
mod rules;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long)]
    debug: bool,

    #[arg(long, hide = true)]
    prefix_dir: Option<String>,

    #[arg(long, hide = true)]
    home_dir: Option<String>,
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

    if cli.prefix_dir.is_some() {
        OVERWRITTEN_PREFIX_DIR.set(cli.prefix_dir).map_err(|val| {
            anyhow!(
                "Could not overwrite prefix directory. Already set to {:?}",
                val
            )
        })?;
    }

    if cli.home_dir.is_some() {
        OVERWRITTEN_HOME_DIR.set(cli.home_dir).map_err(|val| {
            anyhow!(
                "Could not overwrite home directory. Already set to {:?}",
                val
            )
        })?;
    }

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
