#![allow(dead_code)]

mod action;
mod app;
mod components;
mod config;
mod docker;
mod models;
mod tui;
mod ui;

use std::process::Command;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::action::Action;
use crate::app::{App, ModalState, ViewMode};
use crate::components::CreateMode;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize terminal
    let mut terminal = tui::init()?;

    // Create app
    let mut app = App::new().await?;

    // Main event loop
    let tick_rate = Duration::from_millis(500);

    loop {
        // Render
        terminal.draw(|frame| app.render(frame))?;

        // Handle events with timeout for tick
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // Handle modes that need text input separately
                match app.view_mode {
                    ViewMode::Create => {
                        handle_create_mode(&mut app, key).await?;
                    }
                    ViewMode::Filter => {
                        handle_filter_mode(&mut app, key)?;
                    }
                    ViewMode::Exec => {
                        if let Some((container, shell)) = handle_exec_mode(&mut app, key) {
                            // Exec into container and get new terminal
                            terminal = exec_into_container(&container, &shell)?;
                            // Force full redraw
                            terminal.clear()?;
                        }
                    }
                    ViewMode::Info => {
                        // Info modal - close on Esc or i
                        if matches!(key.code, KeyCode::Esc | KeyCode::Char('i')) {
                            app.view_mode = ViewMode::List;
                        }
                    }
                    _ => {
                        // Special handling for 'n' to open create form
                        if key.code == KeyCode::Char('n') && app.view_mode == ViewMode::List && matches!(app.modal, ModalState::None) {
                            app.open_create_form().await?;
                        } else if key.code == KeyCode::Char('/') && app.view_mode == ViewMode::List && matches!(app.modal, ModalState::None) {
                            // Enter filter mode
                            app.filter.activate();
                            app.view_mode = ViewMode::Filter;
                        } else if key.code == KeyCode::Char('e') && app.view_mode == ViewMode::List && matches!(app.modal, ModalState::None) {
                            // Open exec modal for running containers
                            if let Some(container) = app.selected_container() {
                                if container.status.is_running() {
                                    app.open_exec_modal(container.name.clone());
                                }
                            }
                        } else if key.code == KeyCode::Char('i') && app.view_mode == ViewMode::List && matches!(app.modal, ModalState::None) {
                            // Open info modal (network I/O)
                            app.view_mode = ViewMode::Info;
                        } else {
                            let action = handle_key_event(&app, key);
                            app.handle_action(action).await?;
                        }
                    }
                }
            }
        } else {
            // Tick for periodic updates
            app.handle_action(Action::Tick).await?;
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    tui::restore()?;

    Ok(())
}

/// Handle key events in filter mode (text input)
fn handle_filter_mode(app: &mut App, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.filter.deactivate();
            app.update_filtered_indices();
            app.view_mode = ViewMode::List;
        }
        KeyCode::Enter => {
            // Exit filter mode but keep filter active
            app.view_mode = ViewMode::List;
        }
        KeyCode::Backspace => {
            app.filter.backspace();
            app.update_filtered_indices();
        }
        KeyCode::Char(c) => {
            app.filter.type_char(c);
            app.update_filtered_indices();
        }
        KeyCode::Up => {
            app.container_list.previous(app.filtered_indices.len());
        }
        KeyCode::Down => {
            app.container_list.next(app.filtered_indices.len());
        }
        _ => {}
    }
    Ok(())
}

