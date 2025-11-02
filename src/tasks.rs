use anyhow::anyhow;
use execute::shell;
use fuzzy_select::FuzzySelect;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
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

pub fn write_tasks_to_file(tasks: &Tasks, path: &str) -> anyhow::Result<()> {
    let path = Path::new(path);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create (or truncate) the file for writing
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    serde_yaml::to_writer(writer, tasks)?;

    Ok(())
}

/* Function to read a JSON file and parse it */
pub fn read_tasks_from_file_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks> {
    // Open the file
    let file = File::open(path)?;

    // Wrap in a buffered reader for efficiency
    let reader = BufReader::new(file);

    // Parse JSON from the reader
    let tasks: Tasks = serde_yaml::from_reader(reader)?;

    Ok(tasks)
}

/* Function to read a JSON file and parse it */
pub fn read_tasks_from_file_json<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks> {
    // Open the file
    let file = File::open(path)?;

    // Wrap in a buffered reader for efficiency
    let reader = BufReader::new(file);

    // Parse JSON from the reader
    let tasks: Tasks = serde_json5::from_reader(reader)?;

    Ok(tasks)
}

pub fn get_all_tasks() -> anyhow::Result<Tasks> {
    let mut tasks = if file_exists(".aliasx.yaml") {
        read_tasks_from_file_yaml(".aliasx.yaml")?
    } else {
        Tasks::default()
    };

    if file_exists(".vscode/tasks.json") {
        let mut vscode_tasks = read_tasks_from_file_json(".vscode/tasks.json")?;
        tasks.tasks.append(&mut vscode_tasks.tasks);
    }

    Ok(tasks)
}

fn file_exists(path: &str) -> bool {
    let p = Path::new(path);
    return p.is_file();
}

fn print_task(task: &TaskEntry, id: usize, detailed: bool, width: usize) {
    if detailed {
        println!("[{:0>width$}] {} -> {}", id, task.label, task.command);
    } else {
        println!("[{:0>width$}] {}", id, task.label);
    }
}

/// list all tasks
pub fn list_at(id: usize, detailed: bool) -> anyhow::Result<()> {
    let tasks = get_all_tasks()?;
    let width_id = tasks.tasks.len().to_string().len();
    let task = tasks
        .tasks
        .get(id)
        .ok_or_else(|| anyhow!("invalid id: {}", id))?;

    print_task(task, id, detailed, width_id);

    Ok(())
}

/// list all tasks
pub fn list_all(detailed: bool) -> anyhow::Result<()> {
    let tasks = get_all_tasks()?;
    let width_id = tasks.tasks.len().to_string().len();

    for (i, task) in tasks.tasks.iter().enumerate() {
        print_task(task, i, detailed, width_id);
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
