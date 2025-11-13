use clap::{Parser, Subcommand};

use crate::{aliases, tasks};

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
  aliasx -n               (fzf native aliases (.bashrc, .zshrc etc))
  aliasx -n 0 ls -d       (list first native aliases with details)
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// the id of alias to handle
    id: Option<usize>,

    /// only apply to native aliases
    #[arg(short, long)]
    native: bool,
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
    let only_native = &cli.native;

    let tasks = if *only_native {
        aliases::get_aliases_as_tasks()?
    } else {
        tasks::get_all_tasks()?
    };

    match &cli.command {
        Some(Commands::Ls { detailed }) => {
            if id.is_some() {
                tasks.list_at(id.unwrap(), *detailed)?;
            } else {
                tasks.list_all(*detailed)?;
            }
        }

        Some(Commands::Fzf { query }) => {
            tasks.fzf(query.as_deref().unwrap_or(""))?;
        }

        None => {
            if id.is_none() {
                // default to fuzzy finder
                tasks.fzf("")?;
                return Ok(());
            }

            tasks.execute(id.unwrap())?;
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
        assert_eq!(cli.native, false);
    }

    #[test]
    fn test_list_detailed_flag() {
        let args = ["aliasx", "ls", "--detailed"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Ls { detailed }) => assert!(detailed),
            _ => panic!("Expected list command with --detailed flag"),
        }

        assert_eq!(cli.native, false);
    }

    #[test]
    fn test_list_native_detailed_flag() {
        let args = ["aliasx", "--native", "ls", "--detailed"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Ls { detailed }) => assert!(detailed),
            _ => panic!("Expected list command with --detailed flag"),
        }

        assert_eq!(cli.native, true);
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
            Some(Commands::Fzf { query }) => assert_eq!(query, Some("hello".into())),
            _ => panic!("Expected fzf command with query"),
        }

        assert_eq!(cli.native, false);
    }

    #[test]
    fn test_fzf_with_native_aliases() {
        let args = ["aliasx", "-n", "f", "--query", "native"];
        let cli = Cli::try_parse_from(&args).unwrap();

        match cli.command {
            Some(Commands::Fzf { query }) => assert_eq!(query, Some("native".into())),
            _ => panic!("Expected fzf command with query"),
        }

        assert_eq!(cli.native, true);
    }
}
