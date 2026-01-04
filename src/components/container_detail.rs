use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::models::ContainerInfo;
use crate::ui::{border_style, status_color, Theme};
use crate::components::sparkline::StatsHistory;

/// Container detail component (top of right pane)
pub struct ContainerDetail;

impl ContainerDetail {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        container: Option<&ContainerInfo>,
        stats_history: &StatsHistory,
    ) {
        let block = Block::default()
            .title(" Details ")
            .title_style(Style::default().fg(Theme::FG_DARK))
            .borders(Borders::ALL)
            .border_style(border_style(false));

        match container {
            Some(c) => {
                let inner = block.inner(area);
                frame.render_widget(block, area);

                // Split into info and stats sections
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(6), // Info (5 lines + padding)
                        Constraint::Min(0),    // Stats with sparklines
                    ])
                    .split(inner);

                // Container info
                let type_str = if c.is_cli { "CLI" } else { "Web" };

                // Format ports
                let ports_str = if c.ports.is_empty() {
                    "-".to_string()
                } else {
                    c.ports.iter()
                        .map(|p| p.display())
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let info_text = vec![
                    Line::from(vec![
                        Span::styled("Name:   ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(&c.name, Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("Image:  ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(truncate(&c.image, 40), Style::default().fg(Theme::FG)),
                    ]),
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(c.status.as_str(), Style::default().fg(status_color(&c.status))),
                    ]),
                    Line::from(vec![
                        Span::styled("Type:   ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(type_str, Style::default().fg(Theme::FG)),
                    ]),
                    Line::from(vec![
                        Span::styled("Ports:  ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(truncate(&ports_str, 40), Style::default().fg(Theme::YELLOW)),
                    ]),
                ];

                let info = Paragraph::new(info_text);
                frame.render_widget(info, chunks[0]);

                // Stats with sparklines (if running)
                if c.status.is_running() {
                    Self::render_stats(frame, chunks[1], c, stats_history);
                }
            }
            None => {
                let text = Paragraph::new("No container selected")
                    .style(Style::default().fg(Theme::FG_DARK))
                    .block(block);
                frame.render_widget(text, area);
            }
        }
    }

    fn render_stats(frame: &mut Frame, area: Rect, container: &ContainerInfo, history: &StatsHistory) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(2)])
            .split(area);

        // Get sparkline data
        let cpu_history = history.get_cpu(&container.name);
        let mem_history = history.get_mem(&container.name);

        let sparkline_width = 20;
        let cpu_spark = StatsHistory::to_sparkline(&cpu_history, sparkline_width);
        let mem_spark = StatsHistory::to_sparkline(&mem_history, sparkline_width);

        // CPU line with sparkline
        if let Some(stats) = &container.stats {
            let cpu_color = percent_color(stats.cpu_percent as f32);
            let cpu_line = Line::from(vec![
                Span::styled("CPU ", Style::default().fg(Theme::FG_DARK)),
                Span::styled(&cpu_spark, Style::default().fg(Theme::CYAN)),
                Span::styled(format!(" {:>5.1}%", stats.cpu_percent), Style::default().fg(cpu_color)),
            ]);
            frame.render_widget(Paragraph::new(cpu_line), chunks[0]);

            // Memory line with sparkline
            let mem_color = percent_color(stats.memory_percent as f32);
            let mem_line = Line::from(vec![
                Span::styled("MEM ", Style::default().fg(Theme::FG_DARK)),
                Span::styled(&mem_spark, Style::default().fg(Theme::MAGENTA)),
                Span::styled(
                    format!(" {:>5.0}MB ({:.0}%)", stats.memory_usage_mb, stats.memory_percent),
                    Style::default().fg(mem_color),
                ),
            ]);
            frame.render_widget(Paragraph::new(mem_line), chunks[1]);
        } else {
            let loading = Paragraph::new("Loading stats...")
                .style(Style::default().fg(Theme::FG_DARK));
            frame.render_widget(loading, chunks[0]);
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn percent_color(percent: f32) -> Color {
    if percent > 80.0 {
        Theme::RED
    } else if percent > 60.0 {
        Theme::ORANGE
    } else if percent > 40.0 {
        Theme::YELLOW
    } else {
        Theme::GREEN
    }
}
