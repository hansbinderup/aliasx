use anyhow::{anyhow, Context};
use execute::shell;
use indexmap::{IndexMap, IndexSet};
use std::process::Stdio;

use crate::{
    input::Input,
    tasks::{TaskEntry, Tasks},
    validator::Validator,
};
use crate::task_filter::TaskFilter;

#[derive(Debug, Default)]
pub struct TaskCollection {
    sources: Vec<Tasks>,
}

impl TaskCollection {
    pub fn new(sources: Vec<Tasks>) -> Self {
        Self { sources }
    }

    fn total_count(&self) -> usize {
        self.sources.iter().map(|t| t.tasks.len()).sum()
    }

    pub fn width_idx(&self) -> usize {
        self.total_count().to_string().len()
    }

    fn all_tasks(&self) -> IndexSet<&TaskEntry> {
        self.sources.iter().flat_map(|t| t.tasks.iter()).collect()
    }

    fn all_tasks_with_source(&self) -> impl Iterator<Item = (usize, &Tasks, &TaskEntry)> {
        let mut global_idx = 0;
        self.sources.iter().flat_map(move |source| {
            let start_idx = global_idx;
            global_idx += source.tasks.len();

            source
                .tasks
                .iter()
                .enumerate()
                .map(move |(local_idx, task)| (start_idx + local_idx, source, task))
        })
    }

    fn find_task(&self, id: usize) -> anyhow::Result<(&Tasks, &TaskEntry)> {
        let mut current_idx = 0;

        for task_set in &self.sources {
            let next_idx = current_idx + task_set.tasks.len();
            if id < next_idx {
                let local_idx = id - current_idx;
                let task = task_set.tasks.get_index(local_idx).ok_or_else(|| {
                    anyhow!("internal_error! local idx '{}' not found", local_idx)
                })?;

                return Ok((task_set, task));
            }

            current_idx = next_idx;
        }

        Err(anyhow!("invalid index '{}'", id))
    }

    /// Returns all tasks as `(global_index, scope, &TaskEntry)`, deduplicated.
    /// NOTE: indexSet does not support standard slices so therefore the vector..
    pub fn indexed_tasks(&self) -> Vec<(usize, TaskFilter, &TaskEntry)> {
        let mut seen: IndexSet<&TaskEntry> = IndexSet::new();
        let mut result = Vec::new();
        for (idx, source, task) in self.all_tasks_with_source() {
            if seen.insert(task) {
                result.push((idx, source.scope, task));
            }
        }
        result
    }

    /// Returns the inputs required to execute task `id` (direct + via mappings).
    pub fn required_inputs_for_task(&self, id: usize) -> anyhow::Result<Vec<&Input>> {
        let (task_set, task) = self.find_task(id)?;
        task_set.required_inputs_for_command(&task.command)
    }

    pub fn validate_all(&self, verbose: bool) {
        let validator = Validator { verbose };
        let mut all_reports = Vec::new();

        Validator::print_header();

        for (_idx, source, task) in self.all_tasks_with_source() {
            let report = validator.validate_task_command(task, source);

            if verbose {
                report.print(verbose);
            } else {
                report.print_compact();
            }

            all_reports.push(report);
        }

        Validator::print_summary(&all_reports);
    }

    pub fn validate_at(&self, idx: usize, verbose: bool) -> anyhow::Result<()> {
        let validator = Validator { verbose };
        let (source, task) = self.find_task(idx)?;

        let report = validator.validate_task_command(task, source);

        // for single tasks we need to print a verbose report
        // otherwise there might be no output
        report.print(true);

        Ok(())
    }

    pub fn list_at(&self, id: usize, verbose: bool) -> anyhow::Result<()> {
        let (_, task) = self.find_task(id)?;
        task.print(id, verbose, self.width_idx());

        Ok(())
    }

