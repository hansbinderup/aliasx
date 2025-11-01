pub mod aliases;
mod cli;
mod pid;
pub mod vscode_tasks;

fn main() {
    if !pid::is_pid_set() {
        eprintln!(
            "Environment variable `ALIASX_PID` is not set.\n\
    This is required for session-scoped aliasx functionality.\n\
    To fix, add the following line to your shell configuration (~/.bashrc or ~/.zshrc):\n\
        export ALIASX_PID=$$\n\
    Then start a new shell session or run `source ~/.bashrc`."
        );

        return;
    }

    cli::cli::run();
}
