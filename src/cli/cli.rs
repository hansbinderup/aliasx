use clap::{Parser, Subcommand};

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
                    for task in tasks_json.tasks {
                        println!("Label: {}", task.label);
                        match &task.command {
                            Some(cmd) => println!("Command: {}", cmd),
                            None => println!("Command: None"),
                        }
                        println!("---");
                    }
                }
                Err(e) => eprintln!("Failed to parse tasks: {}", e),
            }
        }
        None => {}
    }
}
