use crate::tui::{app::App, theme::Theme, utils};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = utils::centered_rect(40, 20, frame.area());

    let block = Block::default()
        .title("Confirm Deletion")
        .borders(Borders::ALL)
        .border_style(Theme::new().text_error())
        .border_type(ratatui::widgets::BorderType::Thick);

    let inner_area = block.inner(area);

    let popup_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(inner_area);

    let content_area = popup_layout[0];
    let help_area = popup_layout[1];

    let v_centered_layout = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(content_area);

    let text_area = v_centered_layout[1];

    let selected_name = &app.list_component.profile_names[app.list_component.selected_index];
    let text = format!("Are you sure you want to delete '{selected_name}'? (y/n)");
    let main_paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: false });

    let help_paragraph = Paragraph::new("Press 'Esc' to exit")
        .alignment(Alignment::Center)
        .style(Theme::new().text_dim());

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(main_paragraph, text_area);
    frame.render_widget(help_paragraph, help_area);
}
