use anyhow::{anyhow, ensure, Context};
use execute::shell;
use fuzzy_select::FuzzySelect;
use std::process::Stdio;

use crate::input::Input;
use crate::tasks::{TaskEntry, Tasks};

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

    fn width_idx(&self) -> usize {
        self.total_count().to_string().len()
    }

    fn all_tasks(&self) -> impl Iterator<Item = &TaskEntry> {
        self.sources.iter().flat_map(|t| t.tasks.iter())
    }

    fn all_tasks_with_source(&self) -> impl Iterator<Item = (usize, &Tasks, &TaskEntry)> {
        let mut global_idx = 0;
        self.sources.iter().flat_map(move |source| {
            let start_idx = global_idx;
            global_idx += source.tasks.len();
            
            source.tasks.iter().enumerate().map(move |(local_idx, task)| {
                (start_idx + local_idx, source, task)
            })
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

    pub fn validate_all(&self, verbose: bool) {
        let mut failed = 0;
        let mut total = 0;
        let width_idx = self.width_idx();

        for (idx, source, task) in self.all_tasks_with_source() {
            if !source.validate_config(&task, idx, width_idx, verbose) {
                failed += 1;
            }

            total += 1;
        }

        if failed == 0 {
            println!("✅ All {} tasks validated successfully!", total);
        } else {
            println!(
                "❌ Validation failed for {} out of {} tasks!",
                failed, total
            );
        }
    }

    pub fn validate_at(&self, idx: usize, verbose: bool) -> anyhow::Result<()> {
        let (source, task) = self.find_task(idx)?;
        let success = source.validate_config(&task, idx, self.width_idx(), verbose);

        if success {
            println!("✅ validation was successful");
        } else {
            println!("❌ validation failed");
        }

        Ok(())
    }

    pub fn list_at(&self, id: usize, verbose: bool) -> anyhow::Result<()> {
        let (_, task) = self.find_task(id)?;
        task.print(id, verbose, self.width_idx());

        Ok(())
    }

    pub fn list_all(&self, verbose: bool) -> anyhow::Result<()> {
        for (idx, task) in self.all_tasks().enumerate() {
            task.print(idx, verbose, self.width_idx());
        }

        Ok(())
    }

    pub fn fzf(&self, query: &str, verbose: bool) -> anyhow::Result<()> {
        let task_strings: Vec<String> = self
            .all_tasks()
            .enumerate()
            .map(|(i, task)| {
                format!(
                    "[{:0>width$}] {}",
                    i,
                    task.format(verbose),
                    width = self.width_idx()
                )
            })
            .collect();

        let selection = FuzzySelect::new()
            .with_prompt("Search:")
            .with_query(query)
            .with_options(task_strings.clone())
            .select()?;

        let id = task_strings
            .iter()
            .position(|s| s == &selection)
            .ok_or_else(|| anyhow!("Selected task not found"))?;

        self.execute(id, verbose)?;

        Ok(())
    }

    pub fn execute(&self, id: usize, verbose: bool) -> anyhow::Result<()> {
        let (task_set, task) = self.find_task(id)?;
        let inputs = Input::extract_variables(&task.command);

        let mut task_command = task.command.clone();

        for (var_type, var_id) in inputs.iter() {
            ensure!(
                var_type == "input",
                "{} variable is not yet supported",
                var_type
            );

            let input = task_set.get_input(var_id)?;
            let selected_input = input.fzf()?;

            task_command = Input::replace_next_variable(&task_command, &selected_input);
        }

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
