use aliasx_core::task_collection::TaskCollection;
use aliasx_core::task_filter::TaskFilter;
use aliasx_core::tasks::TaskEntry;
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

fn filter_title(f: TaskFilter) -> Line<'static> {
    let tab = |t: TaskFilter| {
        let label = match t {
            TaskFilter::All => "all",
            TaskFilter::Local => "local",
            TaskFilter::Global => "global",
        };
        if t == f {
            Span::styled(
                label,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(label, Style::default().fg(Color::DarkGray))
        }
    };
    Line::from(vec![
        Span::raw(" "),
        tab(TaskFilter::All),
        Span::raw("  "),
        tab(TaskFilter::Local),
        Span::raw("  "),
        tab(TaskFilter::Global),
        Span::raw(" "),
    ])
}

fn next_filter(f: TaskFilter) -> TaskFilter {
    match f {
        TaskFilter::All => TaskFilter::Local,
        TaskFilter::Local => TaskFilter::Global,
        TaskFilter::Global => TaskFilter::All,
    }
}

fn prev_filter(f: TaskFilter) -> TaskFilter {
    match f {
        TaskFilter::All => TaskFilter::Global,
        TaskFilter::Global => TaskFilter::Local,
        TaskFilter::Local => TaskFilter::All,
    }
}

pub fn task_fuzzy_finder(
    tasks: &[(usize, TaskFilter, &TaskEntry)],
    collection: &TaskCollection,
    session: &mut TuiSession,
    query: &str,
    verbose: bool,
) -> Result<usize> {
    let mut query = query.to_string();
    let mut selected = 0usize;
    let mut list_state = ListState::default();
    let mut scope = TaskFilter::All;
    let mut show_details = verbose;

    loop {
        let q = query.to_lowercase();
        let filtered: Vec<(usize, &TaskEntry)> = tasks
            .iter()
            .filter(|(_, s, t)| {
                let in_scope = match scope {
                    TaskFilter::All => true,
                    TaskFilter::Local => s.include_local() && !s.include_global(),
                    TaskFilter::Global => s.include_global() && !s.include_local(),
                };
                in_scope && t.label.to_lowercase().contains(&q)
            })
            .map(|(id, _, t)| (*id, *t))
            .collect();

        selected = selected.min(filtered.len().saturating_sub(1));
        list_state.select(if filtered.is_empty() {
            None
        } else {
            Some(selected)
        });

        let width = collection.width_idx();
        session.draw(|f| {
            let v_layout = Layout::default()
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
            .block(Block::default().title("Search").borders(Borders::ALL));
            f.render_widget(search, v_layout[0]);

            let (list_area, details_area) = if show_details {
                let h = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        // left pane is more important in smaller
                        // windows
                        Constraint::Min(40),
                        Constraint::Percentage(60),
                    ])
                    .split(v_layout[1]);
                (h[0], Some(h[1]))
            } else {
                (v_layout[1], None)
            };

            let items: Vec<ListItem> = filtered
                .iter()
                .map(|(id, t)| {
                    let idx = Span::styled(
                        format!("{:0>width$} ", id),
                        Style::default().fg(Color::DarkGray),
                    );
                    let label = highlight_match(t.label.as_str(), &query);
                    let mut spans = vec![idx];
                    spans.extend(label.spans);
                    ListItem::new(Line::from(spans))
                })
                .collect();
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(filter_title(scope))
                        .borders(Borders::ALL),
                )
                .highlight_symbol("❯ ")
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_stateful_widget(list, list_area, &mut list_state);

            if let Some(area) = details_area {
                let selected_id = filtered.get(selected).map(|(id, _)| *id);
                let cmd_lines: Vec<Line> = filtered
                    .iter()
                    .map(|(id, t)| {
                        let is_selected = Some(*id) == selected_id;
                        let inputs = collection.required_inputs_for_task(*id).unwrap_or_default();
                        let input_hint = if inputs.is_empty() {
                            String::new()
                        } else {
                            let names: Vec<&str> = inputs
                                .iter()
                                .map(|i| i.description.as_deref().unwrap_or(i.id.as_str()))
                                .collect();
                            format!("  [{}]", names.join(", "))
                        };
                        let cmd_style = if is_selected {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::Cyan)
                        };
                        Line::from(vec![
                            Span::styled(t.command.as_str(), cmd_style),
                            Span::styled(input_hint, Style::default().fg(Color::DarkGray)),
                        ])
                    })
                    .collect();

                let details = Paragraph::new(cmd_lines)
                    .block(Block::default().title("commands").borders(Borders::ALL));
                f.render_widget(details, area);
            }

            let mut hints = vec![("tab/⇧tab", "filter")];
            hints.push(if show_details {
                ("?", "hide details")
            } else {
                ("?", "show details")
            });
            f.render_widget(footer(&hints), v_layout[2]);
        })?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Tab => {
                    scope = next_filter(scope);
                    selected = 0;
                }
                KeyCode::BackTab => {
                    scope = prev_filter(scope);
                    selected = 0;
                }
                KeyCode::Char('?') => {
                    show_details = !show_details;
                }
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
