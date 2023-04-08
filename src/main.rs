use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use lazy_static::lazy_static;
use log::{info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

lazy_static! {
    static ref DEFAULT_CONFIG_PATH: String =
        shellexpand::tilde("~/.config/tmoverlook/config.toml").to_string();
    static ref DEFAULT_CACHE_PATH: String =
        shellexpand::tilde("~/.local/share/tmoverlook/cache.toml").to_string();
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    List {},
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

#[derive(Serialize, Deserialize, Debug)]
struct Cache {
    paths: HashSet<String>,
}

fn read_cache() -> Result<Cache> {
    let input = fs::read_to_string(DEFAULT_CACHE_PATH.as_str());
    let cache = match input {
        Err(_) => Cache {
            paths: HashSet::new(),
        },
        Ok(input) => toml::from_str(&input)?,
    };
    Ok(cache)
}

fn write_cache(cache: &Cache) -> Result<()> {
    let path = PathBuf::from(DEFAULT_CACHE_PATH.as_str());
    let dir = path.parent().unwrap();

    fs::create_dir_all(dir)?;

    let contents = toml::to_string(cache)?;
    fs::write(path, contents)?;
    Ok(())
}

fn read_config(path: Option<&str>) -> Result<Config> {
    let config_path = path.unwrap_or(&DEFAULT_CONFIG_PATH);
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

fn add_exclusion(path: &str) -> Result<()> {
    let output = Command::new("tmutil")
        .arg("addexclusion")
        .arg(path)
        .output()?;

    assert!(output.status.success());

    Ok(())
}

fn remove_exclusion(path: &str) -> Result<()> {
    let output = Command::new("tmutil")
        .arg("removeexclusion")
        .arg(path)
        .output()?;

    assert!(output.status.success());

    Ok(())
}

enum Diff {
    Unchanged,
    Added,
    Removed,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter(None, LevelFilter::Info)
        .init();

    match &cli.command {
        Some(Commands::List {}) => {
            let cache = read_cache()?;

            info!("Currenly ignored paths:");
            for path in cache.paths {
                info!("{}", path)
            }
        }
        Some(Commands::Apply { preview, config }) => {
            if *preview {
                info!("Preview mode is active, no changes will be applied",)
            }

            let config = read_config(config.as_deref());
            let cache = read_cache()?;

            let mut paths: HashSet<String> = HashSet::new();

            for rule in config?.rules {
                for path in evaluate_rule(&rule)? {
                    if let Some(path) = path.to_str() {
                        paths.insert(String::from(path));
                    }
                }
            }

            let combined_paths: HashSet<&String> = paths.iter().chain(cache.paths.iter()).collect();

            info!("Changes:");
            for path in combined_paths {
                let is_in_cache = cache.paths.contains(path);
                let is_in_new_list = paths.contains(path);

                let operation: Diff = if is_in_cache && is_in_new_list {
                    Diff::Unchanged
                } else if is_in_cache && !is_in_new_list {
                    Diff::Removed
                } else {
                    Diff::Added
                };

                let diff_char = match operation {
                    Diff::Unchanged => "·",
                    Diff::Added => "+",
                    Diff::Removed => "-",
                };

                info!("{} {}", diff_char, path);

                if !*preview {
                    match operation {
                        Diff::Unchanged | Diff::Added => add_exclusion(path)?,
                        Diff::Removed => remove_exclusion(path)?,
                    }
                }
            }

            if !preview {
                write_cache(&Cache { paths })?
            }
        }
        None => {}
    }

    Ok(())
}
