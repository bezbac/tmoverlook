use crate::{cache::Cache, contants::DEFAULT_CACHE_PATH};
use anyhow::Result;
use log::info;

pub fn run() -> Result<()> {
    let cache = Cache::read(&DEFAULT_CACHE_PATH)?;

    info!("Currenly ignored paths:");
    for path in cache.paths {
        info!("{}", path);
    }

    Ok(())
}
