use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

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
    pub fn render(&mut self, frame: &mut Frame, area: Rect, containers: &[ContainerInfo]) {
        let items: Vec<ListItem> = containers
            .iter()
            .map(|c| {
                let icon = status_icon(&c.status);
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
                    // Running but no stats yet
                    ("        ".to_string(), "  ... ".to_string(),
                     "        ".to_string(), "  ... ".to_string())
                } else {
                    // Not running - show dashes
                    ("────────".to_string(), "   -  ".to_string(),
                     "────────".to_string(), "   -  ".to_string())
                };

                let cpu_color = percent_color(c.stats.as_ref().map(|s| s.cpu_percent).unwrap_or(0.0));
                let mem_color = percent_color(c.stats.as_ref().map(|s| s.memory_percent).unwrap_or(0.0));

                // Build the line with all info inline
                let line = Line::from(vec![
                    // Status icon
                    Span::styled(
                        format!(" {} ", icon),
                        Style::default().fg(status_color(&c.status)),
                    ),
                    // Container name
                    Span::styled(
                        format!("{:<20}", truncate_name(&c.name, 20)),
                        Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD),
                    ),
                    // Type (CLI/WEB)
                    Span::styled(
                        format!(" {:>3} ", type_indicator),
                        Style::default().fg(Theme::FG_DARK),
                    ),
                    // Port
                    Span::styled(
                        format!("{:<14}", truncate_name(&port_str, 14)),
                        Style::default().fg(Theme::YELLOW),
                    ),
                    // CPU label
                    Span::styled(" CPU ", Style::default().fg(Theme::FG_DARK)),
                    // CPU bar
                    Span::styled(cpu_bar, Style::default().fg(Theme::CYAN)),
                    // CPU value
                    Span::styled(cpu_val, Style::default().fg(cpu_color)),
                    // MEM label
                    Span::styled(" MEM ", Style::default().fg(Theme::FG_DARK)),
                    // MEM bar
                    Span::styled(mem_bar, Style::default().fg(Theme::MAGENTA)),
                    // MEM value
                    Span::styled(mem_val, Style::default().fg(mem_color)),
                ]);

                ListItem::new(line)
            })
            .collect();

        let title = format!(" Containers ({}) ", containers.len());
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
