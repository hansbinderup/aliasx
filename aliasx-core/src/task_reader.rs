use anyhow::Context;
use std::path::Path;

use crate::{task_filter::TaskFilter, tasks::Tasks};

/// strict parsing - will fail if not exists or if malformed
pub fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks> {
    let path = path.as_ref();

    let format = TaskFormat::from_path(path)
        .with_context(|| format!("unsupported file format: {:?}", path))?;

    format.parse(path)
}

pub fn push_if_exists<P>(sources: &mut Vec<Tasks>, path: P, filter: TaskFilter) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.is_file() {
        return Ok(());
    }

    let mut tasks = parse_file(path)?;
    tasks.scope = filter;

    sources.push(tasks);

    Ok(())
}

enum TaskFormat {
    Yaml,
    Json,
}

impl TaskFormat {
    fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("yaml") | Some("yml") => Some(Self::Yaml),
            Some("json") | Some("json5") => Some(Self::Json),
            _ => None,
        }
    }

    fn parse(self, path: &Path) -> anyhow::Result<Tasks> {
        match self {
            Self::Yaml => YamlTaskReader::parse_file(path),
            Self::Json => JsonTaskReader::parse_file(path),
        }
    }
}

struct YamlTaskReader;
struct JsonTaskReader;

trait TaskReader {
    fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks>;
}

impl TaskReader for YamlTaskReader {
    fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks> {
        let file = std::fs::File::open(&path)
            .with_context(|| format!("failed to open YAML file: {:?}", path.as_ref()))?;

        let reader = std::io::BufReader::new(file);

        Ok(serde_yaml::from_reader(reader)
            .with_context(|| format!("failed to parse YAML: {:?}", path.as_ref()))?)
    }
}

impl TaskReader for JsonTaskReader {
    fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Tasks> {
        let file = std::fs::File::open(&path)
            .with_context(|| format!("failed to open JSON file: {:?}", path.as_ref()))?;

        let reader = std::io::BufReader::new(file);

        Ok(serde_json5::from_reader(reader)
            .with_context(|| format!("failed to parse JSON: {:?}", path.as_ref()))?)
    }
}
