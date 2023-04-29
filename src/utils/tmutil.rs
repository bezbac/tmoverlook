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

#[derive(Debug)]
pub enum CompareArgs {
    Current,
    Backups { first: String, second: String },
}

pub fn compare(args: &CompareArgs) -> Result<()> {
    let mut cmd = &mut Command::new("tmutil");
    cmd = cmd.arg("compare").arg("-aX");

    match args {
        CompareArgs::Current => {}
        CompareArgs::Backups { first, second } => cmd = cmd.args([first, second]),
    }

    let output = cmd.output()?;

    if !output.status.success() {
        debug!("{:?}", String::from_utf8(output.stderr)?);
        return Err(anyhow!("Failed to execute compare"));
    }

    assert!(output.status.success());

    dbg!(output.stdout);

    Ok(())
}

pub fn list_backups() -> Result<Vec<String>> {
    let backups = Command::new("tmutil")
        .arg("listbackups")
        .output()
        .expect("Failed to execute tmutil");

    let backups = String::from_utf8(backups.stdout)?;

    let backup_paths = backups
        .split("\n")
        .filter(|p| !p.trim().is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    Ok(backup_paths)
}
