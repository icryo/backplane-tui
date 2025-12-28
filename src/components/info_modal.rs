use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::models::ContainerInfo;
use crate::ui::{centered_modal, status_color, status_icon, Theme};
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
        // Dynamic height based on content
        let modal_height = match container {
            Some(c) => {
                let port_lines = if c.ports.is_empty() { 1 } else { c.ports.len().min(4) };
                22 + port_lines as u16
            }
            None => 8,
        };
        let modal_area = centered_modal(area, 65, modal_height);

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
                let sparkline_width = 28;

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

                // Container ID (short)
                let short_id = if c.id.len() >= 12 { &c.id[..12] } else { &c.id };

                // Uptime
                let uptime = format_uptime(c.created);

                // Type
                let type_str = if c.is_cli { "CLI" } else { "Web" };

                let mut lines = vec![
                    // Header section
                    Line::from(vec![
                        Span::styled(format!(" {} ", status_icon(&c.status)), Style::default().fg(status_color(&c.status))),
                        Span::styled(&c.name, Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("  ({})", type_str), Style::default().fg(Theme::FG_DARK)),
                    ]),
                    Line::raw(""),
                    // Container details section
                    Line::styled("── Container Details ──", Style::default().fg(Theme::OVERLAY)),
                    Line::from(vec![
                        Span::styled("  Image:   ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(&c.image, Style::default().fg(Theme::LAVENDER)),
                    ]),
                    Line::from(vec![
                        Span::styled("  ID:      ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(short_id, Style::default().fg(Theme::OVERLAY)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Status:  ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(c.status.as_str(), Style::default().fg(status_color(&c.status))),
                        Span::styled("  │  Uptime: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(&uptime, Style::default().fg(Theme::SKY)),
                    ]),
                    Line::raw(""),
                    // Ports section
                    Line::styled("── Ports ──", Style::default().fg(Theme::OVERLAY)),
                ];

                // Add port lines
                if c.ports.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled("No ports exposed", Style::default().fg(Theme::FG_DARK)),
                    ]));
                } else {
                    for (i, port) in c.ports.iter().take(4).enumerate() {
                        let port_line = if let Some(host_port) = port.host_port {
                            Line::from(vec![
                                Span::styled("  ", Style::default()),
                                Span::styled(format!("{}", host_port), Style::default().fg(Theme::GREEN)),
                                Span::styled(" → ", Style::default().fg(Theme::FG_DARK)),
                                Span::styled(format!("{}", port.container_port), Style::default().fg(Theme::YELLOW)),
                                Span::styled(format!("/{}", port.protocol), Style::default().fg(Theme::FG_DARK)),
                            ])
                        } else {
                            Line::from(vec![
                                Span::styled("  ", Style::default()),
                                Span::styled(format!("{}", port.container_port), Style::default().fg(Theme::YELLOW)),
                                Span::styled(format!("/{}", port.protocol), Style::default().fg(Theme::FG_DARK)),
                                Span::styled(" (not exposed)", Style::default().fg(Theme::FG_DARK)),
                            ])
                        };
                        lines.push(port_line);
                        if i == 3 && c.ports.len() > 4 {
                            lines.push(Line::styled(
                                format!("  ... and {} more", c.ports.len() - 4),
                                Style::default().fg(Theme::FG_DARK),
                            ));
                        }
                    }
                }

                lines.extend(vec![
                    Line::raw(""),
                    // Resource usage section
                    Line::styled("── Resource Usage ──", Style::default().fg(Theme::OVERLAY)),
                    Line::from(vec![
                        Span::styled("  CPU:    ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(&cpu_spark, Style::default().fg(Theme::CYAN)),
                        Span::styled(format!(" {:>5.1}%", cpu_pct), Style::default().fg(percent_color(cpu_pct))),
                    ]),
                    Line::from(vec![
                        Span::styled("  Memory: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled(&mem_spark, Style::default().fg(Theme::MAGENTA)),
                        Span::styled(format!(" {:>5.1}% ({:.0}MB)", mem_pct, mem_mb), Style::default().fg(percent_color(mem_pct))),
                    ]),
                    Line::raw(""),
                    // Network I/O section
                    Line::styled("── Network I/O ──", Style::default().fg(Theme::OVERLAY)),
                    Line::from(vec![
                        Span::styled("  RX: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled("↓ ", Style::default().fg(Theme::GREEN)),
                        Span::styled(format!("{:<10}", rx_str), Style::default().fg(Theme::FG)),
                        Span::styled(format!("({}/s)", rx_rate), Style::default().fg(Theme::GREEN)),
                    ]),
                    Line::from(vec![
                        Span::styled("  TX: ", Style::default().fg(Theme::FG_DARK)),
                        Span::styled("↑ ", Style::default().fg(Theme::PEACH)),
                        Span::styled(format!("{:<10}", tx_str), Style::default().fg(Theme::FG)),
                        Span::styled(format!("({}/s)", tx_rate), Style::default().fg(Theme::PEACH)),
                    ]),
                    Line::raw(""),
                    Line::styled("                    Press Esc or i to close", Style::default().fg(Theme::FG_DARK)),
                ]);

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

/// Format uptime from created timestamp
fn format_uptime(created: Option<i64>) -> String {
    match created {
        Some(ts) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let secs = now - ts;
            if secs < 0 {
                return "-".to_string();
            }
            let secs = secs as u64;
            let days = secs / 86400;
            let hours = (secs % 86400) / 3600;
            let mins = (secs % 3600) / 60;

            if days > 0 {
                format!("{}d {}h", days, hours)
            } else if hours > 0 {
                format!("{}h {}m", hours, mins)
            } else if mins > 0 {
                format!("{}m", mins)
            } else {
                format!("{}s", secs)
            }
        }
        None => "-".to_string(),
    }
}
