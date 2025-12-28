use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::{centered_modal, Theme};

/// Confirm action modal component
pub struct ConfirmModal;

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Delete(String),
    Stop(String),
}

impl ConfirmModal {
    pub fn render(frame: &mut Frame, area: Rect, action: &ConfirmAction) {
        let modal_area = centered_modal(area, 50, 8);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        let (title, message) = match action {
            ConfirmAction::Delete(name) => (
                " Confirm Delete ",
                format!("Are you sure you want to delete '{}'?\n\nThis action cannot be undone.", name),
            ),
            ConfirmAction::Stop(name) => (
                " Confirm Stop ",
                format!("Are you sure you want to stop '{}'?", name),
            ),
        };

        let text = vec![
            Line::raw(""),
            Line::styled(&message, Style::default().fg(Color::White)),
            Line::raw(""),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  y/Enter ", Style::default().fg(Color::Green)),
                Span::raw("Confirm    "),
                Span::styled("n/Esc ", Style::default().fg(Color::Red)),
                Span::raw("Cancel"),
            ]),
        ];

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MODAL_BORDER))
            .style(Style::default().bg(Theme::MODAL_BG));

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, modal_area);
    }
}
