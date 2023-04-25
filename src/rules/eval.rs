use super::Evaluatable;
use anyhow::Result;
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
    fn evaluate(&self, paths: &mut BTreeSet<String>) -> Result<()> {
        let output = execute_shell_command(&self.shell, &self.command)?;

        assert!(output.status.success());

        let path = PathBuf::from(String::from_utf8(output.stdout)?)
            .to_str()
            .unwrap()
            .trim()
            .to_string();

        paths.insert(path);

        Ok(())
    }
}
