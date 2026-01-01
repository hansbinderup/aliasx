use clap::{Parser, Subcommand};

use crate::{
    aliases,
    tasks::{self, TaskFilter},
};

#[derive(Parser)]
#[command(
    version,
    about = "Alias e(x)tended CLI",
    long_about = "Alias e(x)tended CLI

Examples:
  aliasx                    (default to fzf)
  aliasx ls                 (list aliases)
  aliasx fzf -q query       (fzf with query as search)
  aliasx -i 0               (execute alias 0)
  aliasx -n                 (fzf native aliases (.bashrc, .zshrc etc))
  aliasx -n -v --id 0 ls    (list first native aliases verbosely)
  aliasx -f local ls        (filter local aliases only)
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// the id of alias to handle
    #[arg(short, long)]
    id: Option<usize>,

    /// only apply to native aliases
    #[arg(short, long)]
    native: bool,

    /// verbose output
    #[arg(short, long)]
    verbose: bool,

    /// filter which tasks to include
    #[arg(short, long, default_value_t = TaskFilter::All)]
    filter: TaskFilter,
}

#[derive(Subcommand)]
enum Commands {
    /// list all aliases (list)
    #[command(aliases = ["list"])]
    Ls,

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

    let tasks = if cli.native {
        aliases::get_aliases_as_tasks()?
    } else {
        tasks::get_all_tasks(cli.filter)?
    };

    match &cli.command {
        Some(Commands::Ls) => {
            if id.is_some() {
                tasks.list_at(id.unwrap(), cli.verbose)?;
            } else {
                tasks.list_all(cli.verbose)?;
            }
        }

        Some(Commands::Fzf { query }) => {
            tasks.fzf(query.as_deref().unwrap_or(""), cli.verbose)?;
        }

        None => {
            if id.is_none() {
                // default to fuzzy finder
                tasks.fzf("", cli.verbose)?;
                return Ok(());
            }

            tasks.execute(id.unwrap(), cli.verbose)?;
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
        let args = ["aliasx", "-i", "7"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(cli.command.is_none());
        assert_eq!(cli.id, Some(7));
    }

    #[test]
    fn test_list_id() {
        let args = ["aliasx", "--id", "2", "ls"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(matches!(cli.command, Some(Commands::Ls { .. })));
        assert_eq!(cli.id, Some(2));
        assert_eq!(cli.native, false);
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

    #[test]
    fn test_verbose_flag() {
        let args = ["aliasx", "-v"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(cli.verbose, "Expected verbose flag to be true");
    }

}
