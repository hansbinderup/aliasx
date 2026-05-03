use anyhow::{anyhow, Context};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

use crate::input::Input;
use crate::input_mapping::InputMapping;
use crate::task_collection::TaskCollection;
use crate::task_conditions::TaskCondition;
use crate::task_filter::TaskFilter;
use crate::task_reader;

#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct TaskEntry {
    pub label: String,
    pub command: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<TaskCondition>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tasks {
    pub version: Option<String>,
    pub tasks: IndexSet<TaskEntry>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Input>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<InputMapping>,

    #[serde(skip)]
    pub scope: TaskFilter,
}

impl TaskEntry {
    pub fn format(&self, verbose: bool) -> String {
        if verbose {
            format!("{} -> {}", self.label, self.command)
        } else {
            format!("{}", self.label)
        }
    }

    pub fn print(&self, idx: usize, verbose: bool, width: usize) {
        println!("[{:0>width$}] {}", idx, self.format(verbose));
    }
}

impl Tasks {
    pub fn apply_conditions(&mut self) {
        self.tasks.retain(|t| {
            if let Some(c) = &t.conditions {
                c.is_valid()
            } else {
                true
            }
        });
    }

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

    /// Returns all inputs required to execute `command`, including those
    /// referenced indirectly through mappings.
    pub fn required_inputs_for_command(&self, command: &str) -> anyhow::Result<Vec<&Input>> {
        let mut ids: IndexSet<String> = IndexSet::new();

        for id in Input::extract_variables(command) {
            ids.insert(id);
        }

        for map_id in InputMapping::extract_from_str(command) {
            let mapping = self.get_mapping(&map_id)?;
            ids.insert(mapping.input.clone());
        }

        ids.iter().map(|id| self.get_input(id)).collect()
    }

    pub fn apply_mappings(
        &self,
        command: &str,
        input_selections: &IndexMap<String, String>,
    ) -> anyhow::Result<String> {
        let mapping_strings = InputMapping::extract_from_str(command);
        let mut mapped_str = command.to_string();

        for map_str in mapping_strings {
            let input_mapping = self.get_mapping(&map_str)?;
            let key = &input_mapping.input;

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

pub fn get_all_tasks(filter: TaskFilter, apply_conditions: bool) -> anyhow::Result<TaskCollection> {
    const LOCAL_SOURCES: &[&str] = &[
        ".aliasx.yaml",
        ".aliasx.yml",
        ".aliasx.json",
        ".aliasx.json5",
        ".vscode/tasks.json",
    ];

    const GLOBAL_SOURCES: &[&str] = &[
        ".aliasx.yaml",
        ".aliasx.yml",
        ".aliasx.json",
        ".aliasx.json5",
    ];

    let mut sources = Vec::new();

    if filter.include_local() {
        for local_path in LOCAL_SOURCES {
            task_reader::push_if_exists(&mut sources, local_path, TaskFilter::Local)?;
        }
    }

    if filter.include_global() {
        let home_path = dirs::home_dir().context("could not find global configs")?;

        for global_path in GLOBAL_SOURCES {
            task_reader::push_if_exists(
                &mut sources,
                home_path.join(global_path),
                TaskFilter::Global,
            )?;
        }
    }

    if apply_conditions {
        for source in sources.iter_mut() {
            source.apply_conditions();
        }
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
            id: Some("id".to_string()),
            conditions: Option::None,
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
