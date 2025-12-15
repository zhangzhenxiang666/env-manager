use super::app::App;
use super::views::{add_new, list};
use super::widgets::{bottom, confirm_delete_popup, confirm_exit_popup, header};
use crate::tui::app::AppState;
use crate::tui::widgets::main_right;
use ratatui::prelude::*;
use unicode_width::UnicodeWidthStr;

pub fn ui(frame: &mut Frame<'_>, app: &App) {
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(frame.area());

    let required_width = calculate_main_left_width(app);

    let main_windown =
        Layout::horizontal([Constraint::Length(required_width), Constraint::Fill(1)])
            .split(layout[1]);

    header::render(frame, layout[0], app);
    list::render(frame, main_windown[0], app);
    main_right::render(frame, main_windown[1], app);
    bottom::render(frame, layout[2], app);

    match app.state {
        AppState::ConfirmDelete => {
            confirm_delete_popup::render(frame, app);
        }
        AppState::AddNew => {
            add_new::render(frame, app);
        }
        AppState::ConfirmExit => {
            confirm_exit_popup::render(frame, app);
        }
        _ => {}
    }
}

fn calculate_main_left_width(app: &App) -> u16 {
    let profiles = app.list_view.filtered_profiles();
    let max_len = profiles
        .iter()
        .map(|name| UnicodeWidthStr::width(name.as_str()))
        .max()
        .unwrap_or(0);

    // Calculate title widths to prevent truncation
    let filtered_count = profiles.len();
    let current_index = app.list_view.selected_index() + 1;
    let title_str = if filtered_count == 0 {
        "Profile List (0/0)".to_string()
    } else {
        format!("Profile List ({}/{})", current_index, filtered_count)
    };
    let title_width = UnicodeWidthStr::width(title_str.as_str());

    let unsaved_count = app.list_view.unsaved_count();
    let unsaved_width = if unsaved_count > 0 {
        UnicodeWidthStr::width(format!("Unsaved: {}", unsaved_count).as_str())
    } else {
        0
    };

    // +4 for borders/gap between titles
    let min_title_width = title_width + unsaved_width + 4;
    let content_width = max_len + 6;

    (content_width.max(min_title_width)).clamp(25, 60) as u16
}
