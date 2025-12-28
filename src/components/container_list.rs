use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::ListViewMode;
use crate::models::ContainerInfo;
use crate::ui::{border_style, selected_style, status_color, status_icon, Theme, title_style};

/// Container list component (full-width with inline stats)
pub struct ContainerList {
    pub state: ListState,
    pub focused: bool,
}

impl ContainerList {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            focused: true,
        }
    }

    /// Move selection up
    pub fn previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Move selection down
    pub fn next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Go to top
    pub fn top(&mut self) {
        self.state.select(Some(0));
    }

    /// Go to bottom
    pub fn bottom(&mut self, len: usize) {
        if len > 0 {
            self.state.select(Some(len - 1));
        }
    }

    /// Get currently selected index
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Render the container list (full-width with inline stats)
    pub fn render(&mut self, frame: &mut Frame, area: Rect, containers: &[ContainerInfo], view_mode: ListViewMode) {
        let items: Vec<ListItem> = containers
            .iter()
            .map(|c| {
                let icon = status_icon(&c.status);

                let line = match view_mode {
                    ListViewMode::Stats => self.render_stats_line(c, icon),
                    ListViewMode::Network => self.render_network_line(c, icon),
                    ListViewMode::Details => self.render_details_line(c, icon),
                };

                ListItem::new(line)
            })
            .collect();

        let view_label = match view_mode {
            ListViewMode::Stats => "Stats",
            ListViewMode::Network => "Network",
            ListViewMode::Details => "Details",
        };
        let title = format!(" Containers ({}) │ {} ←→ ", containers.len(), view_label);
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .title_style(title_style(self.focused))
                    .borders(Borders::ALL)
                    .border_style(border_style(self.focused)),
            )
            .highlight_style(selected_style())
            .highlight_symbol("▶");

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    /// Render Stats view line: Name, Type, Port, CPU bar, MEM bar
    fn render_stats_line(&self, c: &ContainerInfo, icon: &str) -> Line<'static> {
        let type_indicator = if c.is_cli { "CLI" } else { "WEB" };

        // Format ports (show first port or "-")
        let port_str = if c.ports.is_empty() {
            "-".to_string()
        } else if c.ports.len() == 1 {
            c.ports[0].display()
        } else {
            format!("{}+{}", c.ports[0].display(), c.ports.len() - 1)
        };

        // CPU/MEM bars and values (only if running with stats)
        let (cpu_bar, cpu_val, mem_bar, mem_val) = if let Some(stats) = &c.stats {
            let cpu_bar = make_bar(stats.cpu_percent, 8);
            let cpu_val = format!("{:>5.1}%", stats.cpu_percent);
            let mem_bar = make_bar(stats.memory_percent, 8);
            let mem_val = format!("{:>5.1}%", stats.memory_percent);
            (cpu_bar, cpu_val, mem_bar, mem_val)
        } else if c.status.is_running() {
            ("        ".to_string(), "  ... ".to_string(),
             "        ".to_string(), "  ... ".to_string())
        } else {
            ("────────".to_string(), "   -  ".to_string(),
             "────────".to_string(), "   -  ".to_string())
        };

        let cpu_color = percent_color(c.stats.as_ref().map(|s| s.cpu_percent).unwrap_or(0.0));
        let mem_color = percent_color(c.stats.as_ref().map(|s| s.memory_percent).unwrap_or(0.0));

        Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(status_color(&c.status))),
            Span::styled(format!("{:<20}", truncate_name(&c.name, 20)), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {:>3} ", type_indicator), Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:<14}", truncate_name(&port_str, 14)), Style::default().fg(Theme::YELLOW)),
            Span::styled(" CPU ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(cpu_bar, Style::default().fg(Theme::CYAN)),
            Span::styled(cpu_val, Style::default().fg(cpu_color)),
            Span::styled(" MEM ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(mem_bar, Style::default().fg(Theme::MAGENTA)),
            Span::styled(mem_val, Style::default().fg(mem_color)),
        ])
    }

    /// Render Network view line: Name, ↓RX rate, ↑TX rate, Total RX, Total TX
    fn render_network_line(&self, c: &ContainerInfo, icon: &str) -> Line<'static> {
        let (rx_rate, tx_rate, rx_total, tx_total) = if let Some(stats) = &c.stats {
            (
                format_bytes_rate(stats.net_rx_rate),
                format_bytes_rate(stats.net_tx_rate),
                format_bytes(stats.net_rx_bytes),
                format_bytes(stats.net_tx_bytes),
            )
        } else if c.status.is_running() {
            ("...".to_string(), "...".to_string(), "...".to_string(), "...".to_string())
        } else {
            ("-".to_string(), "-".to_string(), "-".to_string(), "-".to_string())
        };

        Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(status_color(&c.status))),
            Span::styled(format!("{:<20}", truncate_name(&c.name, 20)), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" ↓ ", Style::default().fg(Theme::GREEN)),
            Span::styled(format!("{:>10}", rx_rate), Style::default().fg(Theme::GREEN)),
            Span::styled(" ↑ ", Style::default().fg(Theme::PEACH)),
            Span::styled(format!("{:>10}", tx_rate), Style::default().fg(Theme::PEACH)),
            Span::styled("  Total↓ ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:>8}", rx_total), Style::default().fg(Theme::TEAL)),
            Span::styled("  Total↑ ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:>8}", tx_total), Style::default().fg(Theme::FLAMINGO)),
        ])
    }

    /// Render Details view line: Name, Image, Container ID, Uptime
    fn render_details_line(&self, c: &ContainerInfo, icon: &str) -> Line<'static> {
        let short_id = if c.id.len() >= 12 { &c.id[..12] } else { &c.id };
        let uptime = format_uptime(c.created);

        Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(status_color(&c.status))),
            Span::styled(format!("{:<20}", truncate_name(&c.name, 20)), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" Image: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:<24}", truncate_name(&c.image, 24)), Style::default().fg(Theme::LAVENDER)),
            Span::styled(" ID: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(short_id.to_string(), Style::default().fg(Theme::OVERLAY)),
            Span::styled(" Up: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:>12}", uptime), Style::default().fg(Theme::SKY)),
        ])
    }
}

/// Create a progress bar string
fn make_bar(percent: f64, width: usize) -> String {
    const FULL: char = '█';
    const PARTIAL: &[char] = &[' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    const EMPTY: char = '░';

    let clamped = percent.clamp(0.0, 100.0);
    let filled_width = (clamped / 100.0) * width as f64;
    let full_blocks = filled_width as usize;
    let remainder = filled_width - full_blocks as f64;
    let partial_idx = (remainder * 8.0).round() as usize;

    let mut bar = String::new();

    for i in 0..width {
        if i < full_blocks {
            bar.push(FULL);
        } else if i == full_blocks && partial_idx > 0 {
            bar.push(PARTIAL[partial_idx]);
        } else {
            bar.push(EMPTY);
        }
    }

    bar
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

/// Truncate a name to fit in the given width
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}…", &name[..max_len - 1])
    }
}

impl Default for ContainerList {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes as human readable (KB, MB, GB)
fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Format bytes per second as human readable rate
fn format_bytes_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1.0 {
        return "0 B/s".to_string();
    }
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.1}GB/s", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.1}MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.1}KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0}B/s", bytes_per_sec)
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
