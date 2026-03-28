use globset::{Glob, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct TaskCondition {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

impl TaskCondition {
    pub fn is_valid(&self) -> bool {
        let cwd = match env::current_dir() {
            Ok(p) => p,
            Err(_) => return false, // default to false if we cannot determine the current directory
        };

        // Attempt to read entries from the real filesystem; pass them to is_valid_in.
        let entries = fs::read_dir(&cwd).ok().map(|iter| {
            iter.filter_map(|e| e.ok().and_then(|en| en.file_name().into_string().ok()))
                .collect::<Vec<String>>()
        });

        self.is_valid_in(&cwd, entries.as_deref())
    }

    // This helper allows for testing the globs without the need for FS access
    fn is_valid_in<P: AsRef<std::path::Path>>(&self, cwd: P, entries: Option<&[String]>) -> bool {
        let cwd = cwd.as_ref();

        if !self.paths.is_empty() {
            let mut builder = GlobSetBuilder::new();

            for pattern in &self.paths {
                if let Ok(glob) = Glob::new(pattern) {
                    builder.add(glob);
                }
            }

            if let Ok(set) = builder.build() {
                if set.is_match(cwd) {
                    return true;
                }
            }
        }

        if !self.files.is_empty() {
            let mut builder = GlobSetBuilder::new();

            for pattern in &self.files {
                if let Ok(glob) = Glob::new(pattern) {
                    builder.add(glob);
                }
            }

            if let Ok(set) = builder.build() {
                if let Some(list) = entries {
                    for name in list {
                        if set.is_match(std::path::Path::new(name)) {
                            return true;
                        }
                    }
                } else if let Ok(dir_entries) = fs::read_dir(cwd) {
                    for entry in dir_entries.flatten() {
                        if set.is_match(entry.file_name()) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::TaskCondition;
    use std::path::PathBuf;

    #[test]
    fn empty_conditions_are_invalid() {
        let c = TaskCondition {
            paths: vec![],
            files: vec![],
        };
        // Provide a fake cwd and an empty entry list; no fs used.
        assert!(!c.is_valid_in(PathBuf::from("/some/cwd"), Some(&[])));
    }

    #[test]
    fn matching_path_is_valid() {
        let cwd = PathBuf::from("/home/user/project");
        let pattern = cwd.to_str().unwrap().to_string();
        let c = TaskCondition {
            paths: vec![pattern],
            files: vec![],
        };
        assert!(c.is_valid_in(&cwd, Some(&[])));
    }

    #[test]
    fn matching_file_is_valid() {
        let cwd = PathBuf::from("/home/user/project");
        let fname = "marker.file".to_string();
        let c = TaskCondition {
            paths: vec![],
            files: vec![fname.clone()],
        };
        // Provide entries containing the marker file; no disk IO.
        assert!(c.is_valid_in(&cwd, Some(&[fname])));
    }

    #[test]
    fn matching_dir_is_valid() {
        let cwd = PathBuf::from("/home/user/project");
        // Simulate a direct directory entry as returned by read_dir(cwd).
        let fname = "some_dir".to_string();
        let c = TaskCondition {
            paths: vec![],
            files: vec![fname.clone()],
        };
        // Provide entries containing the marker directory; no disk IO.
        assert!(c.is_valid_in(&cwd, Some(&[fname])));
    }

    #[test]
    fn mix_matches_by_path() {
        let cwd = PathBuf::from("/home/user/project/foo");
        let c = TaskCondition {
            paths: vec!["/home/user/project/*".to_string()],
            files: vec!["no-such-file".to_string()],
        };
        assert!(c.is_valid_in(&cwd, Some(&[])));
    }

    #[test]
    fn mix_matches_by_file_glob() {
        let cwd = PathBuf::from("/home/user/project");
        let entries = vec!["app.log".to_string(), "README.md".to_string()];
        let c = TaskCondition {
            paths: vec!["/no/such/path".to_string()],
            files: vec!["*.log".to_string()],
        };
        assert!(c.is_valid_in(&cwd, Some(&entries)));
    }

    #[test]
    fn recursive_path_glob_matches() {
        let cwd = PathBuf::from("/home/user/a/b/foo");
        let c = TaskCondition {
            paths: vec!["/home/user/**/foo".to_string()],
            files: vec![],
        };
        assert!(c.is_valid_in(&cwd, Some(&[])));
    }

    #[test]
    fn file_glob_question_mark_matches() {
        let cwd = PathBuf::from("/project");
        let entries = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        let c = TaskCondition {
            paths: vec![],
            files: vec!["file?.txt".to_string()],
        };
        assert!(c.is_valid_in(&cwd, Some(&entries)));
    }

    #[test]
    fn negative_with_globs_is_invalid() {
        let cwd = PathBuf::from("/no/match/here");
        let entries = vec!["other.txt".to_string()];
        let c = TaskCondition {
            paths: vec!["/not/**/matching".to_string()],
            files: vec!["*.log".to_string()],
        };
        assert!(!c.is_valid_in(&cwd, Some(&entries)));
    }
}
