use crate::{
    input::Input,
    tasks::{TaskEntry, Tasks},
};

pub struct ValidationReport {
    pub validation_id: String,
    pub statuses: Vec<String>,
}

pub struct Validator {
    pub verbose: bool,
}

impl Validator {
    pub fn validate_task_command(&self, entry: &TaskEntry, source: &Tasks) -> ValidationReport {
        let mut report = ValidationReport {
            validation_id: entry.label.clone(),
            statuses: Vec::new(),
        };

        Input::extract_variables(&entry.command)
            .iter()
            .for_each(|input_id| {
                match source.get_input(input_id) {
                    Ok(_) => {
                        if self.verbose {
                            report
                                .statuses
                                .push(format!("✅ Input '{}' defined", input_id));
                        }
                    }
                    Err(_) => {
                            report
                                .statuses
                                .push(format!("❌ Input '{}' not defined", input_id));
                    }
                }
            });

        report
    }
}
