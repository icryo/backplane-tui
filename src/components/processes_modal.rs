use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Row, Table},
};

use crate::ui::{centered_modal, Theme};

/// Truncate string to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

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
        // Wider modal for process table with command
        let modal_area = centered_modal(area, 100, 22);

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

        // ps aux columns: USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND
        // We show: PID, %CPU, %MEM, USER, COMMAND (index 1, 2, 3, 0, 10+)
        let header = self.processes.first().cloned().unwrap_or_default();
        let header_row = Row::new(vec![
            Text::from(header.get(1).map(|s| s.as_str()).unwrap_or("PID").to_string()),
            Text::from(header.get(2).map(|s| s.as_str()).unwrap_or("%CPU").to_string()),
            Text::from(header.get(3).map(|s| s.as_str()).unwrap_or("%MEM").to_string()),
            Text::from(header.get(0).map(|s| s.as_str()).unwrap_or("USER").to_string()),
            Text::from("COMMAND".to_string()),
        ])
        .style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD));

        let rows: Vec<Row> = self.processes
            .iter()
            .skip(1) // Skip header
            .skip(self.scroll)
            .take(17) // Max visible rows
            .map(|proc| {
                // Get command - it's everything from index 10 onwards (joined)
                let command = if proc.len() > 10 {
                    proc[10..].join(" ")
                } else {
                    proc.last().cloned().unwrap_or_default()
                };
                let cmd_display = if command.len() > 55 {
                    format!("{}...", &command[..52])
                } else {
                    command
                };

                Row::new(vec![
                    Text::from(proc.get(1).cloned().unwrap_or_default()), // PID
                    Text::from(proc.get(2).cloned().unwrap_or_default()), // %CPU
                    Text::from(proc.get(3).cloned().unwrap_or_default()), // %MEM
                    Text::from(truncate(proc.get(0).map(|s| s.as_str()).unwrap_or(""), 10)), // USER
                    Text::from(cmd_display), // COMMAND
                ])
                .style(Style::default().fg(Theme::FG))
            })
            .collect();

        let widths = [
            Constraint::Length(8),  // PID
            Constraint::Length(6),  // %CPU
            Constraint::Length(6),  // %MEM
            Constraint::Length(10), // USER
            Constraint::Min(20),    // COMMAND (flexible)
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
