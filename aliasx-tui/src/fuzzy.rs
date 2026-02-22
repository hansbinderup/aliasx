use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{
    widgets::{footer, highlight_match},
    TuiSession,
};

fn run(
    options: &[String],
    prompt: &str,
    initial_query: &str,
    initial_position: usize,
    session: &mut TuiSession,
) -> Result<(usize, String)> {
    let mut selected = initial_position;
    let mut query = initial_query.to_string();
    let mut list_state = ListState::default();

    loop {
        let query_lower = query.to_lowercase();
        let filtered: Vec<(usize, &String)> = options
            .iter()
            .enumerate()
            .filter(|(_, s)| s.to_lowercase().contains(&query_lower))
            .collect();

        selected = selected.min(filtered.len().saturating_sub(1));
        list_state.select(if filtered.is_empty() {
            None
        } else {
            Some(selected)
        });

        session.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(f.area());

            let display_query = if query.is_empty() {
                Span::styled(
                    "type to search...",
                    Style::default().fg(Color::DarkGray).italic(),
                )
            } else {
                Span::raw(&query)
            };

            let search = Paragraph::new(Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Yellow)),
                display_query,
            ]))
            .block(Block::default().title(prompt).borders(Borders::ALL));
            f.render_widget(search, layout[0]);

            let items: Vec<ListItem> = filtered
                .iter()
                .map(|(_, s)| ListItem::new(highlight_match(s.as_str(), &query)))
                .collect();
            let list = List::new(items)
                .block(Block::default().title("Selections").borders(Borders::ALL))
                .highlight_symbol("❯ ")
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_stateful_widget(list, layout[1], &mut list_state);

            f.render_widget(footer(&[]), layout[2]);
        })?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char(c) => {
                    selected = 0;
                    query.push(c);
                }
                KeyCode::Backspace => {
                    query.pop();
                    selected = 0;
                }
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected + 1 < filtered.len() {
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some((orig_idx, s)) = filtered.get(selected) {
                        return Ok((*orig_idx, (*s).clone()));
                    }
                }
                KeyCode::Esc => return Err(anyhow::anyhow!("No selection made")),
                _ => {}
            }
        }
    }
}

/// Run a string fuzzy finder on an existing [`TuiSession`].
pub fn string_fuzzy_finder_with(
    options: &[String],
    prompt: &str,
    initial_query: &str,
    initial_position: usize,
    session: &mut TuiSession,
) -> Result<(usize, String)> {
    run(options, prompt, initial_query, initial_position, session)
}

/// Open a standalone string fuzzy finder (creates its own terminal session).
pub fn string_fuzzy_finder(
    options: &[String],
    prompt: &str,
    initial_query: &str,
    initial_position: usize,
) -> Result<(usize, String)> {
    run(
        options,
        prompt,
        initial_query,
        initial_position,
        &mut TuiSession::new()?,
    )
}
