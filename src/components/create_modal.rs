use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::ui::{centered_modal, Theme};

/// Form field for container creation
#[derive(Debug, Clone, Default)]
pub struct CreateContainerForm {
    pub name: String,
    pub image: String,
    pub port_host: String,
    pub port_container: String,
    pub env_vars: String,
    pub volumes: String,
    pub command: String,
    pub selected_field: usize,
    pub selected_image_idx: usize,
    pub available_images: Vec<String>,
    pub mode: CreateMode,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum CreateMode {
    #[default]
    Form,
    ImageSelect,
}

impl CreateContainerForm {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            image: String::new(),
            port_host: String::new(),
            port_container: String::new(),
            env_vars: String::new(),
            volumes: String::new(),
            command: String::new(),
            selected_field: 0,
            selected_image_idx: 0,
            available_images: Vec::new(),
            mode: CreateMode::Form,
        }
    }

    pub fn field_count() -> usize {
        7 // name, image, port_host, port_container, env_vars, volumes, command
    }

    pub fn next_field(&mut self) {
        self.selected_field = (self.selected_field + 1) % Self::field_count();
    }

    pub fn prev_field(&mut self) {
        if self.selected_field == 0 {
            self.selected_field = Self::field_count() - 1;
        } else {
            self.selected_field -= 1;
        }
    }

    pub fn current_field_mut(&mut self) -> &mut String {
        match self.selected_field {
            0 => &mut self.name,
            1 => &mut self.image,
            2 => &mut self.port_host,
            3 => &mut self.port_container,
            4 => &mut self.env_vars,
            5 => &mut self.volumes,
            6 => &mut self.command,
            _ => &mut self.name,
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.current_field_mut().push(c);
    }

    pub fn backspace(&mut self) {
        self.current_field_mut().pop();
    }

    pub fn select_image(&mut self) {
        if !self.available_images.is_empty() {
            self.image = self.available_images[self.selected_image_idx].clone();
            self.mode = CreateMode::Form;
        }
    }

    pub fn next_image(&mut self) {
        if !self.available_images.is_empty() {
            self.selected_image_idx = (self.selected_image_idx + 1) % self.available_images.len();
        }
    }

    pub fn prev_image(&mut self) {
        if !self.available_images.is_empty() {
            if self.selected_image_idx == 0 {
                self.selected_image_idx = self.available_images.len() - 1;
            } else {
                self.selected_image_idx -= 1;
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.image.is_empty()
    }
}

/// Create container modal component
pub struct CreateModal;

impl CreateModal {
    pub fn render(frame: &mut Frame, area: Rect, form: &mut CreateContainerForm) {
        let modal_area = centered_modal(area, 70, 22);

        // Clear background
        frame.render_widget(Clear, modal_area);

        match form.mode {
            CreateMode::Form => Self::render_form(frame, modal_area, form),
            CreateMode::ImageSelect => Self::render_image_select(frame, modal_area, form),
        }
    }

    fn render_form(frame: &mut Frame, area: Rect, form: &CreateContainerForm) {
        let block = Block::default()
            .title(" Create Container ")
            .title_style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MAGENTA))
            .style(Style::default().bg(Theme::BG_DARK));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Form layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Name
                Constraint::Length(3), // Image
                Constraint::Length(3), // Ports
                Constraint::Length(3), // Env
                Constraint::Length(3), // Volumes
                Constraint::Length(3), // Command
                Constraint::Min(0),    // Instructions
            ])
            .split(inner);

        // Name field
        Self::render_field(frame, chunks[0], "Name", &form.name, form.selected_field == 0);

        // Image field (with browse hint)
        let image_block = Block::default()
            .title(if form.selected_field == 1 {
                " Image (press Tab to browse) "
            } else {
                " Image "
            })
            .borders(Borders::ALL)
            .border_style(if form.selected_field == 1 {
                Style::default().fg(Theme::CYAN)
            } else {
                Style::default().fg(Theme::BORDER)
            });
        let image_text = Paragraph::new(form.image.as_str())
            .style(Style::default().fg(Theme::FG))
            .block(image_block);
        frame.render_widget(image_text, chunks[1]);

        // Ports (split into two)
        let port_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);
        Self::render_field(frame, port_chunks[0], "Host Port", &form.port_host, form.selected_field == 2);
        Self::render_field(frame, port_chunks[1], "Container Port", &form.port_container, form.selected_field == 3);

        // Env vars
        Self::render_field(frame, chunks[3], "Env (KEY=val,KEY2=val2)", &form.env_vars, form.selected_field == 4);

        // Volumes
        Self::render_field(frame, chunks[4], "Volumes (/host:/container)", &form.volumes, form.selected_field == 5);

        // Command
        Self::render_field(frame, chunks[5], "Command (optional)", &form.command, form.selected_field == 6);

        // Instructions
        let instructions = Paragraph::new(Line::from(vec![
            Span::styled("Tab", Style::default().fg(Theme::BLUE).add_modifier(Modifier::BOLD)),
            Span::styled(" next field  ", Style::default().fg(Theme::FG_DARK)),
            Span::styled("Shift+Tab", Style::default().fg(Theme::BLUE).add_modifier(Modifier::BOLD)),
            Span::styled(" prev field  ", Style::default().fg(Theme::FG_DARK)),
            Span::styled("Enter", Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" create  ", Style::default().fg(Theme::FG_DARK)),
            Span::styled("Esc", Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)),
            Span::styled(" cancel", Style::default().fg(Theme::FG_DARK)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(instructions, chunks[6]);
    }

    fn render_field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
        let block = Block::default()
            .title(format!(" {} ", label))
            .borders(Borders::ALL)
            .border_style(if focused {
                Style::default().fg(Theme::CYAN)
            } else {
                Style::default().fg(Theme::BORDER)
            });

        let display_value = if focused && value.is_empty() {
            "│" // Cursor indicator
        } else if focused {
            // Show cursor at end
            &format!("{}│", value)
        } else {
            value
        };

        let text = Paragraph::new(display_value)
            .style(Style::default().fg(if focused { Theme::FG } else { Theme::FG_DARK }))
            .block(block);
        frame.render_widget(text, area);
    }

    fn render_image_select(frame: &mut Frame, area: Rect, form: &mut CreateContainerForm) {
        let block = Block::default()
            .title(" Select Image ")
            .title_style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MAGENTA))
            .style(Style::default().bg(Theme::BG_DARK));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if form.available_images.is_empty() {
            let text = Paragraph::new("No images found. Pull an image first.")
                .style(Style::default().fg(Theme::FG_DARK))
                .alignment(Alignment::Center);
            frame.render_widget(text, inner);
            return;
        }

        let items: Vec<ListItem> = form
            .available_images
            .iter()
            .map(|img| {
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(img, Style::default().fg(Theme::FG)),
                ]))
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(form.selected_image_idx));

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Theme::SELECTION_BG)
                    .fg(Theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, inner, &mut state);
    }
}
