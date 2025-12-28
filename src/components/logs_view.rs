use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::ui::border_style;

/// Logs view component
pub struct LogsView {
    pub scroll: usize,
    pub follow: bool,
    pub focused: bool,
}

impl LogsView {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            follow: true,
            focused: false,
        }
    }

    /// Scroll up
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
        self.follow = false;
    }

    /// Scroll down
    pub fn scroll_down(&mut self, amount: usize, max: usize) {
        self.scroll = (self.scroll + amount).min(max);
    }

    /// Go to top
    pub fn top(&mut self) {
        self.scroll = 0;
        self.follow = false;
    }

    /// Go to bottom and enable follow mode
    pub fn bottom(&mut self, max: usize) {
        self.scroll = max;
        self.follow = true;
    }

    /// Toggle follow mode
    pub fn toggle_follow(&mut self) {
        self.follow = !self.follow;
    }

    /// Update logs (auto-scroll if following)
    pub fn update_logs(&mut self, log_count: usize, visible_lines: usize) {
        if self.follow && log_count > visible_lines {
            self.scroll = log_count.saturating_sub(visible_lines);
        }
    }

    /// Render the logs view
    pub fn render(&mut self, frame: &mut Frame, area: Rect, logs: &[String], container_name: &str) {
        let block = Block::default()
            .title(format!(
                " Logs: {} {} ",
                container_name,
                if self.follow { "[following]" } else { "" }
            ))
            .borders(Borders::ALL)
            .border_style(border_style(self.focused));

        let inner = block.inner(area);
        let visible_height = inner.height as usize;

        // Update scroll position if following
        self.update_logs(logs.len(), visible_height);

        // Get visible logs
        let visible_logs: Vec<Line> = logs
            .iter()
            .skip(self.scroll)
            .take(visible_height)
            .map(|line| {
                // Parse timestamp if present and style it
                if line.len() > 30 && line.chars().nth(4) == Some('-') {
                    let (timestamp, rest) = line.split_at(30.min(line.len()));
                    Line::from(vec![
                        Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                        Span::raw(rest),
                    ])
                } else {
                    Line::raw(line)
                }
            })
            .collect();

        let paragraph = Paragraph::new(visible_logs).block(block);

        frame.render_widget(paragraph, area);

        // Render scrollbar
        if logs.len() > visible_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));

            let mut scrollbar_state = ScrollbarState::new(logs.len().saturating_sub(visible_height))
                .position(self.scroll);

            frame.render_stateful_widget(
                scrollbar,
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }
}

impl Default for LogsView {
    fn default() -> Self {
        Self::new()
    }
}
