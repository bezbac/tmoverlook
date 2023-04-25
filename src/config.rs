use crate::rules::Rule;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
}

impl Config {
    pub fn read(path: &str) -> Result<Config> {
        let input = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&input)?;
        Ok(config)
    }
}
