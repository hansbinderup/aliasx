use serde::{Deserialize, Serialize};
use serde_json5::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Serialize,Deserialize)]
pub struct Task {
    pub label: String,
    pub command: Option<String>,
}

#[derive(Debug,Serialize,  Deserialize)]
pub struct TasksJson {
    pub version: Option<String>,
    pub tasks: Vec<Task>,
}

/* Function to read a JSON file and parse it */
pub fn read_tasks_from_file<P: AsRef<Path>>(path: P) -> Result<TasksJson> {
    // Open the file
    let file = File::open(path)?;
    
    // Wrap in a buffered reader for efficiency
    let reader = BufReader::new(file);
    
    // Parse JSON from the reader
    let tasks: TasksJson = serde_json5::from_reader(reader)?;
    
    Ok(tasks)
}
