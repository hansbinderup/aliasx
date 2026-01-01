pub mod aliases;
mod cli;
pub mod tasks;

fn main() {
    cli::run().unwrap_or_else(|err| {
        eprintln!("aliasx: {}", err);
        std::process::exit(1);
    });
}
