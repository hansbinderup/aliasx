use crate::{
    input::Input,
    input_mapping::InputMapping,
    tasks::{TaskEntry, Tasks},
};
use owo_colors::OwoColorize;

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Pass { message: String },
    Fail { message: String },
}

impl ValidationStatus {
    fn pass(message: impl Into<String>) -> Self {
        Self::Pass {
            message: message.into(),
        }
    }

    fn fail(message: impl Into<String>) -> Self {
        Self::Fail {
            message: message.into(),
        }
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }

    pub fn format(&self) -> String {
        match self {
            Self::Pass { message } => format!("{} {}", "✓".green().bold(), message.dimmed()),
            Self::Fail { message } => format!("{} {}", "✗".red().bold(), message),
        }
    }
}

pub struct ValidationReport {
    pub validation_id: String,
    pub statuses: Vec<ValidationStatus>,
}

impl ValidationReport {
    fn new(validation_id: impl Into<String>) -> Self {
        Self {
            validation_id: validation_id.into(),
            statuses: Vec::new(),
        }
    }

    fn add_statuses(&mut self, statuses: impl IntoIterator<Item = ValidationStatus>) {
        self.statuses.extend(statuses);
    }

    pub fn has_failures(&self) -> bool {
        self.statuses.iter().any(ValidationStatus::is_fail)
    }

    pub fn failures(&self) -> impl Iterator<Item = &ValidationStatus> {
        self.statuses.iter().filter(|s| s.is_fail())
    }

    pub fn passes(&self) -> impl Iterator<Item = &ValidationStatus> {
        self.statuses.iter().filter(|s| s.is_pass())
    }

    pub fn failure_count(&self) -> usize {
        self.failures().count()
    }

    pub fn pass_count(&self) -> usize {
        self.passes().count()
    }

    pub fn print(&self, verbose: bool) {
        let fail_count = self.failure_count();
        let total = self.statuses.len();

        // Print header with task name and status badge
        if verbose || fail_count > 0 {
            let badge = if fail_count == 0 {
                format!(" {}", "PASS".green().bold())
            } else {
                format!(
                    " {} {}/{}",
                    "FAIL".red().bold(),
                    fail_count.red().bold(),
                    total
                )
            };

            println!("{}{}", self.validation_id.bold(), badge);

            // Print details with indentation
            if verbose || fail_count > 0 {
                for status in &self.statuses {
                    if verbose || status.is_fail() {
                        println!("  {}", status.format());
                    }
                }
            }
        }
    }

    pub fn print_compact(&self) {
        let fail_count = self.failure_count();

        if fail_count == 0 {
            println!("{} {}", "✓".green().bold(), self.validation_id.dimmed());
        } else {
            println!(
                "{} {} {}",
                "✗".red().bold(),
                self.validation_id,
                format!("({} issues)", fail_count).red()
            );
            for status in self.failures() {
                println!("    {}", status.format());
            }
        }
    }
}

pub struct Validator {
    pub verbose: bool,
}

impl Validator {
    pub fn validate_task_command(&self, entry: &TaskEntry, source: &Tasks) -> ValidationReport {
        let mut report = ValidationReport::new(&entry.label);

        report.add_statuses(self.check_inputs(entry, source));
        report.add_statuses(self.check_mappings(entry, source));

        report
    }

    fn check_inputs(&self, entry: &TaskEntry, source: &Tasks) -> Vec<ValidationStatus> {
        Input::extract_variables(&entry.command)
            .into_iter()
            .filter_map(|input_id| self.check_input_defined(&input_id, source))
            .collect()
    }

    fn check_input_defined(&self, input_id: &str, source: &Tasks) -> Option<ValidationStatus> {
        match source.get_input(input_id) {
            Ok(_) if self.verbose => Some(ValidationStatus::pass(format!(
                "Input '{}' defined",
                input_id
            ))),
            Ok(_) => None,
            Err(_) => Some(ValidationStatus::fail(format!(
                "Input '{}' not defined",
                input_id
            ))),
        }
    }

    fn check_mappings(&self, entry: &TaskEntry, source: &Tasks) -> Vec<ValidationStatus> {
        InputMapping::extract_from_str(&entry.command)
            .into_iter()
            .flat_map(|mapping_id| self.check_mapping(&mapping_id, source))
            .collect()
    }

    fn check_mapping(&self, mapping_id: &str, source: &Tasks) -> Vec<ValidationStatus> {
        let mut statuses = Vec::new();

        match source.get_mapping(mapping_id) {
            Ok(mapping) => {
                if self.verbose {
                    statuses.push(ValidationStatus::pass(format!(
                        "Mapping '{}' defined",
                        mapping_id
                    )));
                }
                statuses.extend(self.check_mapping_inputs(mapping, source));
            }
            Err(_) => {
                statuses.push(ValidationStatus::fail(format!(
                    "Mapping '{}' not defined",
                    mapping_id
                )));
            }
        }

        statuses
    }

    fn check_mapping_inputs(
        &self,
        mapping: &InputMapping,
        source: &Tasks,
    ) -> Vec<ValidationStatus> {
        match source.get_input(&mapping.input) {
            Ok(input) => input
                .options
                .iter()
                .filter(|option| !mapping.options.contains_key(*option))
                .map(|option| {
                    ValidationStatus::fail(format!(
                        "Mapping '{}' doesn't define option for input '{}'",
                        mapping.id, option
                    ))
                })
                .collect(),
            Err(_) => {
                vec![ValidationStatus::fail(format!(
                    "Mapping '{}' references undefined input '{}'",
                    mapping.id, mapping.input
                ))]
            }
        }
    }
}
