use std::fs;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::vscode_tasks_parser::TasksJson;

/// Sanitize a label to be a valid shell alias
fn sanitize_label(label: &str) -> String {
    label
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "_")
}

fn file_exists(path: &str) -> bool {
    let p = Path::new(path);
    return p.is_file();
}

/// Generate aliases file from TasksJson
pub fn generate_from_tasks(tasks: &TasksJson, output_path: &str) -> std::io::Result<()> {
    let path = Path::new(output_path);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)?;

    for task in &tasks.tasks {
        if let Some(cmd) = &task.command {
            let alias_name = sanitize_label(&task.label);
            writeln!(file, "alias aliasx-{}='{}'", alias_name, cmd)?;
        }
    }

    Ok(())
}

/// list all aliases
pub fn list_all(output_path: &str) -> io::Result<()> {
    if !file_exists(output_path) {
        return Ok(());
    }

    let file = fs::File::open(output_path)?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result?;
        println!("{}", line);
    }

    Ok(())
}

/// list all aliases
pub fn clear(output_path: &str) -> io::Result<()> {
    if !file_exists(output_path) {
        return Ok(());
    }

    return fs::remove_file(output_path);
}
