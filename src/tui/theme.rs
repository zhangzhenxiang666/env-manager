use ratatui::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct Theme;

impl Theme {
    // Tokyo Night Palette
    /// The primary color for borders, focuses, and active elements.
    pub const PRIMARY: Color = Color::Rgb(122, 162, 247); // #7aa2f7 (Blue)
    /// The secondary color for highlights and accents.
    pub const SECONDARY: Color = Color::Rgb(187, 154, 247); // #bb9af7 (Purple)
    /// The color used for background surfaces.
    pub const BACKGROUND: Color = Color::Reset;
    /// The color for surface overlays (like popups).
    pub const SURFACE: Color = Color::Reset;

    // Status colors
    pub const SUCCESS: Color = Color::Rgb(158, 206, 106); // #9ece6a (Green)
    pub const ERROR: Color = Color::Rgb(247, 118, 142); // #f7768e (Red)
    pub const WARNING: Color = Color::Rgb(224, 175, 104); // #e0af68 (Yellow/Orange)
    pub const INFO: Color = Color::Rgb(125, 207, 255); // #7dcfff (Cyan)

    // Text colors
    pub const TEXT_NORMAL: Color = Color::Rgb(192, 202, 245); // #c0caf5 (White-ish)
    pub const TEXT_DIM: Color = Color::Rgb(86, 95, 137); // #565f89 (Dark Blue-Gray)

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
            .bg(Color::Rgb(61, 89, 161)) // #3d59a1 (Selection Background)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for a selected item that does NOT have focus (e.g. inactive pane)
    pub fn selection_inactive(&self) -> Style {
        Style::default()
            .bg(Color::Rgb(41, 46, 66))
            .fg(Self::TEXT_NORMAL) // Darker background
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
    /// High contrast for Tokyo Night
    pub fn cell_focus(&self) -> Style {
        Style::default()
            .bg(Self::WARNING) // Use the yellow/orange for high attention
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }

    pub fn row_selected(&self) -> Style {
        self.selection_active()
    }
}
