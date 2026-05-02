pub mod cli;

pub fn parse_and_run() -> anyhow::Result<()> {
    cli::parse_and_run()
}
