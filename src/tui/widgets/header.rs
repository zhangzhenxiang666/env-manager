use crate::tui::{app::App, theme::Theme};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let title = if let Some(msg) = &app.status_message {
        Paragraph::new(Line::from(msg.clone()).centered()).style(Theme::new().text_highlight())
    } else {
        Paragraph::new(Line::from("Env-Manage TUI").centered()).style(Theme::new().text_normal())
    };

    let block = title.block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::new().block_active()),
    );

    frame.render_widget(block, area);
}
