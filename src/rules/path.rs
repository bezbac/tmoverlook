use super::Evaluatable;
use crate::environment;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    path: String,
}

impl Evaluatable for Rule {
    fn evaluate(&self, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
        let path = environment::expand_path(&self.path)?;
        paths.insert(path);
        Ok(())
    }
}
