use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::{centered_modal, Theme};

/// Rename container modal
#[derive(Debug, Clone)]
pub struct RenameModal {
    pub container_name: String,
    pub new_name: String,
}

impl RenameModal {
    pub fn new(container_name: String) -> Self {
        Self {
            new_name: container_name.clone(),
            container_name,
        }
    }

    pub fn handle_char(&mut self, c: char) {
        // Only allow valid container name characters
        if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
            self.new_name.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        self.new_name.pop();
    }

    pub fn is_valid(&self) -> bool {
        !self.new_name.is_empty() && self.new_name != self.container_name
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let modal_area = centered_modal(area, 55, 10);

        // Clear background
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(format!(" Rename: {} ", self.container_name))
            .title_style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MAGENTA))
            .style(Style::default().bg(Theme::BG_DARK));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Split for input and instructions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(inner);

        // Label
        let label = Paragraph::new(" New name:")
            .style(Style::default().fg(Theme::FG_DARK));
        frame.render_widget(label, chunks[0]);

        // Input field with cursor
        let input_text = format!(" {}█", self.new_name);
        let input_style = if self.is_valid() {
            Style::default().fg(Theme::GREEN)
        } else {
            Style::default().fg(Theme::YELLOW)
        };
        let input = Paragraph::new(input_text)
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Theme::BORDER)));
        frame.render_widget(input, chunks[1]);

        // Instructions
        let instructions = Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("rename   ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(" Esc ", Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)),
            Span::styled("cancel", Style::default().fg(Theme::FG_DARK)),
        ]);
        let instructions_widget = Paragraph::new(instructions).alignment(Alignment::Center);
        frame.render_widget(instructions_widget, chunks[3]);
    }
}
