use crate::rules::Rule;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
}

impl Config {
    pub fn read(path: &PathBuf) -> Result<Config> {
        let path = fs::canonicalize(path)?;
        let input = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&input)?;
        Ok(config)
    }
}
