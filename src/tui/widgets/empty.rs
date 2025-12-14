use crate::tui::utils::inner;
use ratatui::prelude::*;

pub fn render<W: Widget>(frame: &mut Frame<'_>, area: Rect, data: W, higeht: u16) {
    let area = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(higeht),
        Constraint::Min(0),
    ])
    .split(area)[1];

    frame.render_widget(data, area);
}

pub fn variable_not_defined(frame: &mut Frame<'_>, area: Rect) {
    render(
        frame,
        inner(area),
        Line::styled("No variables defined", Style::default().dim()).centered(),
        1,
    );
}

pub fn profile_not_inherited(frame: &mut Frame<'_>, area: Rect) {
    render(
        frame,
        inner(area),
        Line::styled("No inherited profiles selected", Style::default().dim()).centered(),
        1,
    );
}

pub fn profile_not_selectable(frame: &mut Frame<'_>, area: Rect) {
    render(
        frame,
        inner(area),
        Line::styled("No selectable profiles", Style::default().dim()).centered(),
        1,
    );
}
