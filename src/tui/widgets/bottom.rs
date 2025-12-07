use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span, Text},
};

use crate::tui::{
    app::AppState::{self, AddNew, List},
    theme::Theme,
};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &crate::tui::app::App) {
    let version_info = Text::from(vec![Line::raw(format!(
        "Env-Mnage {}",
        env!("CARGO_PKG_VERSION")
    ))])
    .right_aligned();

    match app.state {
        List => list_state(frame, area, app),
        AddNew => list_state(frame, area, app),
        AppState::Rename => rename_state(frame, area),
        _ => {}
    }
    frame.render_widget(version_info, area);
}

fn list_state(frame: &mut Frame<'_>, area: Rect, app: &crate::tui::app::App) {
    let help_text = if app.list_component.in_search_mode {
        vec![
            Span::raw("Esc: Exit Search"),
            Span::raw("  Enter: Edit"),
            Span::raw("  ↑/↓: Navigate"),
            Span::raw("  F2: Rename"),
            Span::raw("  ^D: Delete"),
            Span::raw("  ^S: Save"),
            Span::raw("  ^W: Save All"),
        ]
    } else {
        vec![
            Span::raw("Esc: Close"),
            Span::raw("  Enter: Edit"),
            Span::raw("  K/↑: Up"),
            Span::raw("  J/↓: Down"),
            Span::raw("  N: New"),
            Span::raw("  F2: Rename"),
            Span::raw("  D: Delete"),
            Span::raw("  S: Save Selected"),
            Span::raw("  W: Save All"),
            Span::raw("  /:Search"),
        ]
    };

    let help = Text::from(Line::from(help_text))
        .left_aligned()
        .style(Theme::new().text_dim());

    frame.render_widget(help, area);
}

fn rename_state(frame: &mut Frame<'_>, area: Rect) {
    let help = Text::from(Line::from(vec![
        Span::raw("Esc: Cancel"),
        Span::raw("  Enter: Confirm"),
    ]))
    .left_aligned()
    .style(Theme::new().text_dim());

    frame.render_widget(help, area);
}
