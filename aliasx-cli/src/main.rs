use aliasx_cli::cli;

fn main() {
    cli::parse_and_run().unwrap_or_else(|err| {
        eprintln!("aliasx: {}", err);
        std::process::exit(1);
    });
}
