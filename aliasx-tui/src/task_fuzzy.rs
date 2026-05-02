use aliasx_core::task_collection::{IndexedTask, TaskCollection};
use aliasx_core::task_filter::TaskFilter;
use anyhow::Result;

use crate::{
    fuzzy::{FuzzyConfig, FuzzyList},
    fuzzy_finder, TuiSession,
};

struct TaskFuzzyItem {
    original_idx: usize,
    id_prefix: String,
    task_label: String,
    detail: String,
    scope_key: Option<String>,
}

impl FuzzyList for TaskFuzzyItem {
    fn label(&self) -> &str {
        &self.task_label
    }

    fn label_prefix(&self) -> Option<&str> {
        Some(&self.id_prefix)
    }

    fn detail(&self) -> Option<String> {
        Some(self.detail.clone())
    }

    fn match_filter(&self, filter: &str) -> bool {
        if let Some(key) = &self.scope_key {
            filter.eq(key) || filter.eq("all")
        } else {
            true
        }
    }
}

pub fn task_fuzzy_finder(
    itasks: &[IndexedTask<'_>],
    collection: &TaskCollection,
    session: &mut TuiSession,
    query: &str,
    verbose: bool,
) -> Result<usize> {
    let width = collection.width_idx();

    let items: Vec<TaskFuzzyItem> = itasks
        .iter()
        .map(|t| TaskFuzzyItem {
            original_idx: t.idx,
            id_prefix: format!("{:0>width$} ", t.idx),
            task_label: t.task.label.clone(),
            detail: format!("{}{}", t.task.id.clone().unwrap_or("".to_string()), t.task.command.clone()),
            scope_key: Some(t.source.scope.to_string()),
        })
        .collect();

    let sel = fuzzy_finder(
        &items,
        "Search",
        FuzzyConfig {
            show_details: verbose,
            filters: vec![
                TaskFilter::All.to_string(),
                TaskFilter::Local.to_string(),
                TaskFilter::Global.to_string(),
            ],
            initial_query: query.to_string(),
            ..FuzzyConfig::default()
        },
        session,
    )?;

    Ok(items[sel].original_idx)
}
