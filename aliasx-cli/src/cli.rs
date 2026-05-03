use aliasx_core::{
    aliases, config_generator::ConfigGenerator, history::History, task_collection::TaskCollection, task_filter::TaskFilter, task_reader::TaskFormat, tasks::{self}
};
use aliasx_tui::{fuzzy_finder, task_fuzzy_finder, FuzzyConfig, TuiSession};
use clap::{Args, Parser, Subcommand, ValueEnum};
use indexmap::IndexMap;
use std::{ops::Index, path::PathBuf};

#[derive(ValueEnum, Clone, Debug, Copy)]
enum TaskFilterCli {
    All,
    Local,
    Global,
}

impl From<TaskFilterCli> for TaskFilter {
    fn from(f: TaskFilterCli) -> Self {
        match f {
            TaskFilterCli::All => TaskFilter::All,
            TaskFilterCli::Local => TaskFilter::Local,
            TaskFilterCli::Global => TaskFilter::Global,
        }
    }
}

impl From<TaskFormatCli> for TaskFormat {
    fn from(f: TaskFormatCli) -> Self {
        match f {
            TaskFormatCli::Yaml => TaskFormat::Yaml,
            TaskFormatCli::Json => TaskFormat::Json,
        }
    }
}

#[derive(ValueEnum, Clone, Debug, Copy)]
enum TaskFormatCli {
    Yaml,
    Json,
}

#[derive(Args)]
struct TaskOptions {
    /// the index of alias to handle
    #[arg(short, long)]
    index: Option<usize>,

    /// verbose output
    #[arg(short, long)]
    verbose: bool,

    /// filter which tasks to include
    #[arg(value_enum, short, long, default_value_t = TaskFilterCli::All)]
    filter: TaskFilterCli,

    /// only apply to native aliases
    #[arg(short, long)]
    native: bool,

    /// enable conditions
    #[arg(short, long)]
    conditions: Option<bool>,
}

#[derive(Parser)]
#[command(version, about = "Alias e(x)tended CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// run a task
    #[command(aliases = ["r"])]
    Run {
        #[command(flatten)]
        task_options: TaskOptions,

        /// id of task to run
        #[arg()]
        id: Option<String>,
    },

    /// list all aliases (list)
    #[command(aliases = ["list"])]
    Ls {
        #[command(flatten)]
        task_options: TaskOptions,
    },

    /// use fuzzy finder (f)
    #[command(aliases = ["f"])]
    Fzf {
        /// add query to search
        #[arg(short, long)]
        query: Option<String>,

        #[command(flatten)]
        task_options: TaskOptions,
    },

    /// run validation on configs files
    #[command()]
    Validate {
        #[command(flatten)]
        task_options: TaskOptions,
    },

    /// use history instead of tasks
    #[command(aliases = ["h"])]
    History {
        /// clear entire history
        #[arg(long)]
        clear: bool,

        #[command(flatten)]
        task_options: TaskOptions,
    },

    /// create or convert existing configs
    ConfigGenerator {
        #[command(subcommand)]
        command: ConfigGeneratorSubCommands,
    },
}

#[derive(Subcommand)]
enum ConfigGeneratorSubCommands {
    /// print a minimal example config
    ExampleConfig {
        #[arg(value_enum, short, long, default_value_t = TaskFormatCli::Yaml)]
        format: TaskFormatCli,
    },

    /// convert existing json config to yaml
    JsonToYaml {
        /// path to json config
        #[arg()]
        path: String,
    },

    /// convert existing yaml config to json
    YamlToJson {
        /// path to yaml config
        #[arg()]
        path: String,
    },
}

fn get_tasks(task_options: &TaskOptions) -> anyhow::Result<TaskCollection> {
    let enable_conditions = task_options.conditions.unwrap_or(true);
    let tasks = if task_options.native {
        aliases::get_aliases_as_tasks()?
    } else {
        tasks::get_all_tasks(task_options.filter.into(), enable_conditions)?
    };

    Ok(tasks)
}

