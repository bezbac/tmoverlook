use anyhow::Result;
use log::info;
use std::process::Command;

pub fn run() -> Result<()> {
    info!("Starting analyze process");

    let backups = Command::new("tmutil")
        .arg("listbackups")
        .output()
        .expect("Failed to execute tmutil");

    let backups = String::from_utf8(backups.stdout)?;

    let backup_paths = backups
        .split("\n")
        .filter(|p| !p.trim().is_empty())
        .collect::<Vec<&str>>();

    Ok(())
}
