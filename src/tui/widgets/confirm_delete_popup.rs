use crate::tui::{app::App, theme::Theme, utils};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
pub fn render(frame: &mut Frame<'_>, app: &App) {
    // Calculate dynamic dimensions based on content
    let name = app.list_component.current_profile().unwrap();
    let text = format!("Are you sure you want to delete '{name}'? (y/n)");

    // Calculate required width
    let title_width = "Confirm Deletion".len() as u16;
    let help_width = "Press 'Esc' to exit".len() as u16;
    let text_width = text.len() as u16;

    let content_width = text_width.max(title_width).max(help_width);
    let min_width = (content_width + 6).min(frame.area().width.saturating_sub(4));

    // Calculate width percentage (at least 40%, at most 90%)
    let width_percent = ((min_width * 100) / frame.area().width).clamp(40, 90);

    // Calculate actual available width for text (accounting for borders and padding)
    let actual_width = (frame.area().width * width_percent / 100).saturating_sub(6);

    // Calculate how many lines the text will need
    let text_lines = if text_width == 0 {
        1
    } else if text_width <= actual_width {
        1
    } else {
        ((text_width as f32) / (actual_width as f32)).ceil() as u16
    };

    // Calculate minimum required height in absolute units
    let min_height = 2 + 1 + text_lines + 1 + 1; // borders + padding + text + padding + help

    // Convert to percentage for centered_rect
    let height_percent = ((min_height * 100) / frame.area().height).clamp(20, 50);

    let area = utils::centered_rect(width_percent, height_percent, frame.area());

    let block = Block::default()
        .title("Confirm Deletion")
        .borders(Borders::ALL)
        .border_style(Theme::new().text_error())
        .border_type(ratatui::widgets::BorderType::Thick);

    let inner_area = block.inner(area);

    // Split into content area and help area at the bottom
    let popup_layout = Layout::vertical([
        Constraint::Min(0),    // Content area (takes remaining space)
        Constraint::Length(1), // Help text area (exactly 1 line)
    ])
    .split(inner_area);

    let content_area = popup_layout[0];
    let help_area = popup_layout[1];

    // Vertically center the main text in content area
    let v_centered_layout = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Length(text_lines),
        Constraint::Percentage(50),
    ])
    .split(content_area);

    let text_area = v_centered_layout[1];

    let main_paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .style(Theme::new().text_normal());

    let help_paragraph = Paragraph::new("Press 'Esc' to exit")
        .alignment(Alignment::Center)
        .style(Theme::new().text_dim());

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(main_paragraph, text_area);
    frame.render_widget(help_paragraph, help_area);
}
