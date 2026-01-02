use anyhow::Result;
use execute::Execute;
use std::process::{Command, Stdio};

use crate::task_collection::TaskCollection;
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

pub fn get_aliases_as_tasks() -> anyhow::Result<TaskCollection> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());

    let mut command = Command::new(shell);
    command.args(["-ic", "alias"]);
    command.stdout(Stdio::piped());

    let output = command.execute_output()?;

    match output.status.code() {
        Some(0) => {}
        Some(code) => {
            return Err(anyhow::anyhow!(
                "calling 'alias' failed (exit code {})",
                code
            ))
        }
        None => return Err(anyhow::anyhow!("calling 'alias' was interrupted")),
    }

    let output_str = String::from_utf8(output.stdout)?;
    let aliases = parse_aliases(&output_str)?;

    Ok(TaskCollection::new(vec![aliases]))
}
