use crate::tui::app::AppState::{self, List};
use crate::tui::theme::Theme;
use ratatui::prelude::*;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &crate::tui::app::App) {
    let version_info = Text::from(vec![Line::raw(format!(
        "Env-Mnage {}",
        env!("CARGO_PKG_VERSION")
    ))])
    .right_aligned();

    match app.state {
        List => list_state(frame, area, app),
        AppState::Edit => edit_state(frame, area, app),
        AppState::Rename => rename_state(frame, area),
        _ => {}
    }
    frame.render_widget(version_info, area);
}

fn list_state(frame: &mut Frame<'_>, area: Rect, app: &crate::tui::app::App) {
    let help_text = if app.list_view.is_searching() {
        vec![
            Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
            Span::raw(": Exit Search  "),
            Span::styled("Enter", Style::default().fg(Color::Rgb(106, 255, 160))),
            Span::raw(": Edit  "),
            Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
            Span::raw(": Switch View  "),
            Span::styled("↑↓", Style::default().fg(Color::Rgb(255, 138, 199))),
            Span::raw(": Navigate  "),
            Span::styled("F2", Style::default().fg(Color::LightYellow)),
            Span::raw(": Rename  "),
            Span::styled("^D", Style::default().fg(Color::LightRed)),
            Span::raw(": Delete  "),
            Span::styled("^S", Style::default().fg(Color::LightBlue)),
            Span::raw(": Save  "),
            Span::styled("^W", Style::default().fg(Color::LightCyan)),
            Span::raw(": Save All"),
        ]
    } else {
        vec![
            Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
            Span::raw(": Close  "),
            Span::styled("Enter", Style::default().fg(Color::Rgb(106, 255, 160))),
            Span::raw(": Edit  "),
            Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
            Span::raw(": Switch View  "),
            Span::styled("↑↓", Style::default().fg(Color::Rgb(255, 138, 199))),
            Span::raw(": Navigate  "),
            Span::styled("N", Style::default().fg(Color::LightGreen)),
            Span::raw(": New  "),
            Span::styled("F2", Style::default().fg(Color::LightYellow)),
            Span::raw(": Rename  "),
            Span::styled("D", Style::default().fg(Color::LightRed)),
            Span::raw(": Delete  "),
            Span::styled("S", Style::default().fg(Color::LightBlue)),
            Span::raw(": Save Selected  "),
            Span::styled("W", Style::default().fg(Color::LightCyan)),
            Span::raw(": Save All  "),
            Span::styled("/", Style::default().fg(Color::LightMagenta)),
            Span::raw(": Search"),
        ]
    };

    let help = Text::from(Line::from(help_text))
        .left_aligned()
        .style(Theme::new().text_dim());

    frame.render_widget(help, area);
}

fn rename_state(frame: &mut Frame<'_>, area: Rect) {
    let help = Text::from(Line::from(vec![
        Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
        Span::raw(": Cancel  "),
        Span::styled("Enter", Style::default().fg(Color::Rgb(106, 255, 160))),
        Span::raw(": Confirm"),
    ]))
    .left_aligned()
    .style(Theme::new().text_dim());

    frame.render_widget(help, area);
}

fn edit_state(frame: &mut Frame<'_>, area: Rect, app: &crate::tui::app::App) {
    use crate::tui::views::edit::{EditFocus, EditVariableFocus};

    let help_text = if app.edit_view.is_editing() {
        // Editing popup is active - show editing-specific help
        match app.edit_view.variable_column_focus() {
            EditVariableFocus::Key => vec![
                Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
                Span::raw(": Cancel  "),
                Span::styled("Enter", Style::default().fg(Color::Rgb(106, 255, 160))),
                Span::raw(": Confirm  "),
                Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
                Span::raw(": Switch Field"),
            ],
            EditVariableFocus::Value => vec![
                Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
                Span::raw(": Cancel  "),
                Span::styled("Enter", Style::default().fg(Color::Rgb(106, 255, 160))),
                Span::raw(": Confirm  "),
                Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
                Span::raw(": Switch Field"),
            ],
        }
    } else {
        // Navigation mode - show section-specific help
        match app.edit_view.current_focus() {
            EditFocus::Profiles => vec![
                Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
                Span::raw(": Back  "),
                Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
                Span::raw(": Focus  "),
                Span::styled("↑/↓", Style::default().fg(Color::Rgb(255, 138, 199))),
                Span::raw(": Navigate  "),
                Span::styled("N", Style::default().fg(Color::LightGreen)),
                Span::raw(": Add Dep  "),
                Span::styled("D", Style::default().fg(Color::LightRed)),
                Span::raw(": Del Dep"),
            ],
            EditFocus::Variables => vec![
                Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
                Span::raw(": Back  "),
                Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
                Span::raw(": Focus  "),
                Span::styled("↑↓←→", Style::default().fg(Color::Rgb(255, 138, 199))),
                Span::raw(" : Navigate  "),
                Span::styled("A", Style::default().fg(Color::LightYellow)),
                Span::raw(": Add Var  "),
                Span::styled("E", Style::default().fg(Color::LightBlue)),
                Span::raw(": Edit  "),
                Span::styled("D", Style::default().fg(Color::LightRed)),
                Span::raw(": Del Var"),
            ],
        }
    };

    let help = Text::from(Line::from(help_text))
        .left_aligned()
        .style(Theme::new().text_dim());

    frame.render_widget(help, area);
}
