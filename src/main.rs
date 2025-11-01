mod vscode_tasks; // include the folder as a module

fn main() {
    let tasks = vscode_tasks::parser::parse_tasks_json("nove");

    match tasks {
        Ok(tasks_json) => {
            // Iterate over each task
            for task in tasks_json.tasks {
                println!("Label: {}", task.label);
                match &task.command {
                    Some(cmd) => println!("Command: {}", cmd),
                    None => println!("Command: None"),
                }
                println!("---");
            }
        }
        Err(e) => eprintln!("Failed to parse tasks: {}", e),
    }
}
