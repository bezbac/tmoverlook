use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::Confirm;
use env_logger::Builder;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use log::{info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, DirEntry};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};

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

        #[arg(short, long)]
        yes: bool,
    },
}

trait Evaluatable {
    fn evaluate(&self, paths: &mut BTreeSet<String>) -> Result<()>;
}

#[derive(Serialize, Deserialize, Debug)]
struct PathRule {
    path: String,
}

impl Evaluatable for PathRule {
    fn evaluate(&self, paths: &mut BTreeSet<String>) -> Result<()> {
        let expanded = fs::canonicalize(shellexpand::tilde(&self.path).to_string())?
            .to_str()
            .map(|str| str.to_string());

        if let Some(path) = expanded {
            paths.insert(path);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum Shell {
    Zsh,
    Bash,
}

fn execute_shell_command(shell: &Shell, command: &str) -> Result<Output> {
    let output = match shell {
        Shell::Zsh => Command::new("zsh").arg("-c").arg(command).output()?,
        Shell::Bash => Command::new("bash").arg("-c").arg(command).output()?,
    };

    Ok(output)
}

#[derive(Serialize, Deserialize, Debug)]
struct EvalRule {
    command: String,
    shell: Shell,
}

impl Evaluatable for EvalRule {
    fn evaluate(&self, paths: &mut BTreeSet<String>) -> Result<()> {
        let output = execute_shell_command(&self.shell, &self.command)?;

        assert!(output.status.success());

        let path = PathBuf::from(String::from_utf8(output.stdout)?)
            .to_str()
            .unwrap()
            .trim()
            .to_string();

        paths.insert(path);

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GitRepositoriesRule {
    search: Vec<String>,
}

impl Evaluatable for GitRepositoriesRule {
    fn evaluate(&self, paths: &mut BTreeSet<String>) -> Result<()> {
        fn walk(pb: &ProgressBar, found_directories: &mut BTreeSet<String>, dir: &PathBuf) {
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
                        if let Some(Some(parent)) = dir
                            .path()
                            .parent()
                            .map(|p| p.to_str().map(|str| str.to_string()))
                        {
                            found_directories.insert(parent);
                        }
                    }
                }

                for dir in &subdirectories {
                    walk(pb, found_directories, &dir.path());
                }
            }
        }

        info!("Searching for git repositories");

        let spinner_style = ProgressStyle::with_template("{spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let pb = ProgressBar::new(0);
        pb.set_style(spinner_style);

        for search_dir in &self.search {
            let root_dir = fs::canonicalize(shellexpand::tilde(search_dir).to_string())?;
            walk(&pb, paths, &root_dir);
        }

        pb.finish_and_clear();

        info!("Found {} git repositories", paths.len());

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Rule {
    Path(PathRule),
    Eval(EvalRule),
    GitRepositories(GitRepositoriesRule),
}

impl Evaluatable for Rule {
    fn evaluate(&self, paths: &mut BTreeSet<String>) -> Result<()> {
        match &self {
            Rule::Path(rule) => rule.evaluate(paths),
            Rule::Eval(rule) => rule.evaluate(paths),
            Rule::GitRepositories(rule) => rule.evaluate(paths),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    rules: Vec<Rule>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Cache {
    paths: BTreeSet<String>,
}

fn read_cache() -> Result<Cache> {
    let input = fs::read_to_string(DEFAULT_CACHE_PATH.as_str());
    let cache = match input {
        Err(_) => Cache {
            paths: BTreeSet::new(),
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

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum Diff {
    Unchanged,
    Added,
    Removed,
}

fn get_rule_priority(rule: &Rule) -> usize {
    match &rule {
        Rule::Path(_) => 3,
        Rule::Eval(_) => 2,
        Rule::GitRepositories(_) => 1,
    }
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
                info!("{}", path);
            }
        }
        Some(Commands::Apply {
            preview,
            config,
            yes,
        }) => {
            if *preview {
                info!("Preview mode is active, no changes will be applied");
            }

            let config = read_config(config.as_deref());
            let cache = read_cache()?;

            let mut paths: BTreeSet<String> = cache.paths.clone();

            let mut rules = config?.rules;

            rules.sort_by_key(|a| std::cmp::Reverse(get_rule_priority(a)));

            for rule in rules {
                rule.evaluate(&mut paths)?;
            }

            let changes: BTreeSet<_> = paths
                .iter()
                .chain(cache.paths.iter())
                .map(|p| {
                    let is_in_cache = cache.paths.contains(p);
                    let is_in_new_list = paths.contains(p);

                    let operation: Diff = if is_in_cache && is_in_new_list {
                        Diff::Unchanged
                    } else if is_in_cache && !is_in_new_list {
                        Diff::Removed
                    } else {
                        Diff::Added
                    };

                    (p, operation)
                })
                .collect();

            info!("Diff:");
            for (path, operation) in &changes {
                let diff_char = match operation {
                    Diff::Unchanged => "·",
                    Diff::Added => "+",
                    Diff::Removed => "-",
                };

                info!("{} {}", diff_char, path);
            }

            if *preview {
                return Ok(());
            }

            if changes.iter().filter(|c| c.1 != Diff::Unchanged).count() == 0 {
                info!("No changes to apply");
                return Ok(());
            }

            let confirmation = if *yes {
                true
            } else {
                Confirm::new()
                    .with_prompt("Do you wan't to apply these changes?")
                    .interact()?
            };

            if !confirmation {
                info!("Aborting");
                return Ok(());
            }

            info!("Applying changes");

            for (path, operation) in &changes {
                match operation {
                    Diff::Unchanged | Diff::Added => add_exclusion(path)?,
                    Diff::Removed => remove_exclusion(path)?,
                }
            }

            write_cache(&Cache {
                paths: paths.iter().map(|e| e.to_string()).collect(),
            })?;

            info!("Changes successfully applied!");
        }
        None => {}
    }

    Ok(())
}
