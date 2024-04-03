use super::Evaluatable;
use anyhow::Result;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    path: String,
}

impl Evaluatable for Rule {
    fn evaluate(&self, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
        let expanded = fs::canonicalize(shellexpand::tilde(&self.path).to_string());

        match expanded {
            Ok(path) => {
                paths.insert(path);

                Ok(())
            }
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    warn!("Could not expand '{}'", self.path);
                    Ok(())
                }
                _ => Err(error.into()),
            },
        }
    }
}
