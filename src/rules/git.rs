use super::Evaluatable;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, DirEntry},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    search: Vec<String>,
}

impl Evaluatable for Rule {
    fn evaluate(&self, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
        let mut new_paths = BTreeSet::new();

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

        info!("Searching for git repositories");

        let spinner_style = ProgressStyle::with_template("{spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let pb = ProgressBar::new(0);
        pb.set_style(spinner_style);

        for search_dir in &self.search {
            let root_dir = fs::canonicalize(shellexpand::tilde(search_dir).to_string())?;
            debug!("Searching in '{}'", root_dir.display());
            walk(&pb, &mut new_paths, &root_dir);
        }

        pb.finish_and_clear();

        info!("Found {} git repositories", new_paths.len());

        paths.extend(new_paths.iter().cloned());

        Ok(())
    }
}
