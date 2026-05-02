use std::path::Path;

use crate::{
    input::Input,
    input_mapping::InputMapping,
    tasks::{TaskEntry, Tasks},
};
use indexmap::IndexMap;

pub struct ConfigGenerator;

impl ConfigGenerator {
    fn create_example_config() -> Tasks {
        let mut tasks = Tasks::default();

        tasks.version = Some("1.0.0".to_string());

        tasks.tasks.insert(TaskEntry {
            id: None,
            label: "Simple task".to_string(),
            command: "echo 'This is just a simple task'".to_string(),
            conditions: None,
        });

        tasks.tasks.insert(TaskEntry {
            id: Some("build".to_string()),
            label: "Perform build".to_string(),
            command: "echo 'building ${input:build-type} in ${mapping:build-dir}...'".to_string(),
            conditions: None,
        });

        tasks.inputs.push(Input {
            id: "build-type".to_string(),
            options: vec![
                "release".to_string(),
                "debug".to_string(),
                "test".to_string(),
            ],
            default: None,
            description: Some("Pick a build type".to_string()),
        });

        tasks.mappings.push(InputMapping {
            id: "build-dir".to_string(),
            input: "build-type".to_string(),
            options: IndexMap::from([
                ("release".to_string(), ".build-release".to_string()),
                ("debug".to_string(), ".build-debug".to_string()),
                ("test".to_string(), ".build-test".to_string()),
            ]),
        });

        tasks
    }

    pub fn print_example_config() -> anyhow::Result<()> {
        let config = ConfigGenerator::create_example_config();
        let config_str = serde_yaml::to_string(&config)?;

        print!("{}", config_str);

        Ok(())
    }

    pub fn convert_json_to_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let tasks : Tasks = serde_json5::from_reader(reader)?;

        let yaml_str = serde_yaml::to_string(&tasks)?;

        print!("{}", yaml_str);

        Ok(())
    }

    pub fn convert_yaml_to_json<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let tasks : Tasks = serde_yaml::from_reader(reader)?;

        let json_str = serde_json::to_string_pretty(&tasks)?;

        println!("{}", json_str);

        Ok(())
    }
}
