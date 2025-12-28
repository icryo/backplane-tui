use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

use crate::ui::{key_span, key_desc_span, Theme};

/// Keybinding definition
pub struct KeyBinding {
    pub key: &'static str,
    pub desc: &'static str,
}

/// Status bar component (bottom of screen) - keybindings only
pub struct StatusBar;

impl StatusBar {
    /// Get keybindings for list view
    pub fn list_keybindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding { key: "↑↓", desc: "nav" },
            KeyBinding { key: "/", desc: "filter" },
            KeyBinding { key: "i", desc: "info" },
            KeyBinding { key: "e", desc: "exec" },
            KeyBinding { key: "s", desc: "start" },
            KeyBinding { key: "x", desc: "stop" },
            KeyBinding { key: "l", desc: "logs" },
            KeyBinding { key: "n", desc: "new" },
            KeyBinding { key: "?", desc: "help" },
        ]
    }

    /// Get keybindings for logs view
    pub fn logs_keybindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding { key: "↑↓", desc: "scroll" },
            KeyBinding { key: "g/G", desc: "top/end" },
            KeyBinding { key: "Esc", desc: "back" },
            KeyBinding { key: "q", desc: "quit" },
        ]
    }

    /// Get keybindings for create view
    pub fn create_keybindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding { key: "Tab", desc: "next field" },
            KeyBinding { key: "Enter", desc: "create" },
            KeyBinding { key: "Esc", desc: "cancel" },
        ]
    }

    /// Get keybindings for filter view
    pub fn filter_keybindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding { key: "↑↓", desc: "select" },
            KeyBinding { key: "Enter", desc: "confirm" },
            KeyBinding { key: "Esc", desc: "clear" },
        ]
    }

    /// Get keybindings for exec view
    pub fn exec_keybindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding { key: "↑↓", desc: "select" },
            KeyBinding { key: "Enter", desc: "exec" },
            KeyBinding { key: "Esc", desc: "cancel" },
        ]
    }

    /// Get keybindings for info view
    pub fn info_keybindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding { key: "i", desc: "close" },
            KeyBinding { key: "Esc", desc: "close" },
        ]
    }

    pub fn render(frame: &mut Frame, area: Rect, view: &str) {
        // Keybindings based on view
        let keybindings = match view {
            "logs" => Self::logs_keybindings(),
            "create" => Self::create_keybindings(),
            "filter" => Self::filter_keybindings(),
            "exec" => Self::exec_keybindings(),
            "info" => Self::info_keybindings(),
            _ => Self::list_keybindings(),
        };

        let mut spans: Vec<Span> = Vec::new();
        for kb in keybindings {
            spans.push(key_span(kb.key));
            spans.push(key_desc_span(kb.desc));
        }

        let keys_line = Line::from(spans);
        let keys_widget = Paragraph::new(keys_line)
            .style(Style::default().bg(Theme::BG_DARK))
            .alignment(Alignment::Center);
        frame.render_widget(keys_widget, area);
    }
}
