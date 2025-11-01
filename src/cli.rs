use clap::{Parser, Subcommand};

use crate::aliases;
use crate::pid;
use crate::vscode_tasks_parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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

pub fn run() -> std::io::Result<()> {
    let cli = Cli::parse();
    let output = pid::try_get_file().expect("UPDATE ME");

    match &cli.command {
        Some(Commands::List) => {
            let output = pid::try_get_file().expect("Could not locate storge");
            let _ = aliases::list_all(&output)?;
        }

        Some(Commands::Clear) => {
            let _ = aliases::clear(&output)?;
        }

        Some(Commands::Vsc { dir }) => {
            let path = dir.as_deref().unwrap_or(".vscode/tasks.json");
            let tasks = vscode_tasks_parser::read_tasks_from_file(path);
            let output = pid::try_get_file().expect("Could not locate storge");

            match tasks {
                Ok(tasks_json) => {
                    aliases::generate_from_tasks(&tasks_json, &output)?;
                }
                Err(e) => eprintln!("Error parsing tasks at '{}': {}", path, e),
            }
        }

        None => {}
    }

    Ok(())
}
