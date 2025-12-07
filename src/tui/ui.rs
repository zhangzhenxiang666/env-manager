use super::app::App;
use super::widgets::{add_new_popup, bottom, confirm_delete_popup, header, main_left};
use crate::tui::app::AppState;
use crate::tui::widgets::main_right;
use ratatui::prelude::*;

pub fn ui(frame: &mut Frame<'_>, app: &App) {
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(frame.area());

    let main_windown =
        Layout::horizontal([Constraint::Min(5), Constraint::Fill(1)]).split(layout[1]);

    header::render(frame, layout[0], app);
    main_left::render(frame, main_windown[0], app);
    main_right::render(frame, main_windown[1], app);
    bottom::render(frame, layout[2], app);

    match app.state {
        AppState::ConfirmDelete => {
            confirm_delete_popup::render(frame, app);
        }
        AppState::AddNew => {
            add_new_popup::render(frame, app);
        }
        _ => {}
    }
}
