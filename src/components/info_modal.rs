use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::models::ContainerInfo;
use crate::ui::{centered_modal, Theme};
use crate::components::sparkline::StatsHistory;

/// Network/Info modal component
pub struct InfoModal;

impl InfoModal {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        container: Option<&ContainerInfo>,
        stats_history: &StatsHistory,
    ) {
        let modal_area = centered_modal(area, 60, 16);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(" Container Info ")
            .title_style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MAGENTA))
            .style(Style::default().bg(Theme::BG_DARK));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        match container {
            Some(c) => {
                let sparkline_width = 30;

                // Get sparkline data
                let cpu_history = stats_history.get_cpu(&c.name);
                let mem_history = stats_history.get_mem(&c.name);
                let cpu_spark = StatsHistory::to_sparkline(cpu_history, sparkline_width);
                let mem_spark = StatsHistory::to_sparkline(mem_history, sparkline_width);

                // Network stats
                let (rx_str, tx_str, rx_rate, tx_rate) = if let Some(stats) = &c.stats {
                    (
                        format_bytes(stats.net_rx_bytes),
                        format_bytes(stats.net_tx_bytes),
                        format_rate(stats.net_rx_rate),
                        format_rate(stats.net_tx_rate),
                    )
                } else {
                    ("-".to_string(), "-".to_string(), "-".to_string(), "-".to_string())
                };

                let cpu_pct = c.stats.as_ref().map(|s| s.cpu_percent).unwrap_or(0.0);
                let mem_pct = c.stats.as_ref().map(|s| s.memory_percent).unwrap_or(0.0);
                let mem_mb = c.stats.as_ref().map(|s| s.memory_usage_mb).unwrap_or(0.0);

                let lines = vec![
                    Line::from(vec![
                        Span::styled("Container: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(&c.name, Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::raw(""),
                    Line::styled("── Resource Usage ──", Style::default().fg(Theme::FG_DARK)),
                    Line::from(vec![
                        Span::styled("CPU:    ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(&cpu_spark, Style::default().fg(Theme::CYAN)),
                        Span::styled(format!(" {:>5.1}%", cpu_pct), Style::default().fg(percent_color(cpu_pct))),
                    ]),
                    Line::from(vec![
                        Span::styled("Memory: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(&mem_spark, Style::default().fg(Theme::MAGENTA)),
                        Span::styled(format!(" {:>5.1}% ({:.0}MB)", mem_pct, mem_mb), Style::default().fg(percent_color(mem_pct))),
                    ]),
                    Line::raw(""),
                    Line::styled("── Network I/O ──", Style::default().fg(Theme::FG_DARK)),
                    Line::from(vec![
                        Span::styled("RX: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled("↓ ", Style::default().fg(Theme::GREEN)),
                        Span::styled(format!("{:<12}", rx_str), Style::default().fg(Theme::FG)),
                        Span::styled(format!("({}/s)", rx_rate), Style::default().fg(Theme::FG_DARK)),
                    ]),
                    Line::from(vec![
                        Span::styled("TX: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled("↑ ", Style::default().fg(Theme::YELLOW)),
                        Span::styled(format!("{:<12}", tx_str), Style::default().fg(Theme::FG)),
                        Span::styled(format!("({}/s)", tx_rate), Style::default().fg(Theme::FG_DARK)),
                    ]),
                    Line::raw(""),
                    Line::raw(""),
                    Line::styled("Press Esc or i to close", Style::default().fg(Theme::FG_DARK)),
                ];

                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, inner);
            }
            None => {
                let text = Paragraph::new("No container selected")
                    .style(Style::default().fg(Theme::FG_DARK));
                frame.render_widget(text, inner);
            }
        }
    }
}

/// Format bytes to human readable
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format rate to human readable
fn format_rate(rate: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    if rate >= MB {
        format!("{:.1} MB", rate / MB)
    } else if rate >= KB {
        format!("{:.1} KB", rate / KB)
    } else {
        format!("{:.0} B", rate)
    }
}

/// Get color based on percentage
fn percent_color(percent: f64) -> Color {
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
