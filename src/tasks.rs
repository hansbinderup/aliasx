use anyhow::{anyhow, Context};
use clap::ValueEnum;
use execute::shell;
use fuzzy_select::FuzzySelect;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::process::Stdio;

#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct TaskEntry {
    pub label: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tasks {
    pub version: Option<String>,
    pub tasks: IndexSet<TaskEntry>,
}

impl TaskEntry {
    pub fn format(&self, verbose: bool) -> String {
        if verbose {
            format!("{} -> {}", self.label, self.command)
        } else {
            format!("{}", self.label)
        }
    }

    pub fn print(&self, id: usize, verbose: bool, width: usize) {
        println!("[{:0>width$}] {}", id, self.format(verbose));
    }
}

impl Tasks {
    pub fn list_at(&self, id: usize, verbose: bool) -> anyhow::Result<()> {
        let width_id = self.tasks.len().to_string().len();
        let task = self
            .tasks
            .get_index(id)
            .ok_or_else(|| anyhow!("invalid id: {}", id))?;

        task.print(id, verbose, width_id);

        Ok(())
    }

    pub fn list_all(&self, verbose: bool) -> anyhow::Result<()> {
        let width_id = self.tasks.len().to_string().len();

        for (i, task) in self.tasks.iter().enumerate() {
            task.print(i, verbose, width_id);
        }

        Ok(())
    }

    pub fn fzf(&self, query: &str, verbose: bool) -> anyhow::Result<()> {
        let width = self.tasks.len().to_string().len();

        // Create display strings for each task
        let task_strings: Vec<String> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| format!("[{:0>width$}] {}", i, task.format(verbose)))
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

        self.execute(id, verbose)?;

        Ok(())
    }

    pub fn execute(&self, id: usize, verbose: bool) -> anyhow::Result<()> {
        if id >= self.tasks.len() {
            return Err(anyhow::anyhow!("invalid id"));
        }

        let task = &self.tasks[id];

        println!("aliasx | {}\n", task.format(verbose));

        // Create a shell command via `execute` crate
        let mut cmd = shell(&task.command);

        // Inherit stdio for live output, like a normal terminal
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        // Run the command and wait for completion
        let status = cmd.status().with_context(|| "failed to execute command")?;

        if !status.success() {
            let code_str = status
                .code()
                .map_or_else(|| "unknown".to_string(), |c| c.to_string());

            return Err(anyhow!(
                "command exited with non-zero status (err={})",
                code_str
            ));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TaskFilter {
    All,
    Local,
    Global,
}

impl TaskFilter {
    pub fn include_local(self) -> bool {
        matches!(self, TaskFilter::All | TaskFilter::Local)
    }

    pub fn include_global(self) -> bool {
        matches!(self, TaskFilter::All | TaskFilter::Global)
    }
}

impl fmt::Display for TaskFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskFilter::All => "all",
            TaskFilter::Local => "local",
            TaskFilter::Global => "global",
        };
        write!(f, "{}", s)
    }
}

pub fn get_all_tasks(filter: TaskFilter) -> anyhow::Result<Tasks> {
    let local_aliasx_path = Path::new(".aliasx.yaml");
    let local_vscode_tasks = Path::new(".vscode/tasks.json");

    // tildes are handles a bit differently - needs to be expanded
    let global_path_binding = shellexpand::tilde("~/.aliasx.yaml");
    let global_path = Path::new(global_path_binding.as_ref());

    let mut tasks = if filter.include_local() && local_aliasx_path.is_file() {
        YamlTaskReader::parse_file(local_aliasx_path)?
    } else {
        Tasks::default()
    };

    if filter.include_local() && local_vscode_tasks.is_file() {
        let mut vscode_tasks = JsonTaskReader::parse_file(local_vscode_tasks)?;
        tasks.tasks.append(&mut vscode_tasks.tasks);
    }

    if filter.include_global() && global_path.is_file() {
        let mut global_tasks = YamlTaskReader::parse_file(global_path)?;
        tasks.tasks.append(&mut global_tasks.tasks);
    }

    Ok(tasks)
}
