use strum::{Display, EnumString};
use serde::{Deserialize, Serialize};

#[derive(EnumString, Display, Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum TaskFilter {
    All,
    Local,
    Global,
}

impl Default for TaskFilter {
    fn default() -> Self {
        TaskFilter::Local
    }
}

impl TaskFilter {
    pub fn include_local(self) -> bool {
        matches!(self, TaskFilter::All | TaskFilter::Local)
    }

    pub fn include_global(self) -> bool {
        matches!(self, TaskFilter::All | TaskFilter::Global)
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