    pub fn list_all(&self, verbose: bool) -> anyhow::Result<()> {
        for (idx, task) in self.all_tasks().iter().enumerate() {
            task.print(idx, verbose, self.width_idx());
        }

        Ok(())
    }

    /// Execute task `id` with pre-collected `input_selections`.
    pub fn execute(&self, id: usize, input_selections: IndexMap<String, String>, verbose: bool) -> anyhow::Result<()> {
        let (task_set, task) = self.find_task(id)?;

        let mut task_command = task.command.clone();

        for input_id in Input::extract_variables(&task.command) {
            let val = input_selections
                .get(&input_id)
                .ok_or_else(|| anyhow!("no selection provided for input '{}'", input_id))?;
            task_command = Input::replace_next_variable(&task_command, val);
        }

        task_command = task_set.apply_mappings(&task_command, &input_selections)?;

        self.run_command(task, task_command, verbose)
    }

    fn run_command(
        &self,
        task: &TaskEntry,
        task_command: String,
        verbose: bool,
    ) -> anyhow::Result<()> {
        println!("aliasx | {}\n", task.format(verbose));

        let mut cmd = shell(&task_command);
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(label: &str, command: &str) -> TaskEntry {
        TaskEntry {
            label: label.to_string(),
            command: command.to_string(),
        }
    }

    fn create_test_tasks(entries: Vec<(&str, &str)>) -> Tasks {
        let mut tasks = Tasks::default();
        for (label, command) in entries {
            tasks.tasks.insert(create_test_task(label, command));
        }
        tasks
    }

    fn create_test_tasks_with_inputs(entries: Vec<(&str, &str)>, inputs: Vec<Input>) -> Tasks {
        let mut tasks = create_test_tasks(entries);
        tasks.inputs = inputs;
        tasks
    }

    #[test]
    fn test_total_count() {
        let source1 = create_test_tasks(vec![("task1", "echo 1"), ("task2", "echo 2")]);
        let source2 = create_test_tasks(vec![("task3", "echo 3")]);

        let collection = TaskCollection::new(vec![source1, source2]);
        assert_eq!(collection.total_count(), 3);
    }

    #[test]
    fn test_total_count_empty() {
        let collection = TaskCollection::new(vec![]);
        assert_eq!(collection.total_count(), 0);
    }

    #[test]
    fn test_width_idx() {
        let source1 = create_test_tasks(vec![
            ("task1", "echo 1"),
            ("task2", "echo 2"),
            ("task3", "echo 3"),
            ("task4", "echo 4"),
            ("task5", "echo 5"),
            ("task6", "echo 6"),
            ("task7", "echo 7"),
            ("task8", "echo 8"),
            ("task9", "echo 9"),
            ("task10", "echo 10"),
        ]);

        let collection = TaskCollection::new(vec![source1]);
        assert_eq!(collection.width_idx(), 2); // 10 tasks, so 2 digits
    }

    #[test]
    fn test_all_tasks() {
        let source1 = create_test_tasks(vec![("task1", "echo 1"), ("task2", "echo 2")]);
        let source2 = create_test_tasks(vec![("task3", "echo 3")]);

        // duplicate task in source3 to test uniqueness and order
        let source3 = create_test_tasks(vec![("task3", "echo 3")]);

        let collection = TaskCollection::new(vec![source1, source2, source3]);
        let labels: Vec<String> = collection
            .all_tasks()
            .iter()
            .map(|t| t.label.clone())
            .collect();

        assert_eq!(labels, vec!["task1", "task2", "task3"]);
    }

    #[test]
    fn test_all_tasks_with_source_indices() {
        let source1 = create_test_tasks(vec![("task1", "echo 1"), ("task2", "echo 2")]);
        let source2 = create_test_tasks(vec![("task3", "echo 3"), ("task4", "echo 4")]);

        let collection = TaskCollection::new(vec![source1, source2]);
        let items: Vec<(usize, String)> = collection
            .all_tasks_with_source()
            .map(|(idx, _source, task)| (idx, task.label.clone()))
            .collect();

        assert_eq!(
            items,
            vec![
                (0, "task1".to_string()),
                (1, "task2".to_string()),
                (2, "task3".to_string()),
                (3, "task4".to_string()),
            ]
        );
    }

    #[test]
    fn test_all_tasks_with_source_indices_three_sources() {
        let source1 = create_test_tasks(vec![("task1", "echo 1")]);
        let source2 = create_test_tasks(vec![("task2", "echo 2"), ("task3", "echo 3")]);
        let source3 = create_test_tasks(vec![("task4", "echo 4")]);

        let collection = TaskCollection::new(vec![source1, source2, source3]);
        let items: Vec<(usize, String)> = collection
            .all_tasks_with_source()
            .map(|(idx, _source, task)| (idx, task.label.clone()))
            .collect();

        assert_eq!(
            items,
            vec![
                (0, "task1".to_string()),
                (1, "task2".to_string()),
                (2, "task3".to_string()),
                (3, "task4".to_string()),
            ]
        );
    }

    #[test]
    fn test_find_task_first_source() {
        let source1 = create_test_tasks(vec![("task1", "echo 1"), ("task2", "echo 2")]);
        let source2 = create_test_tasks(vec![("task3", "echo 3")]);

        let collection = TaskCollection::new(vec![source1, source2]);
        let (_, task) = collection.find_task(1).unwrap();

        assert_eq!(task.label, "task2");
    }

    #[test]
    fn test_find_task_second_source() {
        let source1 = create_test_tasks(vec![("task1", "echo 1"), ("task2", "echo 2")]);
        let source2 = create_test_tasks(vec![("task3", "echo 3")]);

        let collection = TaskCollection::new(vec![source1, source2]);
        let (_, task) = collection.find_task(2).unwrap();

        assert_eq!(task.label, "task3");
    }

    #[test]
    fn test_find_task_boundary() {
        let source1 = create_test_tasks(vec![("task1", "echo 1"), ("task2", "echo 2")]);
        let source2 = create_test_tasks(vec![("task3", "echo 3"), ("task4", "echo 4")]);

        let collection = TaskCollection::new(vec![source1, source2]);

        // Test boundary between sources
        let (_, task) = collection.find_task(1).unwrap();
        assert_eq!(task.label, "task2");

        let (_, task) = collection.find_task(2).unwrap();
        assert_eq!(task.label, "task3");
    }

    #[test]
    fn test_find_task_invalid_id() {
        let source1 = create_test_tasks(vec![("task1", "echo 1")]);

        let collection = TaskCollection::new(vec![source1]);
        let result = collection.find_task(5);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid index"));
    }

    #[test]
    fn test_find_task_empty_collection() {
        let collection = TaskCollection::new(vec![]);
        let result = collection.find_task(0);

        assert!(result.is_err());
    }

    #[test]
    fn test_find_task_with_correct_source() {
        let input1 = Input {
            id: "env1".to_string(),
            options: vec!["dev".to_string()],
            description: None,
            default: None,
        };

        let input2 = Input {
            id: "env2".to_string(),
            options: vec!["prod".to_string()],
            description: None,
            default: None,
        };

        let source1 =
            create_test_tasks_with_inputs(vec![("task1", "echo ${input:env1}")], vec![input1]);

        let source2 =
            create_test_tasks_with_inputs(vec![("task2", "echo ${input:env2}")], vec![input2]);

        let collection = TaskCollection::new(vec![source1, source2]);

        // Find task from source1
        let (source, task) = collection.find_task(0).unwrap();
        assert_eq!(task.label, "task1");
        assert!(source.get_input("env1").is_ok());

        // Find task from source2
        let (source, task) = collection.find_task(1).unwrap();
        assert_eq!(task.label, "task2");
        assert!(source.get_input("env2").is_ok());
    }
}
