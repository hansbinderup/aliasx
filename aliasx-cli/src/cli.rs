use std::{ops::Index, str::FromStr};

use clap::{Parser, Subcommand};
use indexmap::IndexMap;

use aliasx_core::{
    aliases,
    history::History,
    task_collection::TaskCollection,
    task_filter::TaskFilter,
    tasks::{self, TaskEntry},
};
use aliasx_tui::{fuzzy_finder, task_fuzzy_finder, FuzzyConfig, TuiSession};

#[derive(Parser)]
#[command(
    version,
    about = "Alias e(x)tended CLI",
    long_about = "Alias e(x)tended CLI
Examples:
  aliasx                    (default to fzf)
  aliasx ls                 (list aliases)
  aliasx fzf -q query       (fzf with query as search)
  aliasx --index 0          (execute alias 0)
  aliasx -n                 (fzf native aliases (.bashrc, .zshrc etc))
  aliasx -n -v -i 0 ls      (list first native aliases verbosely)
  aliasx ls -f local        (filter local aliases only)
  aliasx validate -v        (validates all configs verbosely)
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// the index of alias to handle
    #[arg(short, long, global = true)]
    index: Option<usize>,

    /// verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// filter which tasks to include
    #[arg(short, long, global = true, value_parser = TaskFilter::from_str, default_value_t = TaskFilter::All)]
    filter: TaskFilter,

    /// only apply to native aliases
    #[arg(short, long, global = true)]
    native: bool,

    /// enable conditions
    #[arg(short, long, global = true)]
    conditions: Option<bool>,
}

#[derive(Subcommand)]
enum Commands {
    /// list all aliases (list)
    #[command(aliases = ["list"])]
    Ls,

    /// use fuzzy finder (f)
    #[command(aliases = ["f"])]
    Fzf {
        /// add query to search
        #[arg(short, long)]
        query: Option<String>,
    },

    /// run validation on configs files
    #[command()]
    Validate,

    /// use history instead of tasks
    #[command(aliases = ["h"])]
    History {
        /// clear entire history
        #[arg(long)]
        clear: bool,
    },
}

fn get_tasks(cli: &Cli) -> anyhow::Result<TaskCollection> {
    let enable_conditions = cli.conditions.unwrap_or(true);

    let tasks = if cli.native {
        aliases::get_aliases_as_tasks()?
    } else {
        tasks::get_all_tasks(cli.filter, enable_conditions)?
    };

    Ok(tasks)
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Ls { }) => {
            let tasks = get_tasks(&cli)?;
            if let Some(idx) = cli.index {
                tasks.list_at(idx, cli.verbose)?;
            } else {
                tasks.list_all(cli.verbose)?;
            }
        }

        Some(Commands::Fzf { query }) => {
            let tasks = get_tasks(&cli)?;
            run_fzf(&tasks, query.as_deref().unwrap_or(""), cli.verbose)?;
        }

        Some(Commands::Validate {  }) => {
            let tasks = tasks::get_all_tasks(cli.filter, false)?; // always disable conditions
            if let Some(idx) = cli.index {
                tasks.validate_at(idx, cli.verbose)?;
            } else {
                tasks.validate_all(cli.verbose);
            }
        }

        Some(Commands::History {  clear }) => {
            if *clear {
                History::clear()?;
                return Ok(());
            }

            let history = History::load_filtered(cli.filter)?;

            let idx = if let Some(idx) = cli.index {
                idx
            } else {
                let mut session = TuiSession::new()?;
                let selected_idx = fuzzy_finder(
                    &history,
                    "History",
                    FuzzyConfig {
                        show_details: true,
                        ..Default::default()
                    },
                    &mut session,
                )?;
                drop(session);
                selected_idx
            };

            if idx >= history.len() {
                return Err(anyhow::anyhow!("history does not contain entry with idx={} (history size={})", idx, history.len()));
            }

            let selected = history.index(idx);
            let name = if cli.verbose {
                &selected.task_command
            } else {
                &selected.task_name
            };
            TaskCollection::run_command(name, &selected.task_command)?
        }

        None => {
            let tasks = get_tasks(&cli)?;
            if cli.index.is_none() {
                run_fzf(&tasks, "", cli.verbose)?;
                return Ok(());
            }

            let idx = cli.index.unwrap();
            let mut selections = IndexMap::new();
            let inputs = tasks.required_inputs_for_task(idx)?;
            if !inputs.is_empty() {
                let mut session = TuiSession::new()?;
                for input in inputs {
                    let prompt = format!(
                        "input | {}:",
                        input.description.as_deref().unwrap_or(&input.id)
                    );
                    let sel = fuzzy_finder(
                        &input.options,
                        &prompt,
                        FuzzyConfig::default(),
                        &mut session,
                    )?;
                    selections.insert(input.id.clone(), input.options[sel].clone());
                }
                drop(session);
            }

            tasks.execute(idx, selections, cli.verbose)?;
        }
    }

    Ok(())
}

fn run_fzf(tasks: &TaskCollection, query: &str, verbose: bool) -> anyhow::Result<()> {
    let indexed = tasks.indexed_tasks();
    let entries: Vec<(usize, TaskFilter, &TaskEntry)> = indexed
        .iter()
        .map(|(id, scope, t)| (*id, *scope, *t))
        .collect();

    let mut session = TuiSession::new()?;
    let selected_id = task_fuzzy_finder(&entries, &tasks, &mut session, query, verbose)?;

    let inputs = tasks.required_inputs_for_task(selected_id)?;
    let mut selections = IndexMap::new();

    for input in inputs {
        let prompt = format!(
            "input | {}:",
            input.description.as_deref().unwrap_or(&input.id)
        );
        let sel = fuzzy_finder(
            &input.options,
            &prompt,
            FuzzyConfig {
                has_details: false,
                initial_position: input.get_default_selection(),
                ..Default::default()
            },
            &mut session,
        )?;
        selections.insert(input.id.clone(), input.options[sel].clone());
    }

    drop(session);
    tasks.execute(selected_id, selections, verbose)
}

// TODO: add mocks to verify calls

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_only() {
        let args = ["aliasx", "-i", "7"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(cli.command.is_none());
        assert_eq!(cli.index, Some(7));
    }

    #[test]
    fn test_list_id() {
        let args = ["aliasx", "--index", "2", "ls"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(matches!(cli.command, Some(Commands::Ls { .. })));
        assert_eq!(cli.index, Some(2));
        assert_eq!(cli.native, false);
    }

    #[test]
    fn test_list_alias_ls() {
        let args = ["aliasx", "ls"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(matches!(cli.command, Some(Commands::Ls { .. })));
        assert_eq!(cli.native, false);
    }

    #[test]
    fn test_fzf_with_query() {
        let args = ["aliasx", "f", "--query", "hello"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Fzf { query, .. }) => assert_eq!(query, Some("hello".into())),
            _ => panic!("Expected fzf command with query"),
        }

        assert_eq!(cli.native, false);
    }

    #[test]
    fn test_fzf_with_native_aliases() {
        let args = ["aliasx", "-n", "f", "--query", "native"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Fzf { query, .. }) => assert_eq!(query, Some("native".into())),
            _ => panic!("Expected fzf command with query"),
        }

        assert_eq!(cli.native, true);
    }

    #[test]
    fn test_verbose_flag() {
        let args = ["aliasx", "-v"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(cli.verbose, "Expected verbose flag to be true");
    }
}
