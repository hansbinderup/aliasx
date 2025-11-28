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

impl Tasks {
    pub fn list_at(&self, id: usize, detailed: bool) -> anyhow::Result<()> {
        let width_id = self.tasks.len().to_string().len();
        let task = self
            .tasks
            .get(id)
            .ok_or_else(|| anyhow!("invalid id: {}", id))?;

        task.print(id, detailed, width_id);

        Ok(())
    }

    pub fn list_all(&self, detailed: bool) -> anyhow::Result<()> {
        let width_id = self.tasks.len().to_string().len();

        for (i, task) in self.tasks.iter().enumerate() {
            task.print(i, detailed, width_id);
        }

        Ok(())
    }

    pub fn fzf(&self, query: &str) -> anyhow::Result<()> {
        let width = self.tasks.len().to_string().len();

        // Create display strings for each task
        let task_strings: Vec<String> = self
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

        self.execute(id)?;

        Ok(())
    }

    pub fn execute(&self, id: usize) -> anyhow::Result<()> {
        if id >= self.tasks.len() {
            return Err(anyhow::anyhow!("invalid id"));
        }

        let task = &self.tasks[id];

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
}

struct YamlTaskReader;
struct JsonTaskReader;

trait TaskReader {
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
    let local_aliasx_path = Path::new(".aliasx.yaml");
    let local_vscode_tasks = Path::new(".vscode/tasks.json");

    // tildes are handles a bit differently - needs to be expanded
    let global_path_binding = shellexpand::tilde("~/.aliasx.yaml");
    let global_path = Path::new(global_path_binding.as_ref());

    let mut tasks = if local_aliasx_path.is_file() {
        YamlTaskReader::parse_file(local_aliasx_path)?
    } else {
        Tasks::default()
    };

    if local_vscode_tasks.is_file() {
        let mut vscode_tasks = JsonTaskReader::parse_file(local_vscode_tasks)?;
        tasks.tasks.append(&mut vscode_tasks.tasks);
    }

    if global_path.is_file() {
        let mut global_tasks = YamlTaskReader::parse_file(global_path)?;
        tasks.tasks.append(&mut global_tasks.tasks);
    }

    Ok(tasks)
}
