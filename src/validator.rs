use crate::{
    input::Input,
    input_mapping::InputMapping,
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

        self.validate_inputs(&mut report, entry, source);
        self.validate_mappings(&mut report, entry, source);

        report
    }

    fn validate_inputs(&self, report: &mut ValidationReport, entry: &TaskEntry, source: &Tasks) {
        Input::extract_variables(&entry.command)
            .iter()
            .for_each(|input_id| match source.get_input(input_id) {
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
            });
    }

    fn validate_mappings(&self, report: &mut ValidationReport, entry: &TaskEntry, source: &Tasks) {
        InputMapping::extract_from_str(&entry.command)
            .iter()
            .for_each(|mapping_id| match source.get_mapping(mapping_id) {
                Ok(mapping) => {
                    if self.verbose {
                        report
                            .statuses
                            .push(format!("✅ Mapping '{}' defined", mapping_id));
                    }

                    self.validate_mapping_inputs(report, mapping, source);
                }
                Err(_) => {
                    report
                        .statuses
                        .push(format!("❌ Mapping '{}' not defined", mapping_id));
                }
            });
    }

    // cross check mapping options against input options
    fn validate_mapping_inputs(
        &self,
        report: &mut ValidationReport,
        mapping: &InputMapping,
        source: &Tasks,
    ) {
        match source.get_input(&mapping.input) {
            Ok(input) => {
                for option in input.options.iter() {
                    if !mapping.options.contains_key(option) {
                        report.statuses.push(format!(
                            "❌ Mapping '{}' doesn't define option for input '{}'",
                            mapping.id, option
                        ));
                    }
                }
            }
            Err(_) => {
                report.statuses.push(format!(
                    "❌ Mapping '{}' references undefined input '{}'",
                    mapping.id, mapping.input
                ));
            }
        }
    }
}
