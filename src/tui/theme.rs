use ratatui::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Theme;

impl Theme {
    /// The primary color for borders, focuses, and active elements.
    pub const PRIMARY: Color = Color::Rgb(46, 204, 113); // Emerald Green
    /// The secondary color for highlights and accents.
    pub const SECONDARY: Color = Color::Rgb(26, 188, 156); // Turquoise / Mint
    /// The color used for background surfaces.
    pub const BACKGROUND: Color = Color::Reset; // Use terminal default or Color::Rgb(15, 23, 42) for fixed Dark.
    /// The color for surface overlays (like popups).
    pub const SURFACE: Color = Color::Reset;

    // Status colors
    pub const SUCCESS: Color = Color::Green;
    pub const ERROR: Color = Color::Red;
    pub const WARNING: Color = Color::Yellow;
    pub const INFO: Color = Color::Blue;

    // Text colors
    pub const TEXT_NORMAL: Color = Color::White;
    pub const TEXT_DIM: Color = Color::DarkGray;

    pub fn new() -> Self {
        Self
    }

    // --- Block / Border Styles ---

    pub fn block_active(&self) -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn block_inactive(&self) -> Style {
        Style::default().fg(Self::TEXT_DIM)
    }

    pub fn block_title_active(&self) -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn block_title_inactive(&self) -> Style {
        Style::default().fg(Self::TEXT_DIM)
    }

    // --- Text Styles ---

    pub fn text_normal(&self) -> Style {
        Style::default().fg(Self::TEXT_NORMAL)
    }

    pub fn text_dim(&self) -> Style {
        Style::default().fg(Self::TEXT_DIM)
    }

    pub fn text_highlight(&self) -> Style {
        Style::default()
            .fg(Self::SECONDARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn text_error(&self) -> Style {
        Style::default().fg(Self::ERROR)
    }

    // --- List / Table Styles ---

    /// Style for a selected item in a list or table row (that has focus)
    pub fn selection_active(&self) -> Style {
        Style::default()
            .bg(Self::PRIMARY)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for a selected item that does NOT have focus (e.g. inactive pane)
    pub fn selection_inactive(&self) -> Style {
        Style::default().bg(Color::DarkGray).fg(Self::TEXT_DIM)
    }

    // --- Input / Edit Styles ---

    /// Style for an active input field text
    pub fn input_active(&self) -> Style {
        Style::default().fg(Self::TEXT_NORMAL)
    }

    pub fn input_cursor(&self) -> Style {
        Style::default().bg(Self::PRIMARY).fg(Color::Black)
    }

    /// Style for the SPECIFIC CELL being edited/focused in a table
    /// High contrast: Yellow Background with Black Text.
    pub fn cell_focus(&self) -> Style {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }
}
