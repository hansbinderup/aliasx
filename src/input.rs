use std::sync::LazyLock;

use fuzzy_select::FuzzySelect;
use regex::Regex;
use serde::{Deserialize, Serialize};

static VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{([^:}]+):([^}]+)\}").expect("invalid regex"));

static REPLACE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{.*?\}").expect("invalid regex"));

// TODO: add default option
// TODO: reuse inputs if same type is specified twice?
#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Input {
    pub id: String,
    pub options: Vec<String>,
    pub description: Option<String>,
}

// Input is defined as ${<type>:<id>}
// FIXME: add type as enum when adding support for runtime arguments
impl Input {
    pub fn extract_variables(s: &str) -> Vec<(String, String)> {
        VARIABLE_REGEX
            .captures_iter(s)
            .map(|cap| (cap[1].to_string(), cap[2].to_string()))
            .collect()
    }

    pub fn replace_next_variable(s: &str, replacement: &str) -> String {
        REPLACE_REGEX.replace(s, replacement).to_string()
    }

    pub fn fzf(&self) -> anyhow::Result<String> {
        let prompt = format!(
            "input | {}:",
            self.description.as_deref().unwrap_or(&self.id)
        );

        // Show fuzzy picker and get selection index
        let selection = FuzzySelect::new()
            .with_prompt(prompt)
            .with_options(self.options.clone())
            .select()?;

        Ok(selection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_parsing_single() {
        let yaml = r#"
        - id: some-type
          description: 'some description'
          options:
              - '1'
              - '2'
        "#;

        let values: Vec<Input> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].id, "some-type");
        assert_eq!(values[0].description, Some("some description".to_string()));
        assert_eq!(values[0].options, vec!["1", "2"]);
    }

    #[test]
    fn test_input_parsing_multiple() {
        let yaml = r#"
        - id: environment
          description: 'Select environment'
          options:
              - dev
              - staging
              - prod
        - id: input
          options:
              - us-east-1
              - eu-west-1
        "#;

        let values: Vec<Input> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].id, "environment");
        assert_eq!(
            values[0].description,
            Some("Select environment".to_string())
        );
        assert_eq!(values[0].options.len(), 3);
        assert_eq!(values[1].id, "input");
        assert_eq!(values[1].description, None);
        assert_eq!(values[1].options.len(), 2);
    }

    #[test]
    fn test_input_parsing_no_description() {
        let yaml = r#"
        - id: choice
          options:
              - option1
              - option2
        "#;

        let values: Vec<Input> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].id, "choice");
        assert_eq!(values[0].description, None);
        assert_eq!(values[0].options, vec!["option1", "option2"]);
    }

    #[test]
    fn test_extract_variables() {
        let input = "deploy ${input:environment} to ${input:region}";
        let vars = Input::extract_variables(input);

        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0], ("input".to_string(), "environment".to_string()));
        assert_eq!(vars[1], ("input".to_string(), "region".to_string()));
    }

    #[test]
    fn test_extract_variables_none() {
        let input = "no variables here";
        let vars = Input::extract_variables(input);
        assert_eq!(vars.len(), 0);
    }

    #[test]
    fn test_extract_variables_complex() {
        let input = "cmd ${type1:id-1} ${type2:id_2} ${type3:id.3}";
        let vars = Input::extract_variables(input);

        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0], ("type1".to_string(), "id-1".to_string()));
        assert_eq!(vars[1], ("type2".to_string(), "id_2".to_string()));
        assert_eq!(vars[2], ("type3".to_string(), "id.3".to_string()));
    }

    #[test]
    fn test_replace_next_variable() {
        let input = "hello ${input:x} and ${input:y}";

        let mut replaced = Input::replace_next_variable(input, "world");
        assert_eq!(replaced, "hello world and ${input:y}");

        replaced = Input::replace_next_variable(&replaced, "goodbye");
        assert_eq!(replaced, "hello world and goodbye");
    }

    #[test]
    fn test_replace_next_variable_single() {
        let input = "only ${input:value} here";
        let replaced = Input::replace_next_variable(input, "replaced");
        assert_eq!(replaced, "only replaced here");
    }

    #[test]
    fn test_replace_next_variable_none() {
        let input = "no variables";
        let replaced = Input::replace_next_variable(input, "replacement");
        assert_eq!(replaced, "no variables");
    }
}
