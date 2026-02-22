use ratatui::prelude::*;
use anyhow::Result;

pub fn fuzzy_select(options: &[String], prompt: &str) -> Result<String> {
    // Placeholder: implement ratatui fuzzy finder here
    // For now, just return the first option
    options.get(0).cloned().ok_or_else(|| anyhow::anyhow!("No options provided"))
}
