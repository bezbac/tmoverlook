use crate::{cache::Cache, contants::DEFAULT_CACHE_PATH};
use anyhow::Result;
use log::info;

pub fn run() -> Result<()> {
    let cache = Cache::read(&DEFAULT_CACHE_PATH)?;

    if cache.paths.is_empty() {
        info!("No ignored paths");
    } else {
        info!("Currenly ignored paths:");
        for path in cache.paths {
            info!("{}", path);
        }
    }

    Ok(())
}
