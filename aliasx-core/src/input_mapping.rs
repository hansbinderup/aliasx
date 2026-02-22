use std::sync::LazyLock;

use anyhow::anyhow;
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use serde::{Deserialize, Serialize};

static FIND_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{mapping:([^}]+)\}").expect("invalid regex"));

// InputMapping is defined as ${mapping:<id>}
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct InputMapping {
    pub id: String,
    pub input: String,
    pub options: IndexMap<String, String>,
}

/*
* InputMapping implementation
* Replaces mapped values in a string based on the input selections provided.
*
* Example:
* "some command: ${input:type1} some args: ${mapping:type1}"
* will replace ${mapping:type1} with the mapped value based on the selected input for 'type1'
*
* eg: if input_selections has "type1" -> "optionA"
* and options has "optionA" -> "mappedValue"
* then the result will be:
* "some command: optionA some args: mappedValue"
*
* The mapping will be replaced for each element of the option selected meaning that
* you can build dynamic commands based on user input selections.
 */
impl InputMapping {
    pub fn extract_from_str(s: &str) -> IndexSet<String> {
        FIND_REGEX
            .captures_iter(s)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    pub fn replace_all(s: &str, id: &str, replacement: &str) -> anyhow::Result<String> {
        let pattern = format!("${{mapping:{}}}", id);
        if !s.contains(&pattern) {
            return Err(anyhow!(
                "mapping pattern '{}' not found in command",
                pattern
            ));
        }

        Ok(s.replace(&pattern, replacement).to_string())
    }
}
