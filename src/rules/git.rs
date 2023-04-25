use super::Evaluatable;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
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
