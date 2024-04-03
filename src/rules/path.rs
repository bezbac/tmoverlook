use super::Evaluatable;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    path: String,
}

impl Evaluatable for Rule {
    fn evaluate(&self, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
        let expanded = shellexpand::tilde(&self.path);
        paths.insert(PathBuf::from(expanded.to_string()));
        Ok(())
    }
}
