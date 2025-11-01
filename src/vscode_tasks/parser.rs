use serde::{Deserialize, Serialize};
use serde_json5::Result;

#[derive(Debug, Serialize,Deserialize)]
pub struct Task {
    pub label: String,
    pub command: Option<String>,
}

#[derive(Debug,Serialize,  Deserialize)]
pub struct TasksJson {
    pub version: Option<String>,
    pub tasks: Vec<Task>,
}

pub fn parse_tasks_json(data: &str) -> Result<TasksJson> {
    let tasks: TasksJson = serde_json5::from_str(data)?;
    return Ok(tasks);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_tasks_json() {
        let data = r#"
        {
            "version": "2.0.0",
            "tasks": [
                {
                    "label": "label0",
                    "command": "command0"
                },
                {
                    "label": "label1",
                    "command": "command1"
                },
                {
                    "label": "label2",
                    "command": "command2"
                },
            ],
        }"#;

        let tasks = parse_tasks_json(data).unwrap();

        // Assertions
        assert_eq!(tasks.version.unwrap(), "2.0.0");
        assert_eq!(tasks.tasks.len(), 3);
        assert_eq!(tasks.tasks[0].label, "label0");
        assert_eq!(tasks.tasks[0].command.as_deref(), Some("command0"));
        assert_eq!(tasks.tasks[2].label, "label2");
        assert_eq!(tasks.tasks[2].command.as_deref(), Some("command2"));
    }
}
