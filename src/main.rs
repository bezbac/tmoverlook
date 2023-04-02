use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::{info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Apply {
        #[arg(short, long, value_name = "FILE")]
        config: Option<String>,

        #[arg(short, long)]
        preview: bool,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct PathRule {
    path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Rule {
    Path(PathRule),
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    rules: Vec<Rule>,
}

const DEFAULT_CONFIG_PATH: &str = "~/.config/tmoverlook/config.toml";

fn read_config(path: Option<&str>) -> Result<Config> {
    let config_path = path.unwrap_or(DEFAULT_CONFIG_PATH);
    let input = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&input)?;
    Ok(config)
}

fn evaluate_rule(rule: &Rule) -> Result<Vec<PathBuf>> {
    let mut paths = vec![];

    match rule {
        Rule::Path(PathRule { path }) => {
            paths.push(fs::canonicalize(shellexpand::tilde(&path).to_string())?)
        }
    }

    Ok(paths)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter(None, LevelFilter::Info)
        .init();

    match &cli.command {
        Some(Commands::Apply { preview, config }) => {
            if *preview {
                info!("Preview mode is active, no changes will be applied",)
            }

            let config = read_config(config.as_deref());

            let mut paths: Vec<PathBuf> = vec![];

            for rule in config?.rules {
                for path in evaluate_rule(&rule)? {
                    paths.push(path)
                }
            }

            for path in &paths {
                if let Some(str_path) = path.to_str() {
                    if !*preview {
                        let output = Command::new("tmutil")
                            .arg("addexclusion")
                            .arg(str_path)
                            .output()
                            .expect("failed to execute process");

                        assert!(output.status.success());
                    }

                    info!("Ignoring {}", path.to_string_lossy())
                }
            }
        }
        None => {}
    }

    Ok(())
}
