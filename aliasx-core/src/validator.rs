use crate::{
    history::History,
    input::Input,
    input_mapping::InputMapping,
    tasks::{TaskEntry, Tasks},
};
use owo_colors::OwoColorize;

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Pass { message: String },
    Fail { message: String },
    Skip { message: String },
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

    fn skip(message: impl Into<String>) -> Self {
        Self::Skip {
            message: message.into(),
        }
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }

    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip { .. })
    }

    pub fn format(&self) -> String {
        match self {
            Self::Pass { message } => format!("{} {}", "✓".green().bold(), message.dimmed()),
            Self::Fail { message } => format!("{} {}", "✗".red().bold(), message),
            Self::Skip { message } => {
                format!("{} {} (skipped)", "⏭".yellow().bold(), message.dimmed())
            }
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

    fn add_status(&mut self, status: ValidationStatus) {
        self.statuses.push(status);
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

    pub fn skips(&self) -> impl Iterator<Item = &ValidationStatus> {
        self.statuses.iter().filter(|s| s.is_skip())
    }

    pub fn failure_count(&self) -> usize {
        self.failures().count()
    }

    pub fn pass_count(&self) -> usize {
        self.passes().count()
    }

    pub fn skip_count(&self) -> usize {
        self.skips().count()
    }

    pub fn print(&self, verbose: bool) {
        if !verbose {
            self.print_compact();
            return;
        }

        let fail_count = self.failure_count();
        let total = self.statuses.len();

        // Print header with task name and status badge
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
        for status in &self.statuses {
            println!("  {}", status.format());
        }
    }

    fn print_compact(&self) {
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
    pub fn validate_history(&self) -> ValidationReport {
        let mut report = ValidationReport::new("History");

        match History::load() {
            Ok(data) => {
                report.add_status(ValidationStatus::pass(format!(
                    "History was loaded ({} entries)",
                    data.len()
                )));
            }
            Err(err) => {
                report.add_status(ValidationStatus::fail(err.to_string()));
            }
        }

        report
    }

    pub fn validate_task_command(&self, entry: &TaskEntry, source: &Tasks) -> ValidationReport {
        let mut report = ValidationReport::new(&entry.label);

        report.add_statuses(self.check_inputs(entry, source));
        report.add_statuses(self.check_mappings(entry, source));
        report.add_statuses(self.check_conditions(entry));

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

    // Ensure every option defined on the input has a corresponding entry in the mapping's options map.
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

    fn check_conditions(&self, entry: &TaskEntry) -> Option<ValidationStatus> {
        if let Some(condition) = &entry.conditions {
            if let Some(err) = condition.validate() {
                return Some(ValidationStatus::fail(format!(
                    "Condition: '{}' is not valid",
                    err
                )));
            }

            if condition.is_valid() {
                Some(ValidationStatus::pass("Conditions are met"))
            } else {
                Some(ValidationStatus::skip("Conditions are not met"))
            }
        } else {
            Option::None
        }
    }

    pub fn print_single_report(&self, report: &ValidationReport) {
        report.print(self.verbose);
        println!("");
    }

    pub fn print_report(&self, reports: &[ValidationReport]) {
        for report in reports.iter() {
            report.print(self.verbose);
        }
        println!("");
    }

    pub fn print_header() {
        println!("{}", "═".repeat(60).dimmed());
        println!("{}", "  VALIDATION REPORT".bold().cyan());
        println!("{}\n", "═".repeat(60).dimmed());
    }

    // not pretty.. fix later
    pub fn print_summary(reports: impl IntoIterator<Item = ValidationReport>) {
        let reports: Vec<_> = reports.into_iter().collect();

        let total_tasks = reports.len();
        let failed_tasks = reports.iter().filter(|r| r.has_failures()).count();
        let passed_tasks = total_tasks - failed_tasks;
        let total_failures = reports.iter().map(|r| r.failure_count()).sum::<usize>();

        println!("{}", "═".repeat(60).dimmed());
        println!("{}", "  SUMMARY".bold().cyan());
        println!("{}", "═".repeat(60).dimmed());

        println!(
            "  {} {}  {} {}",
            "✓".green().bold(),
            format!("{} passed", passed_tasks).green(),
            if failed_tasks > 0 {
                "✗".red().bold().to_string()
            } else {
                "".to_string()
            },
            if failed_tasks > 0 {
                format!("{} failed", failed_tasks).red().to_string()
            } else {
                "".to_string()
            }
        );

        if total_failures > 0 {
            println!(
                "  {} {} total",
                "⚠".yellow().bold(),
                format!("{} issues", total_failures).yellow()
            );
        }

        println!("{}", "═".repeat(60).dimmed());

        if failed_tasks == 0 {
            println!(
                "\n{} {}\n",
                "✓".green().bold(),
                "All validations passed!".green().bold()
            );
        } else {
            println!(
                "\n{} {}\n",
                "⚠".yellow().bold(),
                "Some validations failed.".yellow().bold()
            );
        }
    }
}
