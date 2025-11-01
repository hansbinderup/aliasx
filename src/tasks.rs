use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs;
use std::io::{ BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Serialize,Deserialize)]
pub struct TaskEntry {
    pub label: String,
    pub command: String,
}

#[derive(Debug,Serialize,  Deserialize)]
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

fn file_exists(path: &str) -> bool {
    let p = Path::new(path);
    return p.is_file();
}

/// list all tasks
pub fn list_all(output_path: &str) -> anyhow::Result<()> {
    if !file_exists(output_path) {
        return Ok(());
    }

    let tasks = read_tasks_from_file_yaml(output_path)?;
    for task in &tasks.tasks {
        println!("{} -> {}", task.label, task.command);
    }

    Ok(())
}

/// list all aliases
pub fn clear(output_path: &str) -> anyhow::Result<()> {
    if !file_exists(output_path) {
        return Ok(());
    }

     fs::remove_file(output_path)?;

     Ok(())
}
