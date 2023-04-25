use super::Evaluatable;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs};

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    path: String,
}

impl Evaluatable for Rule {
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
