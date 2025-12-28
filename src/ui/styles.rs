use ratatui::prelude::*;

use crate::models::ContainerStatus;

/// Catppuccin Mocha color theme
/// https://github.com/catppuccin/catppuccin
pub struct Theme;

impl Theme {
    // Base colors (Catppuccin Mocha - darkened)
    pub const CRUST: Color = Color::Rgb(17, 17, 27);          // #11111b - Crust (darkest)
    pub const MANTLE: Color = Color::Rgb(24, 24, 37);         // #181825 - Mantle
    pub const BASE: Color = Color::Rgb(30, 30, 46);           // #1e1e2e - Base

    // Use darkest colors for backgrounds
    pub const BG: Color = Self::CRUST;                        // Darkest background
    pub const BG_DARK: Color = Color::Rgb(12, 12, 20);        // Even darker for modals
    pub const BG_HIGHLIGHT: Color = Color::Rgb(39, 39, 55);   // Slightly lighter for selection
    pub const SURFACE0: Color = Color::Rgb(49, 50, 68);       // #313244 - Surface0
    pub const SURFACE1: Color = Color::Rgb(69, 71, 90);       // #45475a - Surface1
    pub const SURFACE2: Color = Color::Rgb(88, 91, 112);      // #585b70 - Surface2
    pub const FG: Color = Color::Rgb(205, 214, 244);          // #cdd6f4 - Text
    pub const FG_DARK: Color = Color::Rgb(147, 153, 178);     // #9399b2 - Subtext1 (brighter)
    pub const OVERLAY: Color = Color::Rgb(127, 132, 156);     // #7f849c - Overlay1

    // Accent colors (Catppuccin Mocha)
    pub const ROSEWATER: Color = Color::Rgb(245, 224, 220);   // #f5e0dc
    pub const FLAMINGO: Color = Color::Rgb(242, 205, 205);    // #f2cdcd
    pub const PINK: Color = Color::Rgb(245, 194, 231);        // #f5c2e7
    pub const MAUVE: Color = Color::Rgb(203, 166, 247);       // #cba6f7
    pub const RED: Color = Color::Rgb(243, 139, 168);         // #f38ba8
    pub const MAROON: Color = Color::Rgb(235, 160, 172);      // #eba0ac
    pub const PEACH: Color = Color::Rgb(250, 179, 135);       // #fab387
    pub const YELLOW: Color = Color::Rgb(249, 226, 175);      // #f9e2af
    pub const GREEN: Color = Color::Rgb(166, 227, 161);       // #a6e3a1
    pub const TEAL: Color = Color::Rgb(148, 226, 213);        // #94e2d5
    pub const SKY: Color = Color::Rgb(137, 220, 235);         // #89dceb
    pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);    // #74c7ec
    pub const BLUE: Color = Color::Rgb(137, 180, 250);        // #89b4fa
    pub const LAVENDER: Color = Color::Rgb(180, 190, 254);    // #b4befe

    // Semantic aliases
    pub const CYAN: Color = Self::TEAL;
    pub const ORANGE: Color = Self::PEACH;
    pub const MAGENTA: Color = Self::MAUVE;
    pub const PURPLE: Color = Self::MAUVE;

    // UI elements
    pub const BORDER: Color = Self::SURFACE0;
    pub const BORDER_FOCUSED: Color = Self::MAUVE;
    pub const SELECTION_BG: Color = Self::SURFACE0;
    pub const SELECTION_FG: Color = Self::LAVENDER;

    // Status colors
    pub const RUNNING: Color = Self::GREEN;
    pub const EXITED: Color = Self::RED;
    pub const PAUSED: Color = Self::YELLOW;
    pub const CREATED: Color = Self::PEACH;
    pub const NOT_DEPLOYED: Color = Self::OVERLAY;

    // Progress bars
    pub const PROGRESS_FG: Color = Self::SAPPHIRE;
    pub const PROGRESS_BG: Color = Self::SURFACE0;

    // Modal
    pub const MODAL_BG: Color = Self::BG_DARK;
    pub const MODAL_BORDER: Color = Self::MAUVE;

    // Keybinding bar
    pub const KEY_BG: Color = Self::MAUVE;
    pub const KEY_FG: Color = Self::BG_DARK;
    pub const KEY_DESC_FG: Color = Self::FG_DARK;
}

/// Status icons for containers
pub struct StatusIcons;

impl StatusIcons {
    pub const RUNNING: &'static str = "●";
    pub const EXITED: &'static str = "○";
    pub const PAUSED: &'static str = "◐";
    pub const CREATED: &'static str = "◌";
    pub const RESTARTING: &'static str = "↻";
    pub const REMOVING: &'static str = "✕";
    pub const DEAD: &'static str = "✖";
    pub const NOT_DEPLOYED: &'static str = "◯";
}

/// Get the icon for a container status
pub fn status_icon(status: &ContainerStatus) -> &'static str {
    match status {
        ContainerStatus::Running => StatusIcons::RUNNING,
        ContainerStatus::Exited => StatusIcons::EXITED,
        ContainerStatus::Paused => StatusIcons::PAUSED,
        ContainerStatus::Created => StatusIcons::CREATED,
        ContainerStatus::Restarting => StatusIcons::RESTARTING,
        ContainerStatus::Removing => StatusIcons::REMOVING,
        ContainerStatus::Dead => StatusIcons::DEAD,
        ContainerStatus::NotDeployed => StatusIcons::NOT_DEPLOYED,
    }
}

/// Get the color for a container status
pub fn status_color(status: &ContainerStatus) -> Color {
    match status {
        ContainerStatus::Running => Theme::RUNNING,
        ContainerStatus::Exited => Theme::EXITED,
        ContainerStatus::Paused => Theme::PAUSED,
        ContainerStatus::Created => Theme::CREATED,
        ContainerStatus::Restarting => Theme::YELLOW,
        ContainerStatus::Removing => Theme::RED,
        ContainerStatus::Dead => Theme::RED,
        ContainerStatus::NotDeployed => Theme::NOT_DEPLOYED,
    }
}

/// Create a style for selected items
pub fn selected_style() -> Style {
    Style::default()
        .bg(Theme::SELECTION_BG)
        .fg(Theme::SELECTION_FG)
        .add_modifier(Modifier::BOLD)
}

/// Create a style for borders
pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Theme::BORDER_FOCUSED)
    } else {
        Style::default().fg(Theme::BORDER)
    }
}

/// Create a style for the header
pub fn header_style() -> Style {
    Style::default()
        .fg(Theme::LAVENDER)
        .add_modifier(Modifier::BOLD)
}

/// Create a style for panel titles
pub fn title_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Theme::LAVENDER).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::OVERLAY)
    }
}

/// Create a keybinding span (highlighted key)
pub fn key_span(key: &str) -> Span<'_> {
    Span::styled(
        format!(" {} ", key),
        Style::default()
            .bg(Theme::MAUVE)
            .fg(Theme::BG_DARK)
            .add_modifier(Modifier::BOLD),
    )
}

/// Create a keybinding description span (with trailing separator)
pub fn key_desc_span(desc: &str) -> Span<'_> {
    Span::styled(
        format!(" {}   ", desc),  // Space before, triple space after
        Style::default().fg(Theme::FG_DARK),
    )
}
