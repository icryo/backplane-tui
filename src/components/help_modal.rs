use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::{centered_modal, Theme};

/// Help modal component
pub struct HelpModal;

impl HelpModal {
    pub fn render(frame: &mut Frame, area: Rect) {
        let modal_area = centered_modal(area, 60, 24);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        let help_text = vec![
            Line::styled("Keyboard Shortcuts", Style::default().bold().fg(Color::Cyan)),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  j/↓    ", Style::default().fg(Color::Yellow)),
                Span::raw("Move down"),
            ]),
            Line::from(vec![
                Span::styled("  k/↑    ", Style::default().fg(Color::Yellow)),
                Span::raw("Move up"),
            ]),
            Line::from(vec![
                Span::styled("  g      ", Style::default().fg(Color::Yellow)),
                Span::raw("Go to top"),
            ]),
            Line::from(vec![
                Span::styled("  G      ", Style::default().fg(Color::Yellow)),
                Span::raw("Go to bottom"),
            ]),
            Line::from(vec![
                Span::styled("  ←/→    ", Style::default().fg(Color::Yellow)),
                Span::raw("Switch view (Stats/Network/Details)"),
            ]),
            Line::from(vec![
                Span::styled("  f      ", Style::default().fg(Color::Yellow)),
                Span::raw("Filter (All/Groups/Running/Stopped)"),
            ]),
            Line::from(vec![
                Span::styled("  /      ", Style::default().fg(Color::Yellow)),
                Span::raw("Filter by name"),
            ]),
            Line::from(vec![
                Span::styled("  Enter/l", Style::default().fg(Color::Yellow)),
                Span::raw("View logs"),
            ]),
            Line::from(vec![
                Span::styled("  e      ", Style::default().fg(Color::Yellow)),
                Span::raw("Exec shell into container"),
            ]),
            Line::from(vec![
                Span::styled("  n      ", Style::default().fg(Color::Yellow)),
                Span::raw("New container"),
            ]),
            Line::from(vec![
                Span::styled("  s      ", Style::default().fg(Color::Yellow)),
                Span::raw("Start container"),
            ]),
            Line::from(vec![
                Span::styled("  x      ", Style::default().fg(Color::Yellow)),
                Span::raw("Stop container"),
            ]),
            Line::from(vec![
                Span::styled("  R      ", Style::default().fg(Color::Yellow)),
                Span::raw("Restart container"),
            ]),
            Line::from(vec![
                Span::styled("  d      ", Style::default().fg(Color::Yellow)),
                Span::raw("Delete container"),
            ]),
            Line::from(vec![
                Span::styled("  r      ", Style::default().fg(Color::Yellow)),
                Span::raw("Refresh list"),
            ]),
            Line::from(vec![
                Span::styled("  Esc    ", Style::default().fg(Color::Yellow)),
                Span::raw("Back / Close modal"),
            ]),
            Line::from(vec![
                Span::styled("  q      ", Style::default().fg(Color::Yellow)),
                Span::raw("Quit"),
            ]),
            Line::raw(""),
            Line::styled("Press Esc to close", Style::default().fg(Color::DarkGray)),
        ];

        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MODAL_BORDER))
            .style(Style::default().bg(Theme::MODAL_BG));

        let paragraph = Paragraph::new(help_text).block(block);

        frame.render_widget(paragraph, modal_area);
    }
}
