use std::path::PathBuf;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref DEFAULT_CONFIG_PATH: PathBuf =
        PathBuf::from(shellexpand::tilde("~/.config/tmoverlook/config.toml").to_string());
    pub static ref DEFAULT_CACHE_PATH: PathBuf =
        PathBuf::from(shellexpand::tilde("~/.local/share/tmoverlook/cache.toml").to_string());
}
