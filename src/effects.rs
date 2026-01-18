use std::time::{Duration, Instant};
use ratatui::prelude::*;
use tachyonfx::{fx, Effect, EffectTimer, Interpolation};

/// Color cycle for selection highlight (visible purple gradient)
const SELECTION_COLORS: &[(u8, u8, u8)] = &[
    (180, 140, 255), // Bright purple
    (120, 80, 180),  // Dark purple
];

/// Manages visual effects for the application
pub struct EffectManager {
    /// Startup fade-in effect
    startup_fx: Option<Effect>,
    /// Loading pulse effect
    loading_fx: Option<Effect>,
    /// Status change effect (for container state changes)
    status_fx: Option<Effect>,
    /// When selection highlighting started
    selection_start: Instant,
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectManager {
    pub fn new() -> Self {
        Self {
            startup_fx: Some(Self::create_startup_effect()),
            loading_fx: Some(Self::create_loading_effect()),
            status_fx: None,
            selection_start: Instant::now(),
        }
    }

    /// Create the initial fade-in effect for app startup
    fn create_startup_effect() -> Effect {
        fx::fade_from(
            (0, 0, 0),  // fg color as tuple
            (0, 0, 0),  // bg color as tuple
            EffectTimer::from_ms(800, Interpolation::QuadOut),
        )
    }

    /// Create a subtle pulse effect for loading state
    fn create_loading_effect() -> Effect {
        fx::ping_pong(fx::fade_to_fg(
            (180, 180, 220),
            EffectTimer::from_ms(600, Interpolation::SineInOut),
        ))
    }

    /// Create a color flash effect for status changes
    fn create_status_sweep_effect(running: bool) -> Effect {
        let color = if running {
            (80, 200, 120)  // Green for start
        } else {
            (200, 80, 80)   // Red for stop
        };
        // Use fade_from to create a flash effect
        fx::fade_from(
            color,
            (0, 0, 0),
            EffectTimer::from_ms(400, Interpolation::QuadOut),
        )
    }

    /// Trigger status change effect
    pub fn trigger_status_change(&mut self, running: bool) {
        self.status_fx = Some(Self::create_status_sweep_effect(running));
    }

    /// Process all active effects
    pub fn process(&mut self, elapsed: Duration, buf: &mut Buffer, area: Rect) {
        // Process startup effect
        if let Some(ref mut fx) = self.startup_fx {
            fx.process(elapsed.into(), buf, area);
            if fx.done() {
                self.startup_fx = None;
            }
        }
    }

    /// Process loading effects (call on loading indicator area)
    pub fn process_loading(&mut self, elapsed: Duration, buf: &mut Buffer, area: Rect, is_loading: bool) {
        if is_loading {
            if let Some(ref mut fx) = self.loading_fx {
                fx.process(elapsed.into(), buf, area);
                // Don't clear - it loops via ping_pong
            }
        }
    }

    /// Process status change effects
    pub fn process_status(&mut self, elapsed: Duration, buf: &mut Buffer, area: Rect) {
        if let Some(ref mut fx) = self.status_fx {
            fx.process(elapsed.into(), buf, area);
            if fx.done() {
                self.status_fx = None;
            }
        }
    }

    /// Check if startup animation is still playing
    pub fn is_starting_up(&self) -> bool {
        self.startup_fx.is_some()
    }

    /// Render color cycling border effect on the selected row
    /// Call this after rendering the container list
    pub fn render_selection_highlight(&self, buf: &mut Buffer, area: Rect) {
        let elapsed = self.selection_start.elapsed().as_secs_f32();
        let speed = 20.0; // cells per second
        let base_idx = (elapsed * speed) as usize;

        // Get color at a given position in the cycle
        let color_at = |idx: usize| -> Color {
            let (r, g, b) = SELECTION_COLORS[idx % SELECTION_COLORS.len()];
            Color::Rgb(r, g, b)
        };

        // Top border
        for (i, x) in (area.x..area.right()).enumerate() {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_fg(color_at(base_idx + i));
            }
        }

        // Right border
        let offset = area.width as usize;
        for (i, y) in (area.y + 1..area.bottom().saturating_sub(1)).enumerate() {
            if let Some(cell) = buf.cell_mut((area.right().saturating_sub(1), y)) {
                cell.set_fg(color_at(base_idx + offset + i));
            }
        }

        // Bottom border (right to left)
        let offset = offset + area.height.saturating_sub(2) as usize;
        for (i, x) in (area.x..area.right()).rev().enumerate() {
            if let Some(cell) = buf.cell_mut((x, area.bottom().saturating_sub(1))) {
                cell.set_fg(color_at(base_idx + offset + i));
            }
        }

        // Left border (bottom to top)
        let offset = offset + area.width as usize;
        for (i, y) in (area.y + 1..area.bottom().saturating_sub(1)).rev().enumerate() {
            if let Some(cell) = buf.cell_mut((area.x, y)) {
                cell.set_fg(color_at(base_idx + offset + i));
            }
        }
    }

    /// Render a subtle glow effect on a selected row (single line, not box)
    pub fn render_row_highlight(&self, buf: &mut Buffer, area: Rect) {
        let elapsed = self.selection_start.elapsed().as_secs_f32();
        let speed = 30.0; // cells per second for faster animation
        let base_idx = (elapsed * speed) as usize;

        // Get color at a given position in the cycle
        let color_at = |idx: usize| -> Color {
            let (r, g, b) = SELECTION_COLORS[idx % SELECTION_COLORS.len()];
            Color::Rgb(r, g, b)
        };

        // Apply cycling colors to the first and last characters of the row
        // This creates a subtle "bookend" effect
        if area.width >= 2 {
            // Left edge - apply to first char
            if let Some(cell) = buf.cell_mut((area.x, area.y)) {
                cell.set_fg(color_at(base_idx));
            }

            // Right edge - apply to last char
            if let Some(cell) = buf.cell_mut((area.right().saturating_sub(1), area.y)) {
                cell.set_fg(color_at(base_idx + area.width as usize / 2));
            }
        }
    }
}