pub fn parse_and_run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run(&cli)
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    match &cli.command {
        Some(Commands::Ls { task_options }) => {
            let tasks = get_tasks(&task_options)?;
            if let Some(idx) = task_options.index {
                tasks.list_at(idx, task_options.verbose)?;
            } else {
                tasks.list_all(task_options.verbose)?;
            }
        }

        Some(Commands::Fzf {
            task_options,
            query,
        }) => {
            let tasks = get_tasks(&task_options)?;
            run_fzf_task(&tasks, query.as_deref().unwrap_or(""), task_options.verbose)?;
        }

        Some(Commands::Validate { task_options }) => {
            let tasks = tasks::get_all_tasks(task_options.filter.into(), false)?; // always disable conditions
            if let Some(idx) = task_options.index {
                tasks.validate_at(idx, task_options.verbose)?;
            } else {
                tasks.validate_all(task_options.verbose);
            }
        }

        Some(Commands::History {
            clear,
            task_options,
        }) => {
            if *clear {
                History::clear()?;
                return Ok(());
            }

            let history = History::load_filtered(task_options.filter.into())?;

            let idx = if let Some(idx) = task_options.index {
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
                return Err(anyhow::anyhow!(
                    "history does not contain entry with idx={} (history size={})",
                    idx,
                    history.len()
                ));
            }

            let selected = history.index(idx);
            let name = if task_options.verbose {
                &selected.task_command
            } else {
                &selected.task_name
            };
            TaskCollection::run_command(name, &selected.task_command)?
        }

        Some(Commands::Run { id, task_options }) => {
            let tasks = get_tasks(&task_options)?;
            let itask = match (task_options.index, &id) {
                (Some(idx), None) => tasks.find_itask_from_idx(idx)?,
                (None, Some(id)) => tasks.find_itask_from_id(id)?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "provide either an [ID] or --index, but not both"
                    ))
                }
            };

            let mut session = TuiSession::new()?;
            let input_selections = run_fzf_inputs(&tasks, itask.idx, &mut session)?;
            drop(session);

            tasks.execute(&itask, &input_selections, task_options.verbose)?;
        }

        Some(Commands::ConfigGenerator { command }) => match command {
            ConfigGeneratorSubCommands::ExampleConfig { format } => {
                ConfigGenerator::print_example_config((*format).into())?
            },
            ConfigGeneratorSubCommands::JsonToYaml { path } => {
                ConfigGenerator::convert_json_to_yaml(PathBuf::from(path))?
            }
            ConfigGeneratorSubCommands::YamlToJson { path } => {
                ConfigGenerator::convert_yaml_to_json(PathBuf::from(path))?
            }
        },

        None => {
            let tasks = tasks::get_all_tasks(TaskFilter::All, true)?;
            run_fzf_task(&tasks, "", false)?;
            return Ok(());
        }
    }

    Ok(())
}

fn run_fzf_task(tasks: &TaskCollection, query: &str, verbose: bool) -> anyhow::Result<()> {
    let mut session = TuiSession::new()?;

    let entries = tasks.indexed_tasks();
    let selected_idx = task_fuzzy_finder(&entries, &tasks, &mut session, query, verbose)?;
    let input_selections = run_fzf_inputs(tasks, selected_idx, &mut session)?;

    drop(session);

    let itask = tasks.find_itask_from_idx(selected_idx)?;
    tasks.execute(&itask, &input_selections, verbose)
}

fn run_fzf_inputs(
    tasks: &TaskCollection,
    idx: usize,
    session: &mut TuiSession,
) -> anyhow::Result<IndexMap<String, String>> {
    let inputs = tasks.required_inputs_for_task(idx)?;
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
            session,
        )?;
        selections.insert(input.id.clone(), input.options[sel].clone());
    }

    Ok(selections)
}

// TODO: add mocks to verify calls

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_subcommand() {
        let args = ["aliasx", "-i", "7", "-v"];
        let cli = Cli::try_parse_from(&args);

        assert!(cli.is_err());
    }

    #[test]
    fn test_list_id() {
        let args = ["aliasx", "ls", "--index", "2"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Ls { task_options }) => {
                assert_eq!(task_options.index.unwrap(), 2)
            }
            _ => panic!("wrong subcommand"),
        }
    }

    #[test]
    fn test_list_alias_ls() {
        let args = ["aliasx", "ls"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Ls { task_options }) => {
                assert!(task_options.index.is_none());
                assert!(!task_options.native);
            }
            _ => panic!("wrong subcommand"),
        }
    }

    #[test]
    fn test_fzf_with_query() {
        let args = ["aliasx", "f", "--query", "hello"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Fzf {
                query,
                task_options,
            }) => {
                assert_eq!(query.unwrap(), "hello");
                assert!(!task_options.native);
            }
            _ => panic!("wrong subcommand"),
        }
    }

    #[test]
    fn test_fzf_with_native_aliases() {
        let args = ["aliasx", "fzf", "-n", "--query", "native"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Fzf {
                query,
                task_options,
            }) => {
                assert_eq!(query.unwrap(), "native");
                assert_eq!(task_options.native, true);
            }
            _ => panic!("wrong subcommand"),
        }
    }

    #[test]
    fn test_run_command_with_id() {
        let args = ["aliasx", "run", "build"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Run { id, task_options }) => {
                assert_eq!(id.unwrap(), "build");
                assert!(task_options.index.is_none());
            }
            _ => panic!("wrong subcommand"),
        }
    }

    #[test]
    fn test_run_command_with_idx() {
        let args = ["aliasx", "run", "--index", "1"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Run { id, task_options }) => {
                assert!(id.is_none());
                assert_eq!(task_options.index.unwrap(), 1);
            }
            _ => panic!("wrong subcommand"),
        }
    }

    #[test]
    fn test_run_command_with_no_id_nor_idx() {
        let args = ["aliasx", "run"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match &cli.command {
            Some(Commands::Run { id, task_options }) => {
                assert!(id.is_none());
                assert!(task_options.index.is_none());
            }
            _ => panic!("wrong subcommand"),
        }

        let res = run(&cli);
        assert!(res.is_err());
    }

    #[test]
    fn test_run_command_with_both_id_nor_idx() {
        let args = ["aliasx", "run", "task", "--index", "0"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match &cli.command {
            Some(Commands::Run { id, task_options }) => {
                assert_eq!(id.as_ref().unwrap(), "task");
                assert_eq!(task_options.index.unwrap(), 0);
            }
            _ => panic!("wrong subcommand"),
        }

        let res = run(&cli);
        assert!(res.is_err());
    }
}
