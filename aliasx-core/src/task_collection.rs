use anyhow::{anyhow, Context};
use execute::shell;
use indexmap::{IndexMap, IndexSet};
use std::process::Stdio;

use crate::history::HistoryEntry;

use crate::{
    history::History,
    input::Input,
    tasks::{TaskEntry, Tasks},
    validator::Validator,
};

#[derive(Debug, Default)]
pub struct TaskCollection {
    sources: Vec<Tasks>,
}

#[derive(Debug, Clone, Copy)]
pub struct IndexedTask<'a> {
    pub idx: usize,
    pub source: &'a Tasks,
    pub task: &'a TaskEntry,
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

    /// iterates *ALL* tasks - also duplicated ones
    /// use `indexed_tasks` for deduplication
    fn all_tasks(&self) -> IndexSet<&TaskEntry> {
        self.sources.iter().flat_map(|t| t.tasks.iter()).collect()
    }

    /// iterates *ALL* itasks - also duplicated ones
    /// use `indexed_tasks` for deduplication
    fn all_itasks(&self) -> impl Iterator<Item = IndexedTask<'_>> {
        let mut global_idx = 0;
        self.sources.iter().flat_map(move |source| {
            let start_idx = global_idx;
            global_idx += source.tasks.len();

            source
                .tasks
                .iter()
                .enumerate()
                .map(move |(local_idx, task)| IndexedTask {
                    idx: start_idx + local_idx,
                    source,
                    task,
                })
        })
    }

    pub fn find_itask_from_idx(&self, idx: usize) -> anyhow::Result<IndexedTask<'_>> {
        if let Some(found) = self.indexed_tasks().iter().find(|itask| itask.idx == idx) {
            return Ok(*found);
        }

        Err(anyhow!("Couldn't find task with idx={}", idx))
    }

    pub fn find_itask_from_id(&self, id: &str) -> anyhow::Result<IndexedTask<'_>> {
        if let Some(found) = self
            .indexed_tasks()
            .iter()
            .find(|itask| itask.task.id.as_deref() == Some(id))
        {
            return Ok(*found);
        }

        Err(anyhow!("Couldn't find task with id={}", id))
    }

    /// Returns all itasks as deduplicated.
    /// NOTE: indexSet does not support standard slices so therefore the vector..
    pub fn indexed_tasks(&self) -> Vec<IndexedTask<'_>> {
        let mut seen: IndexSet<&TaskEntry> = IndexSet::new();
        self.all_itasks()
            .filter(|itask| seen.insert(itask.task))
            .enumerate()
            .map(|(idx, itask)| IndexedTask {
                idx,
                task: itask.task,
                source: itask.source,
            })
            .collect()
    }

    /// Returns the inputs required to execute task `idx` (direct + via mappings).
    pub fn required_inputs_for_task(&self, idx: usize) -> anyhow::Result<Vec<&Input>> {
        let itask = self.find_itask_from_idx(idx)?;

        return itask
            .source
            .required_inputs_for_command(&itask.task.command);
    }

    pub fn validate_all(&self, verbose: bool) {
        let validator = Validator { verbose };
        let mut task_reports = Vec::new();

        for itask in self.all_itasks() {
            let report = validator.validate_task_command(itask.task, itask.source);
            task_reports.push(report);
        }

        let history_report = validator.validate_history();

        Validator::print_header();

        validator.print_report(&task_reports);
        validator.print_single_report(&history_report);

        Validator::print_summary(
            task_reports
                .into_iter()
                .chain(std::iter::once(history_report)),
        );
    }

    pub fn validate_at(&self, idx: usize, verbose: bool) -> anyhow::Result<()> {
        let validator = Validator { verbose };
        let itask = self.find_itask_from_idx(idx)?;

        let report = validator.validate_task_command(itask.task, itask.source);

        // for single tasks we need to print a verbose report
        // otherwise there might be no output
        report.print(true);

        Ok(())
    }

    pub fn list_at(&self, idx: usize, verbose: bool) -> anyhow::Result<()> {
        let itask = self.find_itask_from_idx(idx)?;
        itask.task.print(idx, verbose, self.width_idx());

        Ok(())
    }

    pub fn list_all(&self, verbose: bool) -> anyhow::Result<()> {
        for (idx, task) in self.all_tasks().iter().enumerate() {
            task.print(idx, verbose, self.width_idx());
        }

        Ok(())
    }

    /// Execute task `idx` with pre-collected `input_selections`.
    pub fn execute(
        &self,
        itask: &IndexedTask,
        input_selections: &IndexMap<String, String>,
        verbose: bool,
    ) -> anyhow::Result<()> {
        let mut task_command = itask.task.command.clone();

        for input_id in Input::extract_variables(&itask.task.command) {
            let val = input_selections
                .get(&input_id)
                .ok_or_else(|| anyhow!("no selection provided for input '{}'", input_id))?;
            task_command = Input::replace_next_variable(&task_command, val);
        }

        task_command = itask
            .source
            .apply_mappings(&task_command, input_selections)?;

        let res = Self::run_command(&itask.task.format(verbose), &task_command);

        let entry = HistoryEntry::new(
            &itask.task.label,
            &itask.task.command,
            if res.is_ok() { 0 } else { 1 },
            itask.source.scope,
        );

        if let Err(err) = History::append(&entry) {
            if verbose {
                println!("Error while adding history: {}", err.to_string());
            }
        }

        res
    }

    pub fn run_command(label: &str, task_command: &str) -> anyhow::Result<()> {
        println!("aliasx | {}\n", label);

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

    fn create_test_task(label: &str, command: &str, id: Option<String>) -> TaskEntry {
        TaskEntry {
            label: label.to_string(),
            command: command.to_string(),
            id: id,
            conditions: Option::None,
        }
    }

    fn create_test_tasks(entries: Vec<(&str, &str)>) -> Tasks {
        let mut tasks = Tasks::default();
        for (label, command) in entries {
            tasks
                .tasks
                .insert(create_test_task(label, command, Option::None));
        }
        tasks
    }

    fn create_test_tasks_with_inputs(entries: Vec<(&str, &str)>, inputs: Vec<Input>) -> Tasks {
        let mut tasks = create_test_tasks(entries);
        tasks.inputs = inputs;
        tasks
    }

    fn create_test_tasks_with_ids(entries: Vec<(&str, &str, &str)>) -> Tasks {
        let mut tasks = Tasks::default();
        for (label, command, id) in entries {
            tasks
                .tasks
                .insert(create_test_task(label, command, Some(id.to_string())));
        }
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
            .all_itasks()
            .map(|itask| (itask.idx, itask.task.label.clone()))
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
            .all_itasks()
            .map(|itask| (itask.idx, itask.task.label.clone()))
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
        let itask = collection.find_itask_from_idx(1).unwrap();

        assert_eq!(itask.task.label, "task2");
    }

    #[test]
    fn test_find_task_second_source() {
        let source1 = create_test_tasks(vec![("task1", "echo 1"), ("task2", "echo 2")]);
        let source2 = create_test_tasks(vec![("task3", "echo 3")]);

        let collection = TaskCollection::new(vec![source1, source2]);
        let itask = collection.find_itask_from_idx(2).unwrap();

        assert_eq!(itask.task.label, "task3");
    }

    #[test]
    fn test_find_task_boundary() {
        let source1 = create_test_tasks(vec![("task1", "echo 1"), ("task2", "echo 2")]);
        let source2 = create_test_tasks(vec![("task3", "echo 3"), ("task4", "echo 4")]);

        let collection = TaskCollection::new(vec![source1, source2]);

        // Test boundary between sources
        let itask = collection.find_itask_from_idx(1).unwrap();
        assert_eq!(itask.task.label, "task2");

        let itask = collection.find_itask_from_idx(2).unwrap();
        assert_eq!(itask.task.label, "task3");
    }

    #[test]
    fn test_find_task_invalid_idx() {
        let source1 = create_test_tasks(vec![("task1", "echo 1")]);

        let collection = TaskCollection::new(vec![source1]);
        let result = collection.find_itask_from_idx(5);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Couldn't find task with idx="));
    }

    #[test]
    fn test_find_task_from_id() {
        let source1 = create_test_tasks_with_ids(vec![("task1", "command", "some-id")]);

        let collection = TaskCollection::new(vec![source1]);
        let result = collection.find_itask_from_id("some-id").unwrap();

        assert_eq!(result.idx, 0);
        assert_eq!(result.task.label, "task1");
        assert_eq!(result.task.command, "command");
    }

    #[test]
    fn test_find_task_from_id_duplicate_id() {
        let source1 = create_test_tasks_with_ids(vec![
            ("task1", "command", "some-id"),
            ("task2", "command2", "some-id"),
        ]);

        let collection = TaskCollection::new(vec![source1]);
        let result = collection.find_itask_from_id("some-id").unwrap();

        // will find the first one
        assert_eq!(result.idx, 0);
        assert_eq!(result.task.label, "task1");
        assert_eq!(result.task.command, "command");
    }

    #[test]
    fn test_find_task_from_id_invalid() {
        let source1 = create_test_tasks_with_ids(vec![
            ("task1", "command", "some-id"),
            ("task2", "command2", "some-id"),
        ]);

        let collection = TaskCollection::new(vec![source1]);
        let result = collection.find_itask_from_id("invalid-id");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Couldn't find task with id="));
    }

    #[test]
    fn test_find_task_empty_collection() {
        let collection = TaskCollection::new(vec![]);
        let result = collection.find_itask_from_idx(0);

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
        let itask = collection.find_itask_from_idx(0).unwrap();
        assert_eq!(itask.task.label, "task1");
        assert!(itask.source.get_input("env1").is_ok());

        // Find task from source2
        let itask = collection.find_itask_from_idx(1).unwrap();
        assert_eq!(itask.task.label, "task2");
        assert!(itask.source.get_input("env2").is_ok());
    }

    #[test]
    fn test_task_idx_deduplication() {
        let source1 = create_test_tasks(vec![("task1", "echo1"), ("task2", "echo2")]);
        let source2 = create_test_tasks(vec![("task1", "echo1"), ("task3", "echo3")]);
        let source3 = create_test_tasks(vec![("task2", "echo2"), ("task3", "echo3")]);
        let source4 = create_test_tasks(vec![("task4", "echo4"), ("task5", "echo5")]);

        let collection = TaskCollection::new(vec![source1, source2, source3, source4]);

        let all_itasks: Vec<IndexedTask<'_>> = collection.all_itasks().collect();
        let deduplicated_itasks = collection.indexed_tasks();

        assert_eq!(all_itasks.len(), 8); // should always return *ALL*
        assert_eq!(deduplicated_itasks.len(), 5); // should only return unique and indexed in order
                                                  // of added

        assert_eq!(
            collection.find_itask_from_idx(0).unwrap().task.label,
            "task1"
        );
        assert_eq!(
            collection.find_itask_from_idx(1).unwrap().task.label,
            "task2"
        );
        assert_eq!(
            collection.find_itask_from_idx(2).unwrap().task.label,
            "task3"
        );
        assert_eq!(
            collection.find_itask_from_idx(3).unwrap().task.label,
            "task4"
        );
        assert_eq!(
            collection.find_itask_from_idx(4).unwrap().task.label,
            "task5"
        );

        assert!(collection.find_itask_from_idx(5).is_err());
    }
}
