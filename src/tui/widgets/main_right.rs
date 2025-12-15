use super::empty;
use crate::tui::{
    app::{App, AppState, MainRightViewMode},
    //    components::edit::{EditFocus, EditVariableFocus}, // Removed unused import
    theme::Theme,
    utils::inner,
};
use crate::{GLOBAL_PROFILE_MARK, config::models::Profile};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let theme = Theme::new();

    if app.list_view.filtered_profiles().is_empty() {
        render_empty_profiles_view(frame, area, &theme);
        return;
    }

    // Safe to unwrap because we already checked that filtered_profiles() is not empty
    let selected_name = app.list_view.current_profile().unwrap();

    let display_name = if selected_name == GLOBAL_PROFILE_MARK {
        "GLOBAL"
    } else {
        selected_name
    };

    // Check if we are in Edit mode
    if app.state == AppState::Edit {
        crate::tui::views::edit::render(frame, area, app);
    } else {
        match app.main_right_view_mode {
            MainRightViewMode::Raw => {
                // View Mode
                let profile = match app.config_manager.get_profile(selected_name) {
                    Some(p) => p,
                    None => {
                        render_error_state(frame, area, display_name, &theme);
                        return;
                    }
                };

                render_raw_mode(frame, area, display_name, profile, &theme);
            }
            MainRightViewMode::Expand => {
                render_expand_mode(frame, area, display_name, app, &theme);
            }
        }
    }
}

fn render_empty_profiles_view(frame: &mut Frame<'_>, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_inactive())
        .title("No Profiles");
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    let line = Line::styled(
        "No profiles are currently selected or available.",
        Style::default().dim(),
    )
    .centered();
    empty::render(frame, inner(inner_area), line, 1);
}

fn render_error_state(frame: &mut Frame, area: Rect, name: &str, theme: &Theme) {
    let block = Block::default().borders(Borders::ALL).title("Error");
    let p = Paragraph::new(format!("Could not find profile '{name}'"))
        .block(block)
        .style(theme.text_error());
    frame.render_widget(p, area);
}

fn render_raw_mode(
    frame: &mut Frame,
    area: Rect,
    profile_name: &str,
    profile: &Profile,
    theme: &Theme,
) {
    let title = format!("Contents for '{profile_name}'");
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_inactive())
        .title_top(
            Line::from(title)
                .left_aligned()
                .style(theme.block_title_inactive()),
        );

    let inner_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    let chunks = Layout::vertical([
        Constraint::Percentage(30), // Inherited Profiles
        Constraint::Percentage(70), // Variables
    ])
    .split(inner_area);

    // Render Inherited Profiles (View)
    let inherited_items: Vec<ListItem> = profile
        .profiles
        .iter()
        .map(|p_name| ListItem::new(p_name.clone()))
        .collect();

    let is_empty = inherited_items.is_empty();

    let list = List::new(inherited_items).block(
        Block::new()
            .title("Inherited Profiles")
            .borders(Borders::ALL)
            .border_style(theme.block_inactive()),
    );

    if is_empty {
        empty::profile_not_inherited(frame, chunks[0])
    }
    frame.render_widget(list, chunks[0]);

    // Render Variables (View)
    let mut variables: Vec<_> = profile.variables.iter().collect();
    variables.sort_by_key(|(k, _)| k.to_string());

    let var_rows: Vec<Row> = variables
        .into_iter()
        .map(|(k, v)| Row::new(vec![k.clone(), v.clone()]))
        .collect();

    let is_empty = var_rows.is_empty();

    let table = Table::new(
        var_rows,
        [Constraint::Percentage(30), Constraint::Percentage(70)],
    )
    .header(Row::new(vec!["Key", "Value"]).style(theme.text_highlight()))
    .block(
        Block::new()
            .title("Variables")
            .borders(Borders::ALL)
            .border_style(theme.block_inactive()),
    );

    if is_empty {
        empty::variable_not_defined(frame, chunks[1]);
    }
    frame.render_widget(table, chunks[1]);
}

fn render_expand_mode(
    frame: &mut Frame<'_>,
    area: Rect,
    profile_name: &str,
    app: &App,
    theme: &Theme,
) {
    let title = format!("Expanded for '{profile_name}'");
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_inactive())
        .title_top(
            Line::from(title)
                .left_aligned()
                .style(theme.block_title_inactive()),
        );

    let inner_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    if let Some(expanded_vars) = &app.expand_env_vars {
        let mut variables: Vec<(&String, &String)> = expanded_vars.iter().collect();
        variables.sort_by_key(|(k, _)| k.to_string());

        let var_rows: Vec<Row> = variables
            .into_iter()
            .map(|(k, v)| Row::new(vec![k.clone(), v.clone()]))
            .collect();

        let is_empty = var_rows.is_empty();

        let table = Table::new(
            var_rows,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        )
        .header(Row::new(vec!["Key", "Value"]).style(theme.text_highlight()))
        .block(
            Block::new()
                .title("Variables")
                .borders(Borders::ALL)
                .border_style(theme.block_inactive()),
        );

        if is_empty {
            empty::variable_not_defined(frame, area);
        }
        frame.render_widget(table, inner_area);
    } else {
        empty::variable_not_defined(frame, area);
    }
}
