use clap::{Parser, Subcommand};

use crate::aliases::generate_aliases_file;
use crate::vscode_tasks;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// parses vscode tasks.json
    VsCodeTasks {
        /// optional dir
        #[arg(short, long)]
        dir: Option<String>,
    },
}

pub fn run() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::VsCodeTasks { dir }) => {
            let path = dir.as_deref().unwrap_or(".vscode/tasks.json");
            let tasks = vscode_tasks::parser::read_tasks_from_file(path);

            match tasks {
                Ok(tasks_json) => {
                    // Iterate over each task
                    for task in &tasks_json.tasks {
                        println!("Label: {}", task.label);
                        match &task.command {
                            Some(cmd) => println!("Command: {}", cmd),
                            None => println!("Command: None"),
                        }
                        println!("---");
                    }

                    let _ = generate_aliases_file(&tasks_json, "tasks.aliasx");
                }
                Err(e) => eprintln!("Error parsing tasks at '{}': {}", path, e),
            }
        }
        None => {}
    }
}
