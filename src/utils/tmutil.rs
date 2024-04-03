use anyhow::anyhow;
use anyhow::Result;
use log::debug;
use std::path::PathBuf;
use std::process::Command;

pub fn add_exclusion(path: &PathBuf) -> Result<()> {
    let output = Command::new("tmutil")
        .arg("addexclusion")
        .arg(path)
        .output()?;

    if !output.status.success() {
        debug!("{:?}", String::from_utf8(output.stderr)?);
        return Err(anyhow!("Failed to add exclusion for {}", path.display()));
    }

    Ok(())
}

pub fn remove_exclusion(path: &PathBuf) -> Result<()> {
    let output = Command::new("tmutil")
        .arg("removeexclusion")
        .arg(path)
        .output()?;

    if !output.status.success() {
        debug!("{:?}", String::from_utf8(output.stderr)?);
        return Err(anyhow!("Failed to remove exclusion for {}", path.display()));
    }

    Ok(())
}
