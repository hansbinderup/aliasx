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

    /// use fuzzy finder
    Fzf {
        /// add query to search
        #[arg(short, long)]
        query: Option<String>,
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

        Some(Commands::Fzf { query }) => {
            tasks::fzf_task(query.as_deref().unwrap_or(""))?;
        }

        None => {
            if id.is_none() {
                // default to fuzzy finder
                tasks::fzf_task("")?;
                return Ok(());
            }

            tasks::execute(id.unwrap())?;
        }
    }

    Ok(())
}
