use clap::{Parser, Subcommand};

use crate::tasks;

#[derive(Parser)]
#[command(
    version,
    about = "Alias e(x)tended CLI",
    long_about = "Alias e(x)tended CLI

Examples:
  aliasx                  (default to fzf)
  aliasx ls --detailed    (list aliases with details)
  aliasx fzf -q query     (fzf with query as search)
  aliasx 0                (execute alias 0)
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// the id of alias to handle
    id: Option<usize>,
}

#[derive(Subcommand)]
enum Commands {
    /// list all aliases (list)
    #[command(aliases = ["list"])]
    Ls {
        /// show all details about the task
        #[arg(short, long)]
        detailed: bool,
    },

    /// use fuzzy finder (f)
    #[command(aliases = ["f"])]
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
        Some(Commands::Ls { detailed }) => {
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

// TODO: add mocks to verify calls

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_only() {
        let args = ["aliasx", "7"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(cli.command.is_none());
        assert_eq!(cli.id, Some(7));
    }

    #[test]
    fn test_list_id() {
        let args = ["aliasx", "2", "ls"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(matches!(cli.command, Some(Commands::Ls { .. })));
        assert_eq!(cli.id, Some(2));
    }

    #[test]
    fn test_list_detailed_flag() {
        let args = ["aliasx", "ls", "--detailed"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Ls { detailed }) => assert!(detailed),
            _ => panic!("Expected list command with --detailed flag"),
        }
    }

    #[test]
    fn test_list_alias_ls() {
        let args = ["aliasx", "ls"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(matches!(cli.command, Some(Commands::Ls { .. })));
    }

    #[test]
    fn test_fzf_with_query() {
        let args = ["aliasx", "f", "--query", "hello"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Fzf { query }) => assert_eq!(query, Some("hello".into())),
            _ => panic!("Expected fzf command with query"),
        }
    }
}
