use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Shared footer bar used across all fuzzy-finder TUIs.
pub fn footer<'a>(extra_hints: &[(&'static str, &'static str)]) -> Paragraph<'a> {
    let mut spans = vec![
        Span::styled(
            format!(" Aliasx v{} ", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" | "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" navigate  "),
        Span::styled("↵", Style::default().fg(Color::Yellow)),
        Span::raw(" select  "),
        Span::styled("esc", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ];

    for (key, label) in extra_hints {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(*key, Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(format!(" {}", label)));
    }

    Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
}

/// Build a [`Line`] with the `query` substring highlighted wherever it appears
/// in `text` (case-insensitive). Non-matching parts are rendered as plain text.
pub fn highlight_match<'a>(text: &'a str, query: &str) -> Line<'a> {
    if query.is_empty() {
        return Line::from(text);
    }

    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    let match_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);

    let mut spans = Vec::new();
    let mut pos = 0;

    while let Some(start) = lower_text[pos..].find(&lower_query) {
        let abs_start = pos + start;
        let abs_end = abs_start + lower_query.len();

        if abs_start > pos {
            spans.push(Span::raw(&text[pos..abs_start]));
        }
        spans.push(Span::styled(&text[abs_start..abs_end], match_style));
        pos = abs_end;
    }

    if pos < text.len() {
        spans.push(Span::raw(&text[pos..]));
    }

    Line::from(spans)
}
