use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::ui::{centered_modal, Theme};

/// Available shells for exec
pub const SHELLS: &[&str] = &["/bin/bash", "/bin/sh", "/bin/zsh", "/bin/ash"];

/// Exec shell modal
#[derive(Debug, Clone)]
pub struct ExecModal {
    pub container_name: String,
    pub selected: usize,
    pub state: ListState,
}

impl ExecModal {
    pub fn new(container_name: String) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            container_name,
            selected: 0,
            state,
        }
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % SHELLS.len();
        self.state.select(Some(self.selected));
    }

    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = SHELLS.len() - 1;
        } else {
            self.selected -= 1;
        }
        self.state.select(Some(self.selected));
    }

    pub fn selected_shell(&self) -> &'static str {
        SHELLS[self.selected]
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let modal_area = centered_modal(area, 50, 12);

        // Clear background
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(format!(" Exec into: {} ", self.container_name))
            .title_style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MAGENTA))
            .style(Style::default().bg(Theme::BG_DARK));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Shell selection list
        let items: Vec<ListItem> = SHELLS
            .iter()
            .map(|shell| {
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(*shell, Style::default().fg(Theme::FG)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Theme::SELECTION_BG)
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        // Split for list and instructions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(4), Constraint::Length(2)])
            .split(inner);

        frame.render_stateful_widget(list, chunks[0], &mut self.state);

        // Instructions
        let instructions = Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("exec  ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(" Esc ", Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)),
            Span::styled("cancel", Style::default().fg(Theme::FG_DARK)),
        ]);
        let instructions_widget = ratatui::widgets::Paragraph::new(instructions)
            .alignment(Alignment::Center);
        frame.render_widget(instructions_widget, chunks[1]);
    }
}
