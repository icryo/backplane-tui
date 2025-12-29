use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Row, Table},
};

use crate::ui::{centered_modal, Theme};

/// Container processes modal (docker top)
#[derive(Debug, Clone)]
pub struct ProcessesModal {
    pub container_name: String,
    pub processes: Vec<Vec<String>>,
    pub scroll: usize,
}

impl ProcessesModal {
    pub fn new(container_name: String, processes: Vec<Vec<String>>) -> Self {
        Self {
            container_name,
            processes,
            scroll: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        let max_scroll = self.processes.len().saturating_sub(1);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Larger modal for process table
        let modal_area = centered_modal(area, 90, 20);

        // Clear background
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(format!(" Processes: {} ", self.container_name))
            .title_style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MAGENTA))
            .style(Style::default().bg(Theme::BG_DARK));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        if self.processes.is_empty() {
            let msg = Paragraph::new("No processes running")
                .style(Style::default().fg(Theme::FG_DARK))
                .alignment(Alignment::Center);
            frame.render_widget(msg, inner);
            return;
        }

        // Split for table and instructions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(2)])
            .split(inner);

        // Build table rows
        let header = self.processes.first().cloned().unwrap_or_default();
        let header_row = Row::new(header.iter().take(6).map(|s| {
            let truncated = if s.len() > 12 { &s[..12] } else { s };
            Text::from(truncated.to_string())
        }))
        .style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD));

        let rows: Vec<Row> = self.processes
            .iter()
            .skip(1) // Skip header
            .skip(self.scroll)
            .take(15) // Max visible rows
            .map(|proc| {
                Row::new(proc.iter().take(6).map(|s| {
                    let truncated = if s.len() > 12 { &s[..12] } else { s };
                    Text::from(truncated.to_string())
                }))
                .style(Style::default().fg(Theme::FG))
            })
            .collect();

        let widths = [
            Constraint::Length(12), // USER
            Constraint::Length(8),  // PID
            Constraint::Length(6),  // %CPU
            Constraint::Length(6),  // %MEM
            Constraint::Length(10), // VSZ
            Constraint::Length(10), // RSS
        ];

        let table = Table::new(rows, widths)
            .header(header_row)
            .column_spacing(1);

        frame.render_widget(table, chunks[0]);

        // Instructions
        let total = self.processes.len().saturating_sub(1);
        let instructions = Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("scroll   ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(" Esc/t ", Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)),
            Span::styled("close   ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!(" [{}/{}] ", self.scroll + 1, total.max(1)), Style::default().fg(Theme::FG_DARK)),
        ]);
        let instructions_widget = Paragraph::new(instructions).alignment(Alignment::Center);
        frame.render_widget(instructions_widget, chunks[1]);
    }
}
