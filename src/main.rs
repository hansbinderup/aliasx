pub mod tasks;
mod cli;

fn main() {
    let res = cli::run();
    match res {
        Ok(_) => return,
        Err(e) => {
            eprintln!("aliasx failed with err: {}", e);

        },
    }
}
