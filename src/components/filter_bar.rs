use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::Theme;

/// Filter bar component for fuzzy searching containers
#[derive(Debug, Clone, Default)]
pub struct FilterBar {
    pub query: String,
    pub active: bool,
}

impl FilterBar {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            active: false,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.query.clear();
    }

    pub fn type_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn backspace(&mut self) {
        self.query.pop();
    }

    pub fn clear(&mut self) {
        self.query.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    /// Check if a container name matches the filter (fuzzy)
    pub fn matches(&self, name: &str) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let query_lower = self.query.to_lowercase();
        let name_lower = name.to_lowercase();

        // Simple substring match (can be enhanced to true fuzzy)
        name_lower.contains(&query_lower)
    }

    /// Get match positions for highlighting
    pub fn match_positions(&self, name: &str) -> Vec<usize> {
        if self.query.is_empty() {
            return vec![];
        }

        let query_lower = self.query.to_lowercase();
        let name_lower = name.to_lowercase();

        let mut positions = Vec::new();
        if let Some(start) = name_lower.find(&query_lower) {
            for i in start..(start + query_lower.len()) {
                positions.push(i);
            }
        }
        positions
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, match_count: usize, total_count: usize) {
        if !self.active {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::CYAN))
            .style(Style::default().bg(Theme::BG_DARK));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let cursor = if self.query.is_empty() { "│" } else { "" };

        let text = Line::from(vec![
            Span::styled(" / ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(&self.query, Style::default().fg(Theme::FG)),
            Span::styled(cursor, Style::default().fg(Theme::CYAN)),
            Span::styled(
                format!("  ({}/{})", match_count, total_count),
                Style::default().fg(Theme::FG_DARK),
            ),
        ]);

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, inner);
    }
}
