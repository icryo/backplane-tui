use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

use crate::models::SystemStats;
use crate::ui::Theme;

/// Header component with title and system stats
pub struct Header;

impl Header {
    pub fn render(frame: &mut Frame, area: Rect, stats: &SystemStats, vram: Option<f32>) {
        use crate::ui::layout::header_layout;

        let (title_area, stats_area) = header_layout(area);

        // Title
        let title = Paragraph::new(" Backplane TUI ")
            .style(Style::default().fg(Theme::BLUE).add_modifier(Modifier::BOLD));
        frame.render_widget(title, title_area);

        // System stats with colors
        let cpu_color = stat_color(stats.cpu_percent);
        let mem_color = stat_color(stats.memory_percent);
        let disk_color = stat_color(stats.disk_percent);

        let mut spans = vec![
            Span::styled("CPU ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:>4.0}%", stats.cpu_percent), Style::default().fg(cpu_color)),
            Span::styled(" │ ", Style::default().fg(Theme::BORDER)),
            Span::styled("MEM ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(
                format!("{:.1}/{:.0}G", stats.memory_used_gb, stats.memory_total_gb),
                Style::default().fg(mem_color),
            ),
            Span::styled(" │ ", Style::default().fg(Theme::BORDER)),
            Span::styled("DISK ", Style::default().fg(Theme::FG_DARK)),
            Span::styled(format!("{:>4.0}%", stats.disk_percent), Style::default().fg(disk_color)),
        ];

        // Add VRAM if available
        if let Some(vram_percent) = vram {
            let vram_color = stat_color(vram_percent);
            spans.push(Span::styled(" │ ", Style::default().fg(Theme::BORDER)));
            spans.push(Span::styled("VRAM ", Style::default().fg(Theme::FG_DARK)));
            spans.push(Span::styled(format!("{:>4.0}%", vram_percent), Style::default().fg(vram_color)));
        }

        let stats_line = Line::from(spans);
        let stats_widget = Paragraph::new(stats_line).alignment(Alignment::Right);
        frame.render_widget(stats_widget, stats_area);
    }
}

/// Get color based on usage percentage
fn stat_color(percent: f32) -> Color {
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
