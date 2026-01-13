pub mod aliases;
mod cli;
pub mod input;
pub mod input_mapping;
pub mod task_collection;
pub mod tasks;
pub mod validator;

fn main() {
    cli::run().unwrap_or_else(|err| {
        eprintln!("aliasx: {}", err);
        std::process::exit(1);
    });
}
