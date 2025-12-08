use crate::tui::{app::App, theme::Theme};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if app.list_component.is_searching() {
        let search_input = app.list_component.search_input();
        // Text "Search: " + input.text
        let prefix = "Search: ";

        let paragraph = Paragraph::new(Line::from(vec![
            Span::styled(prefix, Theme::new().text_highlight()),
            Span::styled(search_input.text.as_str(), Theme::new().text_normal()), // Use normal or highlight?
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::new().block_active()), // Active color for search
        );

        frame.render_widget(paragraph, area);

        // Cursor Calculation
        // Calculate the width of text before the cursor
        let cursor_char_index = search_input.cursor_position;
        let input_text_before_cursor: String =
            search_input.text.chars().take(cursor_char_index).collect();
        // Width of "Search: "
        let prefix_width = prefix.width();
        // Width of input text up to cursor
        let input_width_before_cursor = UnicodeWidthStr::width(input_text_before_cursor.as_str());

        // Total visual offset x
        let cursor_x = area.x + 1 + (prefix_width + input_width_before_cursor) as u16; // +1 for left border
        let cursor_y = area.y + 1; // +1 for top border

        // Ensure cursor is within bounds (rudimentary check, scroll not implemented for header yet but unlikely to overflow for simple partial match)
        // If overflow, we'd need scrolling. Assuming header width is sufficient for now.
        frame.set_cursor_position((cursor_x, cursor_y));
    } else {
        let title = if let Some(msg) = &app.status_message {
            Paragraph::new(Line::from(msg.clone()).centered()).style(Theme::new().text_highlight())
        } else {
            Paragraph::new(Line::from("Env-Manage TUI").centered())
                .style(Theme::new().text_normal())
        };

        let block = title.block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::new().block_active()),
        );

        frame.render_widget(block, area);
    }
}
