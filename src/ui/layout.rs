use ratatui::prelude::*;

/// Create the main layout with header, body (split pane), and footer
pub fn main_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Min(0),     // Body
            Constraint::Length(1),  // Footer/status bar
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}

/// Split header into title and stats sections
pub fn header_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(16),        // Title
            Constraint::Length(60),     // Stats (wider for VRAM)
        ])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create the split pane layout for container list and details
pub fn split_pane(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Container list
            Constraint::Percentage(65), // Details/logs
        ])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create the details pane layout
pub fn details_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Container info + stats
            Constraint::Min(0),     // Logs
        ])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create a centered modal area
pub fn centered_modal(area: Rect, width_percent: u16, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height.min(80)) / 2),
            Constraint::Length(height),
            Constraint::Percentage((100 - height.min(80)) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
