use std::fs;
use std::io::Write;
use std::path::Path;

use crate::vscode_tasks::parser::TasksJson;

/// Sanitize a label to be a valid shell alias
fn sanitize_label(label: &str) -> String {
    label
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "_")
}

/// Generate aliases file from TasksJson
pub fn generate_aliases_file(tasks: &TasksJson, output_path: &str) -> std::io::Result<()> {
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
