use anyhow::Result;
use execute::Execute;
use std::process::{Command, Stdio};

use crate::tasks::{TaskEntry, Tasks};

fn parse_aliases(output: &str) -> Result<Tasks> {
    let mut tasks = Tasks::default();

    for line in output.lines() {
        // only parse lines starting with "alias "
        if let Some(stripped) = line.strip_prefix("alias ") {
            if let Some((name, cmd)) = stripped.split_once('=') {
                let task = TaskEntry {
                    label: name.trim().to_string(),
                    command: cmd.trim_matches('\'').to_string(),
                };
                tasks.tasks.insert(task);
            }
        }
    }

    Ok(tasks)
}

pub fn get_aliases_as_tasks() -> anyhow::Result<Tasks> {
    // try to determine the user's shell or default to bash
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());

    // call 'alias' interactively and as command (ic)
    let mut command = Command::new(shell);
    command.args(["-ic", "alias"]);

    command.stdout(Stdio::piped());

    let output = command.execute_output()?;

    match output.status.code() {
        Some(0) => {} // success
        Some(code) => {
            return Err(anyhow::anyhow!(
                "calling 'alias' failed (exit code {})",
                code
            ))
        }
        None => return Err(anyhow::anyhow!("calling 'alias' was interrupted")),
    }

    let output_str = String::from_utf8(output.stdout)?;

    parse_aliases(&output_str)
}
