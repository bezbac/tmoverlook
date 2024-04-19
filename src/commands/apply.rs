use crate::cache::Cache;
use crate::config::Config;
use crate::contants::DEFAULT_CACHE_PATH;
use crate::contants::DEFAULT_CONFIG_PATH;
use crate::rules::Evaluatable;
use crate::utils::tmutil::add_exclusion;
use crate::utils::tmutil::remove_exclusion;
use crate::Commands;
use anyhow::Result;
use dialoguer::Confirm;
use log::info;
use log::warn;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum Diff {
    Unchanged,
    Added,
    Removed,
}

pub fn run(cmd: &Commands) -> Result<()> {
    let Commands::Apply {
        config,
        yes,
        preview,
    } = cmd
    else {
        panic!("Invalid command passed to run");
    };

    if *preview {
        info!("Preview mode is active, no changes will be applied");
    }

    let config_path = config
        .as_ref()
        .map_or(DEFAULT_CONFIG_PATH.to_path_buf(), |c| PathBuf::from(c));

    let config = Config::read(&config_path).map_err(|_| {
        anyhow::anyhow!(
            "Could not read config file '{}'",
            config_path.to_string_lossy()
        )
    })?;

    info!("Using config file '{}'", config_path.to_string_lossy());

    let cache = Cache::read(&DEFAULT_CACHE_PATH)?;

    let mut paths: BTreeSet<PathBuf> = cache.paths.clone();

    let mut rules = config.rules;

    rules.sort_by_key(|a| std::cmp::Reverse(a.get_priority()));

    for rule in rules {
        rule.evaluate(&mut paths).unwrap_or_else(|_| {
            warn!("Failed to evaluate rule {:?}", rule);
        });
    }

    paths = paths
        .into_iter()
        .filter_map(|p| {
            if let Ok(path) = std::fs::canonicalize(&p) {
                Some(path)
            } else {
                warn!("Failed to canonicalize path '{}', skipping", &p.display());
                None
            }
        })
        .collect();

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

        info!("{} {}", diff_char, path.display());
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
            Diff::Unchanged | Diff::Added => add_exclusion(path).unwrap_or_else(|_| {
                warn!("Failed to add exclusion for {}", path.display());
            }),
            Diff::Removed => remove_exclusion(path).unwrap_or_else(|_| {
                warn!("Failed to remove exclusion for {}", path.display());
            }),
        }
    }

    Cache {
        paths: paths.iter().cloned().collect(),
    }
    .write(&DEFAULT_CACHE_PATH)?;

    info!("Changes successfully applied!");

    Ok(())
}
