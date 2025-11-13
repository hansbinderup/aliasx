use anyhow::anyhow;
use execute::shell;
use fuzzy_select::FuzzySelect;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskEntry {
    pub label: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tasks {
    pub version: Option<String>,
    pub tasks: Vec<TaskEntry>,
}

impl TaskEntry {
    pub fn print(&self, id: usize, detailed: bool, width: usize) {
        if detailed {
            println!("[{:0>width$}] {} -> {}", id, self.label, self.command);
        } else {
            println!("[{:0>width$}] {}", id, self.label);
        }
    }
}

pub struct YamlTaskReader;
pub struct JsonTaskReader;

pub trait TaskReader {
    fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks>;
}

impl TaskReader for YamlTaskReader {
    fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Ok(serde_yaml::from_reader(reader)?)
    }
}

impl TaskReader for JsonTaskReader {
    fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Ok(serde_json5::from_reader(reader)?)
    }
}

pub fn get_all_tasks() -> anyhow::Result<Tasks> {
    let mut tasks = if file_exists(".aliasx.yaml") {
        YamlTaskReader::parse_file(".aliasx.yaml")?
    } else {
        Tasks::default()
    };

    if file_exists(".vscode/tasks.json") {
        let mut vscode_tasks = JsonTaskReader::parse_file(".vscode/tasks.json")?;
        tasks.tasks.append(&mut vscode_tasks.tasks);
    }

    Ok(tasks)
}

fn file_exists(path: &str) -> bool {
    let p = Path::new(path);
    return p.is_file();
}

/// list all tasks
pub fn list_at(id: usize, detailed: bool) -> anyhow::Result<()> {
    let tasks = get_all_tasks()?;
    let width_id = tasks.tasks.len().to_string().len();
    let task = tasks
        .tasks
        .get(id)
        .ok_or_else(|| anyhow!("invalid id: {}", id))?;

    task.print(id, detailed, width_id);

    Ok(())
}

/// list all tasks
pub fn list_all(detailed: bool) -> anyhow::Result<()> {
    let tasks = get_all_tasks()?;
    let width_id = tasks.tasks.len().to_string().len();

    for (i, task) in tasks.tasks.iter().enumerate() {
        task.print(i, detailed, width_id);
    }

    Ok(())
}

pub fn execute(id: usize) -> anyhow::Result<()> {
    let tasks = get_all_tasks()?;

    if id >= tasks.tasks.len() {
        return Err(anyhow::anyhow!("invalid id"));
    }

    let task = &tasks.tasks[id];

    println!("aliasx | {}\n", task.label);

    // Create a shell command via `execute` crate
    let mut cmd = shell(&task.command);

    // Inherit stdio for live output, like a normal terminal
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Run the command and wait for completion
    let status = cmd.status()?; // returns std::process::ExitStatus

    if !status.success() {
        eprintln!("Command exited with status: {:?}", status.code());
    }

    Ok(())
}

pub fn fzf_task(query: &str) -> anyhow::Result<()> {
    let tasks = get_all_tasks()?; // your TasksJson or similar
    let width = tasks.tasks.len().to_string().len();

    // Create display strings for each task
    let task_strings: Vec<String> = tasks
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| format!("[{:0>width$}] {}", i, task.label))
        .collect();

    // hate this.. fix it

    // Show fuzzy picker and get selection index
    let selection = FuzzySelect::new()
        .with_prompt("Search:")
        .with_query(query)
        .with_options(task_strings.clone())
        .select()?;

    // Find the index of the selected string in the vector
    let id = task_strings
        .iter()
        .position(|s| s == &selection)
        .ok_or_else(|| anyhow!("Selected task not found"))?;

    execute(id)?;

    Ok(())
}
