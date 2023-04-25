use anyhow::anyhow;
use anyhow::Result;
use log::debug;
use std::process::Command;

pub fn add_exclusion(path: &str) -> Result<()> {
    let output = Command::new("tmutil")
        .arg("addexclusion")
        .arg(path)
        .output()?;

    if !output.status.success() {
        debug!("{:?}", String::from_utf8(output.stderr)?);
        return Err(anyhow!("Failed to add exclusion for {}", path));
    }

    Ok(())
}

pub fn remove_exclusion(path: &str) -> Result<()> {
    let output = Command::new("tmutil")
        .arg("removeexclusion")
        .arg(path)
        .output()?;

    assert!(output.status.success());

    Ok(())
}
