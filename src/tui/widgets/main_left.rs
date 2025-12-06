use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::tui::{
    app::{App, AppState},
    theme::Theme,
};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .list_component
        .profile_names
        .iter()
        .map(|name| {
            let display_text = if app.list_component.dirty_profiles.contains(name) {
                vec![
                    Span::from(name.as_str()),
                    Span::styled("*", Theme::new().text_highlight()),
                ]
            } else {
                vec![Span::from(name.as_str())]
            };
            ListItem::new(Text::from(Line::from(display_text)))
        })
        .collect();

    let title = if app.list_component.profile_names.is_empty() {
        "Profile List (0/0)".to_string()
    } else {
        format!(
            "Profile List ({}/{})",
            app.list_component.selected_index + 1,
            app.list_component.profile_names.len()
        )
    };

    let mut list = List::new(items)
        .highlight_style(Theme::new().selection_active())
        .highlight_symbol("> ");

    let mut block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(title).left_aligned());

    if app.state == AppState::List {
        block = block
            .border_style(Theme::new().block_active())
            .border_type(ratatui::widgets::BorderType::Thick);
    } else {
        block = block.border_style(Theme::new().block_inactive());
    }

    list = list.block(block);

    let mut list_state = ListState::default();
    if !app.list_component.profile_names.is_empty() {
        list_state.select(Some(app.list_component.selected_index));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}
