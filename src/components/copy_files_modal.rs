use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::{centered_modal, Theme};

/// Copy direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CopyDirection {
    ToContainer,   // Host -> Container
    FromContainer, // Container -> Host
}

/// Copy files modal
#[derive(Debug, Clone)]
pub struct CopyFilesModal {
    pub container_name: String,
    pub direction: CopyDirection,
    pub host_path: String,
    pub container_path: String,
    pub active_field: usize, // 0 = direction, 1 = host_path, 2 = container_path
}

impl CopyFilesModal {
    pub fn new(container_name: String) -> Self {
        Self {
            container_name,
            direction: CopyDirection::FromContainer,
            host_path: String::new(),
            container_path: String::new(),
            active_field: 1,
        }
    }

    pub fn toggle_direction(&mut self) {
        self.direction = match self.direction {
            CopyDirection::ToContainer => CopyDirection::FromContainer,
            CopyDirection::FromContainer => CopyDirection::ToContainer,
        };
    }

    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % 3;
    }

    pub fn prev_field(&mut self) {
        if self.active_field == 0 {
            self.active_field = 2;
        } else {
            self.active_field -= 1;
        }
    }

    pub fn handle_char(&mut self, c: char) {
        match self.active_field {
            0 => self.toggle_direction(),
            1 => self.host_path.push(c),
            2 => self.container_path.push(c),
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.active_field {
            1 => { self.host_path.pop(); }
            2 => { self.container_path.pop(); }
            _ => {}
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.host_path.is_empty() && !self.container_path.is_empty()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let modal_area = centered_modal(area, 65, 16);

        // Clear background
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(format!(" Copy Files: {} ", self.container_name))
            .title_style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MAGENTA))
            .style(Style::default().bg(Theme::BG_DARK));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Direction
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Host path
                Constraint::Length(3), // Container path
                Constraint::Min(0),    // Instructions
            ])
            .split(inner);

        // Direction toggle
        let direction_str = match self.direction {
            CopyDirection::ToContainer => "  Host → Container  ",
            CopyDirection::FromContainer => "  Container → Host  ",
        };
        let dir_style = if self.active_field == 0 {
            Style::default().fg(Theme::BG_DARK).bg(Theme::MAUVE).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::MAUVE)
        };
        let direction_widget = Paragraph::new(direction_str)
            .style(dir_style)
            .alignment(Alignment::Center);
        frame.render_widget(direction_widget, chunks[0]);

        // Host path
        let host_label = if self.direction == CopyDirection::ToContainer { "Source (host):" } else { "Destination (host):" };
        let host_active = self.active_field == 1;
        self.render_input_field(frame, chunks[2], host_label, &self.host_path, host_active);

        // Container path
        let container_label = if self.direction == CopyDirection::ToContainer { "Destination (container):" } else { "Source (container):" };
        let container_active = self.active_field == 2;
        self.render_input_field(frame, chunks[3], container_label, &self.container_path, container_active);

        // Instructions
        let instructions = Line::from(vec![
            Span::styled(" Tab ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("next   ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(" Enter ", Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("copy   ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(" Esc ", Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)),
            Span::styled("cancel", Style::default().fg(Theme::FG_DARK)),
        ]);
        let instructions_widget = Paragraph::new(instructions).alignment(Alignment::Center);
        frame.render_widget(instructions_widget, chunks[4]);
    }

    fn render_input_field(&self, frame: &mut Frame, area: Rect, label: &str, value: &str, active: bool) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(26), Constraint::Min(0)])
            .split(area);

        let label_widget = Paragraph::new(format!(" {}", label))
            .style(Style::default().fg(Theme::FG_DARK));
        frame.render_widget(label_widget, chunks[0]);

        let input_text = if active {
            format!(" {}█", value)
        } else {
            format!(" {}", value)
        };
        let border_style = if active {
            Style::default().fg(Theme::CYAN)
        } else {
            Style::default().fg(Theme::BORDER)
        };
        let input = Paragraph::new(input_text)
            .style(Style::default().fg(Theme::FG))
            .block(Block::default().borders(Borders::ALL).border_style(border_style));
        frame.render_widget(input, chunks[1]);
    }
}