/// Handle key events in exec mode (shell selection)
/// Returns Some((container, shell)) if exec should be performed
fn handle_exec_mode(app: &mut App, key: event::KeyEvent) -> Option<(String, String)> {
    match key.code {
        KeyCode::Esc => {
            app.exec_modal = None;
            app.view_mode = ViewMode::List;
            None
        }
        KeyCode::Enter => {
            if let Some(ref modal) = app.exec_modal {
                let shell = modal.selected_shell().to_string();
                let container = modal.container_name.clone();

                // Close modal
                app.exec_modal = None;
                app.view_mode = ViewMode::List;

                Some((container, shell))
            } else {
                None
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(ref mut modal) = app.exec_modal {
                modal.previous();
            }
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(ref mut modal) = app.exec_modal {
                modal.next();
            }
            None
        }
        _ => None,
    }
}

/// Execute docker exec into container
/// Returns a new terminal after reinitializing
fn exec_into_container(container: &str, shell: &str) -> Result<ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>> {
    // Restore terminal for interactive docker exec
    tui::restore()?;

    // Run docker exec interactively
    let status = Command::new("docker")
        .args(["exec", "-it", container, shell])
        .status();

    if let Err(e) = status {
        eprintln!("Failed to exec into container: {}", e);
        // Small delay so user can see error
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Reinitialize terminal and return it
    Ok(tui::init()?)
}

/// Handle key events in create mode (text input)
async fn handle_create_mode(app: &mut App, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            if app.create_form.mode == CreateMode::ImageSelect {
                app.create_form.mode = CreateMode::Form;
            } else {
                app.view_mode = ViewMode::List;
            }
        }
        KeyCode::Enter => {
            if app.create_form.mode == CreateMode::ImageSelect {
                app.create_form.select_image();
            } else if app.create_form.is_valid() {
                app.create_container_from_form().await?;
            }
        }
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.create_form.prev_field();
            } else {
                // If on image field, open image selector
                if app.create_form.selected_field == 1 && app.create_form.mode == CreateMode::Form {
                    app.create_form.mode = CreateMode::ImageSelect;
                } else {
                    app.create_form.next_field();
                }
            }
        }
        KeyCode::BackTab => {
            app.create_form.prev_field();
        }
        KeyCode::Up => {
            if app.create_form.mode == CreateMode::ImageSelect {
                app.create_form.prev_image();
            }
        }
        KeyCode::Down => {
            if app.create_form.mode == CreateMode::ImageSelect {
                app.create_form.next_image();
            }
        }
        KeyCode::Backspace => {
            if app.create_form.mode == CreateMode::Form {
                app.create_form.backspace();
            }
        }
        KeyCode::Char(c) => {
            if app.create_form.mode == CreateMode::Form {
                app.create_form.type_char(c);
            }
        }
        _ => {}
    }
    Ok(())
}

/// Convert key events to actions based on current state
fn handle_key_event(app: &App, key: event::KeyEvent) -> Action {
    // Handle modal keys first
    if !matches!(app.modal, ModalState::None) {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('n') => Action::CloseModal,
            KeyCode::Enter | KeyCode::Char('y') => Action::ConfirmAction,
            _ => Action::None,
        };
    }

    // Global keys
    match key.code {
        KeyCode::Char('q') => return Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Action::Quit
        }
        KeyCode::Char('?') => return Action::ShowHelp,
        _ => {}
    }

    // View-specific keys
    match app.view_mode {
        ViewMode::List => handle_list_key(app, key),
        ViewMode::Logs => handle_logs_key(key),
        ViewMode::Create | ViewMode::Filter | ViewMode::Exec | ViewMode::Info => Action::None, // Handled separately
    }
}

/// Handle keys in list view
fn handle_list_key(app: &App, key: event::KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::Down,
        KeyCode::Char('k') | KeyCode::Up => Action::Up,
        KeyCode::Left | KeyCode::Char('h') => Action::Left,
        KeyCode::Right => Action::Right,
        KeyCode::Char('g') => Action::Top,
        KeyCode::Char('G') => Action::Bottom,

        KeyCode::Enter | KeyCode::Char('l') => {
            if let Some(name) = app.selected_container_name() {
                Action::ViewLogs(name)
            } else {
                Action::None
            }
        }

        KeyCode::Char('s') => {
            if let Some(name) = app.selected_container_name() {
                Action::StartContainer(name)
            } else {
                Action::None
            }
        }

        KeyCode::Char('x') => {
            if let Some(name) = app.selected_container_name() {
                Action::ShowConfirmStop(name)
            } else {
                Action::None
            }
        }

        KeyCode::Char('R') => {
            if let Some(name) = app.selected_container_name() {
                Action::RestartContainer(name)
            } else {
                Action::None
            }
        }

        KeyCode::Char('d') => {
            if let Some(name) = app.selected_container_name() {
                Action::ShowConfirmDelete(name)
            } else {
                Action::None
            }
        }

        KeyCode::Char('r') => Action::Refresh,

        // 'n' for new container - handled specially
        KeyCode::Char('n') => Action::None, // Will be handled in main loop

        _ => Action::None,
    }
}

/// Handle keys in logs view
fn handle_logs_key(key: event::KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::BackToList,
        KeyCode::Char('j') | KeyCode::Down => Action::Down,
        KeyCode::Char('k') | KeyCode::Up => Action::Up,
        KeyCode::Char('g') => Action::Top,
        KeyCode::Char('G') => Action::Bottom,
        _ => Action::None,
    }
}
