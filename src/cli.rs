use clap::{Parser, Subcommand};

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
    List {
        /// show all details about the task
        #[arg(short, long)]
        detailed: bool,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let id = &cli.id;

    match &cli.command {
        Some(Commands::List { detailed }) => {
            if id.is_some() {
                tasks::list_at(id.unwrap(), *detailed)?;
            } else {
                tasks::list_all(*detailed)?;
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
