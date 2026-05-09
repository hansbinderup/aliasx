use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, path::PathBuf};

// TODO: add to docs
// TODO: consider adding example to config generator
// TODO: add validation?

// Follows the structure of "vs-code tasks options"

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct TaskOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<String, String>,
}

impl Hash for TaskOption {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cwd.hash(state);

        // IndexMap is deterministic hence why this is okay
        for (k, v) in &self.env {
            k.hash(state);
            v.hash(state);
        }
    }
}
