use crate::tui::{app::App, theme::Theme, utils};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render(frame: &mut Frame<'_>, _app: &App) {
    let area = utils::centered_rect(50, 20, frame.area());
    let theme = Theme::new();

    let block = Block::default()
        .title("Unsaved Changes")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::WARNING))
        .border_type(ratatui::widgets::BorderType::Thick);

    let inner_area = block.inner(area);

    let popup_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(inner_area);

    let content_area = popup_layout[0];
    let help_area = popup_layout[1];

    let v_centered_layout = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(2),
        Constraint::Min(0),
    ])
    .split(content_area);

    let text_area = v_centered_layout[1];

    let text = "You have unsaved changes.\nSave all before exiting?";
    let main_paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .style(theme.text_normal());

    let help_text = vec![
        Span::styled("y", Style::default().fg(Color::Rgb(106, 255, 160))),
        Span::raw(": Save & Quit  "),
        Span::styled("n", Style::default().fg(Color::Rgb(255, 107, 107))),
        Span::raw(": Discard & Quit  "),
        Span::styled("Esc", Style::default().fg(Color::Gray)),
        Span::raw(": Cancel"),
    ];
    let help_paragraph = Paragraph::new(Line::from(help_text)).alignment(Alignment::Center);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(main_paragraph, text_area);
    frame.render_widget(help_paragraph, help_area);
}
