use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::{ListViewMode, StatusFilter};
use crate::models::ContainerInfo;
use crate::ui::{border_style, selected_style, status_color, status_icon, Theme, title_style};

/// Container list component (full-width with inline stats)
pub struct ContainerList {
    pub state: ListState,
    pub focused: bool,
    /// When in Groups mode, maps visual index to container index (None = header row)
    item_to_container: Vec<Option<usize>>,
}

impl ContainerList {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            focused: true,
            item_to_container: Vec::new(),
        }
    }

    /// Move selection up (skips header rows in groups mode)
    pub fn previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let mut i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        // Skip header rows (where item_to_container is None)
        if !self.item_to_container.is_empty() {
            let start = i;
            while self.item_to_container.get(i).copied().flatten().is_none() {
                i = if i == 0 { len - 1 } else { i - 1 };
                if i == start {
                    break; // Avoid infinite loop
                }
            }
        }
        self.state.select(Some(i));
    }

    /// Move selection down (skips header rows in groups mode)
    pub fn next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let mut i = match self.state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        // Skip header rows (where item_to_container is None)
        if !self.item_to_container.is_empty() {
            let start = i;
            while self.item_to_container.get(i).copied().flatten().is_none() {
                i = if i >= len - 1 { 0 } else { i + 1 };
                if i == start {
                    break; // Avoid infinite loop
                }
            }
        }
        self.state.select(Some(i));
    }

    /// Go to top (skips header if present)
    pub fn top(&mut self) {
        let mut i = 0;
        // Skip header at top if in groups mode
        if !self.item_to_container.is_empty() {
            while self.item_to_container.get(i).copied().flatten().is_none() {
                i += 1;
                if i >= self.item_to_container.len() {
                    i = 0;
                    break;
                }
            }
        }
        self.state.select(Some(i));
    }

    /// Go to bottom (skips header if present)
    pub fn bottom(&mut self, len: usize) {
        if len > 0 {
            let mut i = len - 1;
            // Skip header at bottom if in groups mode
            if !self.item_to_container.is_empty() {
                while self.item_to_container.get(i).copied().flatten().is_none() && i > 0 {
                    i -= 1;
                }
            }
            self.state.select(Some(i));
        }
    }

    /// Get currently selected visual index
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Get the container index for the current selection (handles groups mode mapping)
    pub fn selected_container_index(&self) -> Option<usize> {
        self.state.selected().and_then(|i| {
            if self.item_to_container.is_empty() {
                Some(i) // Not in groups mode, direct mapping
            } else {
                self.item_to_container.get(i).copied().flatten()
            }
        })
    }

    /// Render the container list (full-width with inline stats)
    pub fn render(&mut self, frame: &mut Frame, area: Rect, containers: &[ContainerInfo], view_mode: ListViewMode, status_filter: StatusFilter, total_count: usize) {
        // Build items - either flat or grouped
        let (items, item_count) = if status_filter == StatusFilter::Groups {
            self.build_grouped_items(containers, view_mode)
        } else {
            self.item_to_container.clear(); // Clear mapping for non-groups mode
            let items: Vec<ListItem> = containers
                .iter()
                .map(|c| {
                    let icon = status_icon(&c.status);
                    let line = match view_mode {
                        ListViewMode::Stats => self.render_stats_line(c, icon, false),
                        ListViewMode::Network => self.render_network_line(c, icon),
                        ListViewMode::Details => self.render_details_line(c, icon),
                    };
                    ListItem::new(line)
                })
                .collect();
            let len = items.len();
            (items, len)
        };

        // Build tab indicator
        let tabs = self.build_tabs(view_mode);

        // Build status filter indicator
        let filter_spans = self.build_filter_indicator(status_filter);

        // Show filtered count vs total if filtering is active
        let count_str = if status_filter == StatusFilter::All || status_filter == StatusFilter::Groups {
            format!(" Containers ({}) ", containers.len())
        } else {
            format!(" Containers ({}/{}) ", containers.len(), total_count)
        };

        let title = Line::from(vec![
            Span::styled(count_str, title_style(self.focused)),
            Span::styled("│ ", Style::default().fg(Theme::BORDER)),
            tabs.0, tabs.1, tabs.2,
            Span::styled(" │ ", Style::default().fg(Theme::BORDER)),
            filter_spans.0, filter_spans.1, filter_spans.2, filter_spans.3,
        ]);

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style(self.focused)),
            )
            .highlight_style(selected_style())
            .highlight_symbol("▶");

        frame.render_stateful_widget(list, area, &mut self.state);

        // Ensure selection is valid for grouped mode
        if status_filter == StatusFilter::Groups && !self.item_to_container.is_empty() {
            if let Some(sel) = self.state.selected() {
                // If current selection is a header, move to next container
                if self.item_to_container.get(sel).copied().flatten().is_none() {
                    self.next(item_count);
                }
            }
        }
    }

    /// Build grouped items with project headers
    fn build_grouped_items(&mut self, containers: &[ContainerInfo], view_mode: ListViewMode) -> (Vec<ListItem<'static>>, usize) {
        let mut items: Vec<ListItem> = Vec::new();
        self.item_to_container.clear();
        let mut current_project: Option<&str> = Some("__initial__"); // Sentinel to force first header

        for (idx, c) in containers.iter().enumerate() {
            let container_project = c.compose_project.as_deref();

            // Check if we're entering a new project group
            if container_project != current_project {
                current_project = container_project;
                // Add project header
                let header = self.render_group_header(container_project);
                items.push(header);
                self.item_to_container.push(None); // Header row
            }

            let icon = status_icon(&c.status);
            let line = match view_mode {
                ListViewMode::Stats => self.render_stats_line(c, icon, true),
                ListViewMode::Network => self.render_network_line(c, icon),
                ListViewMode::Details => self.render_details_line(c, icon),
            };
            items.push(ListItem::new(line));
            self.item_to_container.push(Some(idx));
        }

        let len = items.len();
        (items, len)
    }

    /// Render a group header row
    fn render_group_header(&self, project: Option<&str>) -> ListItem<'static> {
        let project_name = project.unwrap_or("Ungrouped");
        let header_style = Style::default()
            .fg(Theme::MAUVE)
            .add_modifier(Modifier::BOLD);

        let line = Line::from(vec![
            Span::styled("   ", Style::default()), // Indent to align with container names
            Span::styled(format!("┌─ {} ", project_name), header_style),
            Span::styled("─".repeat(60), Style::default().fg(Theme::BORDER)),
        ]);

        ListItem::new(line).style(Style::default().bg(Theme::BG_DARK))
    }

    /// Build styled tab spans for the view mode indicator
    fn build_tabs(&self, view_mode: ListViewMode) -> (Span<'static>, Span<'static>, Span<'static>) {
        let active_style = Style::default()
            .fg(Theme::BG_DARK)
            .bg(Theme::MAUVE)
            .add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(Theme::FG_DARK);

        let (stats_style, network_style, details_style) = match view_mode {
            ListViewMode::Stats => (active_style, inactive_style, inactive_style),
            ListViewMode::Network => (inactive_style, active_style, inactive_style),
            ListViewMode::Details => (inactive_style, inactive_style, active_style),
        };

        (
            Span::styled(" Stats ", stats_style),
            Span::styled(" Network ", network_style),
            Span::styled(" Details ", details_style),
        )
    }

    /// Build styled spans for status filter indicator
    fn build_filter_indicator(&self, status_filter: StatusFilter) -> (Span<'static>, Span<'static>, Span<'static>, Span<'static>) {
        let active_style = Style::default()
            .fg(Theme::BG_DARK)
            .bg(Theme::TEAL)
            .add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(Theme::FG_DARK);

        let (all_style, groups_style, running_style, stopped_style) = match status_filter {
            StatusFilter::All => (active_style, inactive_style, inactive_style, inactive_style),
            StatusFilter::Groups => (inactive_style, active_style, inactive_style, inactive_style),
            StatusFilter::Running => (inactive_style, inactive_style, active_style, inactive_style),
            StatusFilter::Stopped => (inactive_style, inactive_style, inactive_style, active_style),
        };

        (
            Span::styled(" All ", all_style),
            Span::styled(" Groups ", groups_style),
            Span::styled(" Running ", running_style),
            Span::styled(" Stopped ", stopped_style),
        )
    }

    /// Render Stats view line: Name, Project, Port, CPU bar, MEM bar, GPU
    /// When grouped=true, project column is hidden (shown in header instead)
    fn render_stats_line(&self, c: &ContainerInfo, icon: &str, grouped: bool) -> Line<'static> {
        // Format ports (show first port or "-")
        let port_str = if c.ports.is_empty() {
            "-".to_string()
        } else if c.ports.len() == 1 {
            c.ports[0].display()
        } else {
            format!("{}+{}", c.ports[0].display(), c.ports.len() - 1)
        };

        // CPU/MEM bars and values (only if running with stats)
        let (cpu_bar, cpu_val, mem_bar, mem_val, gpu_val) = if let Some(stats) = &c.stats {
            let cpu_bar = make_bar(stats.cpu_percent, 8);
            let cpu_val = format!("{:>5.1}%", stats.cpu_percent);
            let mem_bar = make_bar(stats.memory_percent, 8);
            let mem_val = format!("{:>5.1}%", stats.memory_percent);
            // GPU VRAM usage
            let gpu_val = match stats.vram_usage_mb {
                Some(vram) if vram >= 1024.0 => format!("{:.1}G", vram / 1024.0),
                Some(vram) => format!("{:.0}M", vram),
                None => "─".to_string(),
            };
            (cpu_bar, cpu_val, mem_bar, mem_val, gpu_val)
        } else if c.status.is_running() {
            ("        ".to_string(), "  ... ".to_string(),
             "        ".to_string(), "  ... ".to_string(), "─".to_string())
        } else {
            ("────────".to_string(), "   -  ".to_string(),
             "────────".to_string(), "   -  ".to_string(), "─".to_string())
        };

        let cpu_color = percent_color(c.stats.as_ref().map(|s| s.cpu_percent).unwrap_or(0.0));
        let mem_color = percent_color(c.stats.as_ref().map(|s| s.memory_percent).unwrap_or(0.0));
        let gpu_color = if c.stats.as_ref().and_then(|s| s.vram_usage_mb).is_some() {
            Theme::GREEN
        } else {
            Theme::FG_DARK
        };

        if grouped {
            // In grouped mode: show indent, no project column (project shown in header)
            Line::from(vec![
                Span::styled("  ", Style::default()), // Indent for group hierarchy
                Span::styled(format!(" {} ", icon), Style::default().fg(status_color(&c.status))),
                Span::styled(format!("{:<20}", truncate_name(&c.name, 20)), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:<12}", truncate_name(&port_str, 12)), Style::default().fg(Theme::YELLOW)),
                Span::styled(" CPU ", Style::default().fg(Theme::FG_DARK)),
                Span::styled(cpu_bar, Style::default().fg(Theme::CYAN)),
                Span::styled(cpu_val, Style::default().fg(cpu_color)),
                Span::styled(" MEM ", Style::default().fg(Theme::FG_DARK)),
                Span::styled(mem_bar, Style::default().fg(Theme::MAGENTA)),
                Span::styled(mem_val, Style::default().fg(mem_color)),
                Span::styled(" GPU ", Style::default().fg(Theme::FG_DARK)),
                Span::styled(format!("{:>5}", gpu_val), Style::default().fg(gpu_color)),
            ])
        } else {
            // Normal mode: show project column
            let project_str = c.compose_project.as_ref()
                .map(|p| truncate_name(p, 8))
                .unwrap_or_else(|| "─".to_string());

            Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(status_color(&c.status))),
                Span::styled(format!("{:<18}", truncate_name(&c.name, 18)), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {:<8} ", project_str), Style::default().fg(Theme::LAVENDER)),
                Span::styled(format!("{:<10}", truncate_name(&port_str, 10)), Style::default().fg(Theme::YELLOW)),
                Span::styled(" CPU ", Style::default().fg(Theme::FG_DARK)),
                Span::styled(cpu_bar, Style::default().fg(Theme::CYAN)),
                Span::styled(cpu_val, Style::default().fg(cpu_color)),
                Span::styled(" MEM ", Style::default().fg(Theme::FG_DARK)),
                Span::styled(mem_bar, Style::default().fg(Theme::MAGENTA)),
                Span::styled(mem_val, Style::default().fg(mem_color)),
                Span::styled(" GPU ", Style::default().fg(Theme::FG_DARK)),
                Span::styled(format!("{:>5}", gpu_val), Style::default().fg(gpu_color)),
            ])
        }
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

    /// Render Details view line: Name, Image, Project, Uptime
    fn render_details_line(&self, c: &ContainerInfo, icon: &str) -> Line<'static> {
        let project_str = c.compose_project.as_ref()
            .map(|p| truncate_name(p, 12))
            .unwrap_or_else(|| "─".to_string());
        let uptime = format_uptime(c.created);

        Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(status_color(&c.status))),
            Span::styled(format!("{:<20}", truncate_name(&c.name, 20)), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" Image: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:<20}", truncate_name(&c.image, 20)), Style::default().fg(Theme::LAVENDER)),
            Span::styled(" Project: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:<12}", project_str), Style::default().fg(Theme::TEAL)),
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

impl ContainerList {
    /// Get the item count for navigation (includes headers in groups mode)
    pub fn item_count(&self) -> usize {
        if self.item_to_container.is_empty() {
            0
        } else {
            self.item_to_container.len()
        }
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
