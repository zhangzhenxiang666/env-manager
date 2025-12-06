use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

use crate::tui::{
    app::{App, AppState},
    theme::Theme,
};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // Handle Empty State
    if app.list_component.profile_names.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::new().block_inactive())
            .title("Profile Details");
        let p = Paragraph::new("No profiles exist.").block(block);
        frame.render_widget(p, area);
        return;
    }

    // Get Data
    let selected_name = &app.list_component.profile_names[app.list_component.selected_index];
    let profile = match app.config_manager.app_config.profiles.get(selected_name) {
        Some(p) => p,
        None => {
            // This should not happen if the app state is consistent
            let block = Block::default().borders(Borders::ALL).title("Error");
            let p =
                Paragraph::new(format!("Could not find profile '{selected_name}'")).block(block);
            frame.render_widget(p, area);
            return;
        }
    };

    // Create Outer Block
    let mut main_block = Block::default().borders(Borders::ALL);
    let title = format!("Contents for '{selected_name}'");

    if app.state == AppState::Edit {
        main_block = main_block
            .border_style(Theme::new().block_active())
            .border_type(ratatui::widgets::BorderType::Thick)
            .title_top(
                Line::from(title)
                    .left_aligned()
                    .style(Theme::new().block_title_active()),
            );
    } else {
        main_block = main_block
            .border_style(Theme::new().block_inactive())
            .title_top(
                Line::from(title)
                    .left_aligned()
                    .style(Theme::new().block_title_inactive()),
            );
    }

    // Render the main block first to draw its borders, then calculate inner area
    let inner_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    // Inner Layout
    let inner_chunks = Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(inner_area);

    // Render Inherited Profiles List
    let inherited_items: Vec<ListItem> = profile
        .profiles
        .iter()
        .map(|p_name| ListItem::new(p_name.clone()))
        .collect();
    let inherited_list = List::new(inherited_items).block(
        Block::new()
            .title("Inherited Profiles")
            .borders(Borders::ALL)
            .border_style(if app.state == AppState::Edit {
                Theme::new().block_active()
            } else {
                Theme::new().block_inactive()
            }),
    );
    frame.render_widget(inherited_list, inner_chunks[0]);

    // Render Variables Table
    let mut variables: Vec<_> = profile.variables.iter().collect();
    variables.sort_by_key(|(k, _)| k.to_string());

    let var_rows: Vec<Row> = variables
        .into_iter()
        .map(|(k, v)| Row::new(vec![k.clone(), v.clone()]))
        .collect();

    let var_table = Table::new(
        var_rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .header(Row::new(vec!["Key", "Value"]).style(Theme::new().text_highlight()))
    .block(
        Block::new()
            .title("Variables")
            .borders(Borders::ALL)
            .border_style(if app.state == AppState::Edit {
                Theme::new().block_active()
            } else {
                Theme::new().block_inactive()
            }),
    );

    frame.render_widget(var_table, inner_chunks[1]);
}
