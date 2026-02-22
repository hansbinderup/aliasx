use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFilter {
    All,
    Local,
    Global,
}

impl TaskFilter {
    pub fn include_local(self) -> bool {
        matches!(self, TaskFilter::All | TaskFilter::Local)
    }

    pub fn include_global(self) -> bool {
        matches!(self, TaskFilter::All | TaskFilter::Global)
    }
}

impl FromStr for TaskFilter {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "all" => Ok(TaskFilter::All),
            "local" => Ok(TaskFilter::Local),
            "global" => Ok(TaskFilter::Global),
            _ => Err(format!("Invalid value for TaskFilter: {}", s)),
        }
    }
}

impl fmt::Display for TaskFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskFilter::All => "all",
            TaskFilter::Local => "local",
            TaskFilter::Global => "global",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_filter_include_local() {
        assert!(TaskFilter::All.include_local());
        assert!(TaskFilter::Local.include_local());
        assert!(!TaskFilter::Global.include_local());
    }

    #[test]
    fn test_task_filter_include_global() {
        assert!(TaskFilter::All.include_global());
        assert!(!TaskFilter::Local.include_global());
        assert!(TaskFilter::Global.include_global());
    }
}
