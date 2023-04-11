use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use log::{info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::fs::{self, DirEntry};
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

trait Evaluatable {
    fn evaluate(&self) -> Result<Vec<PathBuf>>;
}

#[derive(Serialize, Deserialize, Debug)]
struct PathRule {
    path: String,
}

impl Evaluatable for PathRule {
    fn evaluate(&self) -> Result<Vec<PathBuf>> {
        Ok(vec![fs::canonicalize(
            shellexpand::tilde(&self.path).to_string(),
        )?])
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GitRepositoriesRule {
    search: Vec<String>,
}

impl Evaluatable for GitRepositoriesRule {
    fn evaluate(&self) -> Result<Vec<PathBuf>> {
        info!("Searching for git repositories");

        let spinner_style = ProgressStyle::with_template("{spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let pb = ProgressBar::new(0);
        pb.set_style(spinner_style);

        let mut paths: BTreeSet<PathBuf> = BTreeSet::new();

        fn walk(pb: &ProgressBar, found_directories: &mut BTreeSet<PathBuf>, dir: &PathBuf) {
            pb.inc(1);
            pb.set_message(format!("{}", dir.display()));

            if found_directories
                .iter()
                .any(|already_found_dirs| dir.starts_with(already_found_dirs))
            {
                return;
            }

            if let Ok(entries) = fs::read_dir(dir) {
                let subdirectories: Vec<DirEntry> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.metadata()
                            .map(|metadata| metadata.is_dir())
                            .unwrap_or(false)
                    })
                    .collect();

                for dir in &subdirectories {
                    if dir.path().ends_with(".git") {
                        if let Some(parent) = dir.path().parent() {
                            found_directories.insert(parent.to_path_buf());
                        }
                    }
                }

                for dir in &subdirectories {
                    walk(pb, found_directories, &dir.path());
                }
            }
        }

        for search_dir in &self.search {
            let root_dir = fs::canonicalize(shellexpand::tilde(search_dir).to_string())?;
            walk(&pb, &mut paths, &root_dir);
        }

        pb.finish_and_clear();

        info!("Found {} git repositories", paths.len());

        Ok(paths.into_iter().collect())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Rule {
    Path(PathRule),
    GitRepositories(GitRepositoriesRule),
}

impl Evaluatable for Rule {
    fn evaluate(&self) -> Result<Vec<PathBuf>> {
        match &self {
            Rule::Path(rule) => rule.evaluate(),
            Rule::GitRepositories(rule) => rule.evaluate(),
        }
    }
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
                for path in &rule.evaluate()? {
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
