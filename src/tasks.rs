use anyhow::anyhow;
use clap::ValueEnum;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

use crate::input::Input;
use crate::task_collection::TaskCollection;

#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct TaskEntry {
    pub label: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tasks {
    pub version: Option<String>,
    pub tasks: IndexSet<TaskEntry>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Input>,
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
    pub fn get_input(&self, id: &str) -> anyhow::Result<&Input> {
        if self.inputs.is_empty() {
            return Err(anyhow!("no inputs defined"));
        }

        self.inputs
            .iter()
            .find(|input| input.id == id)
            .ok_or_else(|| anyhow!("input with id '{}' not found", id))
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

pub fn get_all_tasks(filter: TaskFilter) -> anyhow::Result<TaskCollection> {
    let local_aliasx_path = Path::new(".aliasx.yaml");
    let local_vscode_tasks = Path::new(".vscode/tasks.json");

    let global_path_binding = shellexpand::tilde("~/.aliasx.yaml");
    let global_path = Path::new(global_path_binding.as_ref());

    let mut sources = Vec::new();

    if filter.include_local() && local_aliasx_path.is_file() {
        sources.push(YamlTaskReader::parse_file(local_aliasx_path)?);
    }

    if filter.include_local() && local_vscode_tasks.is_file() {
        sources.push(JsonTaskReader::parse_file(local_vscode_tasks)?);
    }

    if filter.include_global() && global_path.is_file() {
        sources.push(YamlTaskReader::parse_file(global_path)?);
    }

    Ok(TaskCollection::new(sources))
}
