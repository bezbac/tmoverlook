use std::path::PathBuf;

use lazy_static::lazy_static;

use crate::environment;

lazy_static! {
    pub static ref DEFAULT_CONFIG_PATH: PathBuf =
        environment::expand_path(&"~/.config/tmoverlook/config.toml").unwrap();
    pub static ref DEFAULT_CACHE_PATH: PathBuf =
        environment::expand_path(&"~/.local/share/tmoverlook/cache.toml").unwrap();
}
