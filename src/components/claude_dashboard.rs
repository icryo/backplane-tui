use chrono::{Local, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::claude::SessionBlock;
use crate::ui::Theme;

/// Claude sessions dashboard state
pub struct ClaudeDashboard {
    pub sessions: Vec<SessionBlock>,
    pub selected_index: usize,
    pub selected_session_id: Option<String>,
    pub state: ListState,
    pub tmux_available: bool,
}

impl ClaudeDashboard {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            sessions: Vec::new(),
            selected_index: 0,
            selected_session_id: None,
            state,
            tmux_available: is_tmux_available(),
        }
    }

    pub fn update_sessions(&mut self, mut sessions: Vec<SessionBlock>) {
        sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        let new_index = if let Some(ref selected_id) = self.selected_session_id {
            sessions.iter()
                .position(|s| &s.session_id == selected_id)
                .unwrap_or(0)
        } else {
            sessions.iter()
                .position(|s| s.is_active)
                .unwrap_or(0)
        };

        self.sessions = sessions;
        self.selected_index = new_index;
        self.state.select(Some(new_index));

        if let Some(s) = self.sessions.get(self.selected_index) {
            self.selected_session_id = Some(s.session_id.clone());
        }
    }

    pub fn selected_session(&self) -> Option<&SessionBlock> {
        self.sessions.get(self.selected_index)
    }

    pub fn select_next(&mut self) {
        if !self.sessions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.sessions.len();
            self.state.select(Some(self.selected_index));
            self.update_selection();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.sessions.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.sessions.len() - 1
            } else {
                self.selected_index - 1
            };
            self.state.select(Some(self.selected_index));
            self.update_selection();
        }
    }

    fn update_selection(&mut self) {
        if let Some(s) = self.sessions.get(self.selected_index) {
            self.selected_session_id = Some(s.session_id.clone());
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Split into left (sessions list) and right (details)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(38),
                Constraint::Percentage(60),
            ])
            .split(area);

        self.render_sessions_list(frame, chunks[0]);
        self.render_session_details(frame, chunks[1]);
    }

    fn render_sessions_list(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(
                format!(" {} sessions ", self.sessions.len()),
                Style::default().fg(Theme::MAUVE).add_modifier(Modifier::BOLD),
            ));

        let now = Utc::now();

        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let is_selected = i == self.selected_index;
                let is_active = session.is_active;

                let minutes_ago = now.signed_duration_since(session.end_time).num_minutes() as f64;
                let time_str = format_relative_time(minutes_ago.max(0.0));

                let tokens_str = if session.total_tokens >= 1_000_000 {
                    format!("{:.1}M", session.total_tokens as f64 / 1_000_000.0)
                } else if session.total_tokens >= 1000 {
                    format!("{}k", session.total_tokens / 1000)
                } else {
                    format!("{}", session.total_tokens)
                };

                let icon = if is_active { "●" } else { "○" };
                let project = session.short_name(16);

                let content = if project.is_empty() {
                    format!(" {} {:>7} {:>6}", icon, time_str, tokens_str)
                } else {
                    format!(" {} {:>7} {:>6} {}", icon, time_str, tokens_str, project)
                };

                let style = if is_selected {
                    Style::default().bg(Theme::BG_HIGHLIGHT).fg(Theme::FG)
                } else if is_active {
                    Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::FG_DARK)
                };

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_session_details(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(
                " Session Details ",
                Style::default().fg(Theme::MAUVE).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(session) = self.selected_session() else {
            let msg = Paragraph::new("No sessions found")
                .style(Style::default().fg(Theme::FG_DARK));
            frame.render_widget(msg, inner);
            return;
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2),  // Status + times
                Constraint::Length(2),  // Project path
                Constraint::Length(2),  // Session ID
                Constraint::Length(1),  // Spacer
                Constraint::Length(2),  // Duration / Messages / Last activity
                Constraint::Length(2),  // Token totals
                Constraint::Length(2),  // Token breakdown
                Constraint::Length(2),  // Models
                Constraint::Length(1),  // Spacer
                Constraint::Min(3),     // Last prompt
                Constraint::Length(1),  // Footer
            ])
            .split(inner);

        self.render_status_line(frame, chunks[0], session);
        self.render_project_line(frame, chunks[1], session);
        self.render_session_id(frame, chunks[2], session);
        self.render_activity_stats(frame, chunks[4], session);
        self.render_token_totals(frame, chunks[5], session);
        self.render_token_breakdown(frame, chunks[6], session);
        self.render_models(frame, chunks[7], session);
        self.render_last_prompt(frame, chunks[9], session);
        self.render_footer(frame, chunks[10]);
    }

    fn render_last_prompt(&self, frame: &mut Frame, area: Rect, session: &SessionBlock) {
        let prompt_text = session.last_prompt.as_deref().unwrap_or("(no recent prompt)");

        // Wrap text to fit area width (use chars() to handle UTF-8 properly)
        let max_width = area.width.saturating_sub(8) as usize;
        let char_count = prompt_text.chars().count();
        let display_text = if char_count > max_width {
            let truncated: String = prompt_text.chars().take(max_width.saturating_sub(3)).collect();
            format!("{}...", truncated)
        } else {
            prompt_text.to_string()
        };

        let text = vec![
            Line::from(vec![
                Span::styled("Prompt: ", Style::default().fg(Theme::FG_DARK)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {}", display_text), Style::default().fg(Theme::LAVENDER)),
            ]),
        ];

        frame.render_widget(Paragraph::new(text), area);
    }

    fn render_status_line(&self, frame: &mut Frame, area: Rect, session: &SessionBlock) {
        let local_start = session.start_time.with_timezone(&Local);
        let local_end = session.end_time.with_timezone(&Local);
        let now = Utc::now();

        let age_minutes = now.signed_duration_since(session.start_time).num_minutes() as f64;
        let age_str = if age_minutes < 60.0 {
            format!("{}m old", age_minutes as u64)
        } else if age_minutes < 24.0 * 60.0 {
            format!("{}h old", (age_minutes / 60.0) as u64)
        } else {
            format!("{}d old", (age_minutes / (24.0 * 60.0)) as u64)
        };

        let status = if session.is_active {
            Span::styled("● LIVE", Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("○ idle", Style::default().fg(Theme::FG_DARK))
        };

        let text = Line::from(vec![
            status,
            Span::raw("  "),
            Span::styled(
                format!("{} → {}",
                    local_start.format("%b %d %H:%M"),
                    local_end.format("%H:%M")),
                Style::default().fg(Theme::FG),
            ),
            Span::raw("  "),
            Span::styled(format!("({})", age_str), Style::default().fg(Theme::FG_DARK)),
        ]);

        frame.render_widget(Paragraph::new(text), area);
    }

    fn render_project_line(&self, frame: &mut Frame, area: Rect, session: &SessionBlock) {
        let display = session.display_name();
        let project = if display == "(unknown)" {
            "(no project)".to_string()
        } else {
            truncate_path(display, 60)
        };

        let text = Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(project, Style::default().fg(Theme::CYAN)),
        ]);

        frame.render_widget(Paragraph::new(text), area);
    }

    fn render_session_id(&self, frame: &mut Frame, area: Rect, session: &SessionBlock) {
        let text = Line::from(vec![
            Span::styled("ID:   ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(&session.session_id, Style::default().fg(Theme::FG_DARK)),
        ]);

        frame.render_widget(Paragraph::new(text), area);
    }

    fn render_activity_stats(&self, frame: &mut Frame, area: Rect, session: &SessionBlock) {
        let duration = session.duration_minutes();
        let now = Utc::now();
        let last_activity_mins = now.signed_duration_since(session.end_time).num_minutes();

        let last_activity = if last_activity_mins < 1 {
            "just now".to_string()
        } else {
            format_relative_time(last_activity_mins as f64)
        };

        let text = Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format_time(duration), Style::default().fg(Theme::FG)),
            Span::raw("   "),
            Span::styled("Messages: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{}", session.message_count), Style::default().fg(Theme::FG)),
            Span::raw("   "),
            Span::styled("Last: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(last_activity, Style::default().fg(Theme::FG)),
        ]);

        frame.render_widget(Paragraph::new(text), area);
    }

    fn render_token_totals(&self, frame: &mut Frame, area: Rect, session: &SessionBlock) {
        let burn = session.burn_rate();

        let text = Line::from(vec![
            Span::styled("Tokens:   ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format_number(session.total_tokens), Style::default().fg(Theme::FG)),
            Span::raw("   "),
            Span::styled("Rate: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:.0}/min", burn), Style::default().fg(Theme::YELLOW)),
            Span::raw("   "),
            Span::styled("Cost: ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("${:.2}", session.cost_usd), Style::default().fg(Theme::GREEN)),
        ]);

        frame.render_widget(Paragraph::new(text), area);
    }

    fn render_token_breakdown(&self, frame: &mut Frame, area: Rect, session: &SessionBlock) {
        let mut input_total = 0u64;
        let mut output_total = 0u64;
        let mut cache_create = 0u64;
        let mut cache_read = 0u64;

        for stats in session.per_model_stats.values() {
            input_total += stats.input_tokens;
            output_total += stats.output_tokens;
            cache_create += stats.cache_creation_tokens;
            cache_read += stats.cache_read_tokens;
        }

        let text = Line::from(vec![
            Span::styled("Breakdown:", Style::default().fg(Theme::FG_DARK)),
            Span::raw(" "),
            Span::styled(format!("in:{}", format_compact(input_total)), Style::default().fg(Theme::TEAL)),
            Span::raw(" "),
            Span::styled(format!("out:{}", format_compact(output_total)), Style::default().fg(Theme::PEACH)),
            Span::raw(" "),
            Span::styled(format!("cache+:{}", format_compact(cache_create)), Style::default().fg(Theme::LAVENDER)),
            Span::raw(" "),
            Span::styled(format!("cache-:{}", format_compact(cache_read)), Style::default().fg(Theme::SKY)),
        ]);

        frame.render_widget(Paragraph::new(text), area);
    }

    fn render_models(&self, frame: &mut Frame, area: Rect, session: &SessionBlock) {
        let total: u64 = session.per_model_stats.values().map(|s| s.total_tokens()).sum();

        if total == 0 {
            let text = Line::from(vec![
                Span::styled("Models:   ", Style::default().fg(Theme::FG_DARK)),
                Span::styled("(none)", Style::default().fg(Theme::FG_DARK)),
            ]);
            frame.render_widget(Paragraph::new(text), area);
            return;
        }

        let mut models: Vec<_> = session.per_model_stats.iter().collect();
        models.sort_by(|a, b| b.1.total_tokens().cmp(&a.1.total_tokens()));

        let mut spans = vec![Span::styled("Models:   ", Style::default().fg(Theme::FG_DARK))];

        for (i, (name, stats)) in models.iter().take(4).enumerate() {
            if i > 0 {
                spans.push(Span::raw(" │ "));
            }

            let pct = (stats.total_tokens() as f64 / total as f64) * 100.0;

            let color = if name.contains("opus") {
                Theme::MAUVE
            } else if name.contains("sonnet") {
                Theme::CYAN
            } else {
                Theme::GREEN
            };

            spans.push(Span::styled(
                format!("{} {:.0}%", name, pct),
                Style::default().fg(color),
            ));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let dim = Style::default().fg(Theme::FG_DARK);
        let key = Style::default().fg(Theme::CYAN);
        let inactive = Style::default().fg(Theme::BG_HIGHLIGHT);

        let e_style = if self.tmux_available { key } else { inactive };

        let spans = vec![
            Span::styled("↑↓", key), Span::styled(" nav", dim),
            Span::raw("  "),
            Span::styled("e", e_style), Span::styled(" resume", dim),
            Span::raw("  "),
            Span::styled("r", key), Span::styled(" reload", dim),
            Span::raw("  "),
            Span::styled("TAB", key), Span::styled(" containers", dim),
        ];

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }
}

impl Default for ClaudeDashboard {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions
fn format_relative_time(minutes_ago: f64) -> String {
    if minutes_ago < 1.0 {
        "now".to_string()
    } else if minutes_ago < 60.0 {
        format!("{}m ago", minutes_ago as u64)
    } else if minutes_ago < 24.0 * 60.0 {
        format!("{}h ago", (minutes_ago / 60.0).floor() as u64)
    } else {
        format!("{}d ago", (minutes_ago / (24.0 * 60.0)).floor() as u64)
    }
}

fn format_time(minutes: f64) -> String {
    let hours = (minutes / 60.0).floor() as u64;
    let mins = (minutes % 60.0).floor() as u64;

    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().rev().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result.chars().rev().collect()
}

fn format_compact(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1000 {
        format!("{}k", n / 1000)
    } else {
        format!("{}", n)
    }
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}

/// Find tmux socket path - checks common locations
fn find_tmux_socket() -> Option<String> {
    // Check if we're inside tmux already
    if std::env::var("TMUX").is_ok() {
        return Some(String::new()); // Empty means use default
    }

    // Check for tmux sockets in /tmp
    for uid in [1000, 1001, 0] {
        let socket_path = format!("/tmp/tmux-{}/default", uid);
        if std::path::Path::new(&socket_path).exists() {
            return Some(socket_path);
        }
    }

    None
}

/// Check if tmux is available (has running sessions we can attach to)
fn is_tmux_available() -> bool {
    find_tmux_socket().is_some()
}

/// Open a new tmux window in the session's directory
pub fn resume_session(_session_id: &str, path: &str) -> bool {
    let socket = match find_tmux_socket() {
        Some(s) => s,
        None => return false,
    };

    let working_dir = if !path.is_empty() && std::path::Path::new(path).is_dir() {
        path.to_string()
    } else {
        return false;
    };

    // Build tmux command to open new window in directory
    let mut cmd = std::process::Command::new("tmux");

    if !socket.is_empty() {
        cmd.args(["-S", &socket]);
    }

    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    cmd.args([
        "new-window",
        "-t", "0:",
        "-c", &working_dir,
    ]);

    // Use spawn() to avoid blocking, detach completely
    cmd.spawn().is_ok()
}
