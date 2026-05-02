use aliasx_core::history::{History, HistoryEntry};
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

pub struct FuzzyConfig {
    pub has_details: bool,
    pub show_details: bool,

    /// Filter tab labels. When non-empty, Tab/Shift+Tab cycle through the tabs.
    /// Items are matched by `FuzzyList::match_filter`.
    pub filters: Vec<String>,

    pub initial_query: String,
    pub initial_position: usize,
}

impl Default for FuzzyConfig {
    fn default() -> Self {
        Self {
            has_details: true,
            show_details: false,
            filters: vec![],
            initial_query: String::new(),
            initial_position: 0,
        }
    }
}

pub trait FuzzyList {
    fn label(&self) -> &str;

    fn label_prefix(&self) -> Option<&str> {
        None
    }

    fn label_suffix(&self) -> Option<&str> {
        None
    }

    fn detail(&self) -> Option<String> {
        None
    }

    fn searchable(&self) -> String {
        self.label().to_lowercase()
    }

    fn match_filter(&self, _filter: &str) -> bool {
        true
    }
}

impl FuzzyList for String {
    fn label(&self) -> &str {
        self
    }
}

impl FuzzyList for HistoryEntry {
    fn label(&self) -> &str {
        &self.task_name
    }

    fn label_prefix(&self) -> Option<&str> {
        None
    }

    fn detail(&self) -> Option<String> {
        Some(format!(
            "[{}] {} | {}",
            History::format_timestamp(&self.started_at),
            self.task_command,
            self.scope
        ))
    }
}

fn filter_tabs(filters: &[String], active: usize) -> Line<'_> {
    let active_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::DarkGray);

    let mut spans = Vec::new();
    for (i, name) in filters.iter().enumerate() {
        spans.push(Span::styled(
            format!(" {} ", name),
            if i == active {
                active_style
            } else {
                inactive_style
            },
        ));
    }
    Line::from(spans)
}

fn build_list_item<'a, T: FuzzyList>(item: &'a T, query: &str) -> ListItem<'a> {
     let highlighted = highlight_match(item.label(), query);
     let prefix = item.label_prefix();
     let suffix = item.label_suffix();

     if prefix.is_none() && suffix.is_none() {
         return ListItem::new(highlighted);
     }

     let mut spans = vec![];
     if let Some(prefix) = prefix {
         spans.push(Span::styled(prefix, Style::default().fg(Color::DarkGray)));
     }
     spans.extend(highlighted.spans);
     if let Some(suffix) = suffix {
         spans.push(Span::styled(suffix, Style::default().fg(Color::DarkGray)));
     }
     ListItem::new(Line::from(spans))
}

pub fn fuzzy_finder<T: FuzzyList>(
    items: &[T],
    prompt: &str,
    config: FuzzyConfig,
    session: &mut TuiSession,
) -> anyhow::Result<usize> {
    let mut selected = config.initial_position;
    let mut query = config.initial_query;
    let mut list_state = ListState::default();
    let mut show_details = config.show_details;
    let mut filter_idx = 0usize;
    let has_filters = !config.filters.is_empty();
    let num_filters = config.filters.len();

    loop {
        let q = query.to_lowercase();
        let active_filter = (has_filters).then(|| config.filters[filter_idx].as_str());

        let filtered: Vec<(usize, &T)> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                let matches_filter = active_filter.map_or(true, |f| item.match_filter(f));
                let matches_query = item.searchable().contains(&q);
                matches_filter && matches_query
            })
            .collect();

        selected = selected.min(filtered.len().saturating_sub(1));
        list_state.select((!filtered.is_empty()).then_some(selected));

        session.draw(|f| {
            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(f.area());

            // search bar
            let display_query = if query.is_empty() {
                Span::styled(
                    "type to search...",
                    Style::default().fg(Color::DarkGray).italic(),
                )
            } else {
                Span::raw(&query)
            };
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("> ", Style::default().fg(Color::Yellow)),
                    display_query,
                ]))
                .block(Block::default().title(prompt).borders(Borders::ALL)),
                v[0],
            );

            // list / detail pane
            let (list_area, detail_area) = if show_details {
                let h = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(40), Constraint::Percentage(60)])
                    .split(v[1]);
                (h[0], Some(h[1]))
            } else {
                (v[1], None)
            };

            let list_title = if has_filters {
                filter_tabs(&config.filters, filter_idx)
            } else {
                Line::raw("")
            };
            let list_items: Vec<ListItem> = filtered
                .iter()
                .map(|(_, item)| build_list_item(*item, &query))
                .collect();
            f.render_stateful_widget(
                List::new(list_items)
                    .block(Block::default().title(list_title).borders(Borders::ALL))
                    .highlight_symbol("❯ ")
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                list_area,
                &mut list_state,
            );

            // detail pane
            if let Some(area) = detail_area {
                let selected_idx = filtered.get(selected).map(|(i, _)| *i);
                let lines: Vec<Line> = filtered
                    .iter()
                    .map(|(i, item)| {
                        let style = if Some(*i) == selected_idx {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        Line::from(Span::styled(item.detail().unwrap_or_default(), style))
                    })
                    .collect();
                f.render_stateful_widget(
                    List::new(lines).block(Block::default().title("Details").borders(Borders::ALL)),
                    area,
                    &mut list_state,
                );
            }

            let mut hints: Vec<(&str, &str)> = vec![];
            if has_filters {
                hints.push(("tab/⇧tab", "filter"));
            }

            if config.has_details {
                hints.push(if show_details {
                    ("?", "hide details")
                } else {
                    ("?", "show details")
                });
                hints.push(("esc", "cancel"));
            }

            f.render_widget(footer(&hints), v[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Tab if has_filters => {
                    filter_idx = (filter_idx + 1) % (num_filters);
                    selected = 0;
                }
                KeyCode::BackTab if has_filters => {
                    filter_idx = filter_idx.checked_sub(1).unwrap_or(num_filters - 1);
                    selected = 0;
                }
                KeyCode::Char('?') => show_details = config.has_details && !show_details,
                KeyCode::Char(c) => {
                    query.push(c);
                    selected = 0;
                }
                KeyCode::Backspace => {
                    query.pop();
                    selected = 0;
                }
                KeyCode::Up => {
                    selected = if selected > 0 {
                        selected - 1
                    } else {
                        filtered.len().saturating_sub(1)
                    }
                }
                KeyCode::Down => {
                    selected = if selected + 1 < filtered.len() {
                        selected + 1
                    } else {
                        0
                    }
                }
                KeyCode::Enter => {
                    if let Some((orig_idx, _)) = filtered.get(selected) {
                        return Ok(*orig_idx);
                    }
                }
                KeyCode::Esc => return Err(anyhow::anyhow!("No selection made")),
                _ => {}
            }
        }
    }
}
