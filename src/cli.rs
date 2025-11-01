use anyhow::anyhow;
use clap::{Parser, Subcommand};

use crate::pid;
use crate::tasks;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// the given alias
    alias: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// list all aliases
    List,

    /// clear all aliases
    Clear,

    /// parses vscode tasks.json
    Vsc {
        /// optional dir
        #[arg(short, long)]
        dir: Option<String>,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let output = pid::try_get_file().expect("UPDATE ME");
    let alias = &cli.alias;

    match &cli.command {
        Some(Commands::List) => {
            let output = pid::try_get_file().expect("Could not locate storge");
            let _ = tasks::list_all(&output)?;
        }

        Some(Commands::Clear) => {
            let _ = tasks::clear(&output)?;
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
            if alias.is_none() {
                return Err(anyhow!("invalid input"));
            }

            let _output = pid::try_get_file().expect("Could not locate storge");
            // let res = aliases::try_find_alias(&output, alias.as_ref().unwrap())?;
            // match res {
            //     Some((label, cmd)) => {
            //         println!("{} -> {}", label, cmd);
            //     }
            //     None => return Err(anyhow!("found no alias")),
            // }
        }
    }

    Ok(())
}
