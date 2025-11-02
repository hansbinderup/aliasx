use clap::{Parser, Subcommand};

use crate::pid;
use crate::tasks;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// the id of alias to run
    id: Option<usize>,
}

#[derive(Subcommand)]
enum Commands {
    /// list all aliases
    List,

    /// parses vscode tasks.json
    Vsc {
        /// optional dir
        #[arg(short, long)]
        dir: Option<String>,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let id = &cli.id;

    match &cli.command {
        Some(Commands::List) => {
            tasks::list_all()?;
        }

        Some(Commands::Vsc { dir }) => {
            let path = dir.as_deref().unwrap_or(".vscode/tasks.json");
            let tasks = tasks::read_tasks_from_file_json(path);
            let output = pid::try_get_file().expect("Could not locate storge");

            match tasks {
                Ok(tasks_json) => {
                    tasks::write_tasks_to_file(&tasks_json, &output)?;
                }
                Err(e) => eprintln!("Error parsing tasks at '{}': {}", path, e),
            }
        }

        None => {
            if id.is_none() {
                // simply do nothing
                return Ok(());
            }

            tasks::execute(id.unwrap())?;
        }
    }

    Ok(())
}
