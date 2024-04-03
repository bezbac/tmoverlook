use super::Evaluatable;
use anyhow::Result;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    process::{Command, Output},
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum Shell {
    Zsh,
    Bash,
}

fn execute_shell_command(shell: &Shell, command: &str) -> Result<Output> {
    let output = match shell {
        Shell::Zsh => Command::new("zsh").arg("-c").arg(command).output()?,
        Shell::Bash => Command::new("bash").arg("-c").arg(command).output()?,
    };

    Ok(output)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    command: String,
    shell: Shell,
}

impl Evaluatable for Rule {
    fn evaluate(&self, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
        let output = execute_shell_command(&self.shell, &self.command)?;

        if !output.status.success() {
            warn!(
                "'{}' exited with status code {}, run tmoverlook in debug mode for more information",
                self.command, output.status
            );

            debug!("{}", std::str::from_utf8(&output.stdout)?);
            debug!("{}", std::str::from_utf8(&output.stderr)?);

            return Err(anyhow::anyhow!(
                "Failed to execute command '{}'",
                self.command
            ));
        }

        let path = PathBuf::from(String::from_utf8(output.stdout)?.trim());

        if !path.exists() {
            warn!(
                "The command returned a nonexistent path: '{}'",
                path.display()
            );
            return Err(anyhow::anyhow!(
                "The command returned a nonexistent path '{}'",
                self.command
            ));
        }

        paths.insert(path);

        Ok(())
    }
}
