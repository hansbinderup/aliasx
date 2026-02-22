use anyhow::anyhow;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::input::Input;
use crate::input_mapping::InputMapping;
use crate::task_collection::TaskCollection;
use crate::task_filter::TaskFilter;

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

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<InputMapping>,
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

    pub fn get_mapping(&self, id: &str) -> anyhow::Result<&InputMapping> {
        if self.mappings.is_empty() {
            return Err(anyhow!("no mappings defined"));
        }

        self.mappings
            .iter()
            .find(|input| input.id == id)
            .ok_or_else(|| anyhow!("mapping with id '{}' not found", id))
    }

    pub fn apply_mappings(
        &self,
        command: &str,
        input_selections: &mut IndexMap<String, String>,
    ) -> anyhow::Result<String> {
        let mapping_strings = InputMapping::extract_from_str(command);
        let mut mapped_str = command.to_string();

        for map_str in mapping_strings {
            let input_mapping = self.get_mapping(&map_str)?;
            let key = &input_mapping.input;

            /* if input is not already selected, prompt for it */
            if !input_selections.contains_key(key) {
                let fzf_selection = self.get_input(key)?.fzf()?;
                input_selections.insert(key.clone(), fzf_selection);
            }

            /* we could unwrap here since we just inserted it if it was missing
             *  but let's be pragmatic with user warnings/errors */
            let sel_value = input_selections
                .get(key)
                .ok_or_else(|| anyhow!("no selection found for input '{}'", key))?;

            let mapping = input_mapping.options.get(sel_value).ok_or_else(|| {
                anyhow!(
                    "no mapping found for selection '{}' in input '{}'",
                    sel_value,
                    key
                )
            })?;

            mapped_str = InputMapping::replace_all(&mapped_str, &input_mapping.id, mapping)?;
        }

        Ok(mapped_str)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(label: &str, command: &str) -> TaskEntry {
        TaskEntry {
            label: label.to_string(),
            command: command.to_string(),
        }
    }

    fn create_test_input(id: &str, options: Vec<&str>) -> Input {
        Input {
            id: id.to_string(),
            options: options.iter().map(|s| s.to_string()).collect(),
            description: None,
            default: None,
        }
    }

    #[test]
    fn test_validate_config_no_inputs() {
        use crate::validator::Validator;

        let tasks = Tasks::default();
        let task = create_test_task("simple", "echo hello");

        let validator = Validator { verbose: false };
        let report = validator.validate_task_command(&task, &tasks);
        assert!(!report.has_failures());
    }

    #[test]
    fn test_validate_config_valid_input() {
        use crate::validator::Validator;

        let mut tasks = Tasks::default();
        tasks
            .inputs
            .push(create_test_input("env", vec!["dev", "prod"]));

        let task = create_test_task("deploy", "deploy ${input:env}");

        let validator = Validator { verbose: false };
        let report = validator.validate_task_command(&task, &tasks);
        assert!(!report.has_failures());
    }

    #[test]
    fn test_validate_config_missing_input() {
        use crate::validator::Validator;

        let tasks = Tasks::default();
        let task = create_test_task("deploy", "deploy ${input:missing}");

        let validator = Validator { verbose: false };
        let report = validator.validate_task_command(&task, &tasks);
        assert!(report.has_failures());
    }

    #[test]
    fn test_validate_config_multiple_valid_inputs() {
        use crate::validator::Validator;

        let mut tasks = Tasks::default();
        tasks
            .inputs
            .push(create_test_input("env", vec!["dev", "prod"]));
        tasks
            .inputs
            .push(create_test_input("region", vec!["us", "eu"]));

        let task = create_test_task("deploy", "deploy ${input:env} ${input:region}");

        let validator = Validator { verbose: false };
        let report = validator.validate_task_command(&task, &tasks);
        assert!(!report.has_failures());
    }

    #[test]
    fn test_validate_config_multiple_inputs_one_missing() {
        use crate::validator::Validator;

        let mut tasks = Tasks::default();
        tasks
            .inputs
            .push(create_test_input("env", vec!["dev", "prod"]));

        let task = create_test_task("deploy", "deploy ${input:env} ${input:missing}");

        let validator = Validator { verbose: false };
        let report = validator.validate_task_command(&task, &tasks);
        assert!(report.has_failures());
    }

    #[test]
    fn test_validate_config_duplicate_input_references() {
        use crate::validator::Validator;

        let mut tasks = Tasks::default();
        tasks
            .inputs
            .push(create_test_input("env", vec!["dev", "prod"]));

        let task = create_test_task("deploy", "deploy ${input:env} again ${input:env}");

        let validator = Validator { verbose: false };
        let report = validator.validate_task_command(&task, &tasks);
        assert!(!report.has_failures());
    }

    #[test]
    fn test_get_input_exists() {
        let mut tasks = Tasks::default();
        tasks
            .inputs
            .push(create_test_input("env", vec!["dev", "prod"]));

        let result = tasks.get_input("env");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "env");
    }

    #[test]
    fn test_get_input_not_found() {
        let mut tasks = Tasks::default();
        tasks.inputs.push(create_test_input("other", vec!["value"]));

        let result = tasks.get_input("missing");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found"),
            "Expected 'not found' in: {}",
            err_msg
        );
    }

    #[test]
    fn test_get_input_empty_inputs() {
        let tasks = Tasks::default();

        let result = tasks.get_input("any");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no inputs defined"));
    }

    #[test]
    fn test_task_entry_format_verbose() {
        let task = create_test_task("test-task", "echo hello");
        assert_eq!(task.format(true), "test-task -> echo hello");
    }

    #[test]
    fn test_task_entry_format_non_verbose() {
        let task = create_test_task("test-task", "echo hello");
        assert_eq!(task.format(false), "test-task");
    }
}
