use std::str::FromStr;

use clap::{Parser, Subcommand};

use aliasx_core::{
    aliases, task_filter::TaskFilter, tasks::{self}
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
  aliasx --index 0          (execute alias 0)
  aliasx -n                 (fzf native aliases (.bashrc, .zshrc etc))
  aliasx -n -v -i 0 ls      (list first native aliases verbosely)
  aliasx -f local ls        (filter local aliases only)
  aliasx -v validate        (validates all configs verbosely)
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// the index of alias to handle
    #[arg(short, long)]
    index: Option<usize>,

    /// only apply to native aliases
    #[arg(short, long)]
    native: bool,

    /// verbose output
    #[arg(short, long)]
    verbose: bool,

    /// filter which tasks to include
    #[arg(short, long, value_parser = TaskFilter::from_str, default_value_t = TaskFilter::All)]
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

    /// run validation on configs files
    #[command()]
    Validate,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let index = &cli.index;

    let tasks = if cli.native {
        aliases::get_aliases_as_tasks()?
    } else {
        tasks::get_all_tasks(cli.filter)?
    };

    match &cli.command {
        Some(Commands::Ls) => {
            if let Some(idx) = cli.index {
                tasks.list_at(idx, cli.verbose)?;
            } else {
                tasks.list_all(cli.verbose)?;
            }
        }

        Some(Commands::Fzf { query }) => {
            tasks.fzf(query.as_deref().unwrap_or(""), cli.verbose)?;
        }

        Some(Commands::Validate) => {
            if let Some(idx) = cli.index {
                tasks.validate_at(idx, cli.verbose)?;
            } else {
                tasks.validate_all(cli.verbose);
            }
        }

        None => {
            if index.is_none() {
                tasks.fzf("", cli.verbose)?;
                return Ok(());
            }

            tasks.execute(index.unwrap(), cli.verbose)?;
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
        assert_eq!(cli.index, Some(7));
    }

    #[test]
    fn test_list_id() {
        let args = ["aliasx", "--index", "2", "ls"];
        let cli = Cli::try_parse_from(&args).unwrap();

        assert!(matches!(cli.command, Some(Commands::Ls { .. })));
        assert_eq!(cli.index, Some(2));
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
