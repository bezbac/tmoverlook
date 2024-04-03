use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    pub paths: BTreeSet<PathBuf>,
}

impl Cache {
    pub fn read(path: &PathBuf) -> Result<Cache> {
        let input = fs::read_to_string(path);
        let cache = match input {
            Err(_) => Cache {
                paths: BTreeSet::new(),
            },
            Ok(input) => toml::from_str(&input)?,
        };
        Ok(cache)
    }

    pub fn write(&self, path: &PathBuf) -> Result<()> {
        let dir = path.parent().unwrap();

        fs::create_dir_all(dir)?;

        let contents = toml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}
