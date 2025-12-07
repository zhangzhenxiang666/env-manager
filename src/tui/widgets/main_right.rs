use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

use crate::config::models::Profile;
use crate::tui::{
    app::{App, AppState},
    theme::Theme,
};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let theme = Theme::new();

    if app.list_component.profile_names.is_empty() {
        render_empty_state(frame, area, &theme);
        return;
    }

    let selected_name = &app.list_component.profile_names[app.list_component.selected_index];
    let profile = match app.config_manager.app_config.profiles.get(selected_name) {
        Some(p) => p,
        None => {
            render_error_state(frame, area, selected_name, &theme);
            return;
        }
    };

    render_profile_details(frame, area, app, selected_name, profile, &theme);
}

fn render_empty_state(frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_inactive())
        .title("Profile Details");
    let p = Paragraph::new("No profiles exist.").block(block);
    frame.render_widget(p, area);
}

fn render_error_state(frame: &mut Frame, area: Rect, name: &str, _theme: &Theme) {
    let block = Block::default().borders(Borders::ALL).title("Error");
    let p = Paragraph::new(format!("Could not find profile '{name}'")).block(block);
    frame.render_widget(p, area);
}

fn render_profile_details(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    profile_name: &str,
    profile: &Profile,
    theme: &Theme,
) {
    let is_editing = app.state == AppState::Edit;

    let border_style = if is_editing {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let title_style = if is_editing {
        theme.block_title_active()
    } else {
        theme.block_title_inactive()
    };

    let title = format!("Contents for '{profile_name}'");
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .border_type(if is_editing {
            ratatui::widgets::BorderType::Thick
        } else {
            ratatui::widgets::BorderType::Plain
        })
        .title_top(Line::from(title).left_aligned().style(title_style));

    let inner_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    let chunks = Layout::vertical([
        Constraint::Percentage(30), // Inherited Profiles
        Constraint::Percentage(70), // Variables
    ])
    .split(inner_area);

    render_inherited_profiles_list(frame, chunks[0], profile, is_editing, theme);
    render_variables_table(frame, chunks[1], profile, is_editing, theme);
}

fn render_inherited_profiles_list(
    frame: &mut Frame,
    area: Rect,
    profile: &Profile,
    is_editing: bool,
    theme: &Theme,
) {
    let inherited_items: Vec<ListItem> = profile
        .profiles
        .iter()
        .map(|p_name| ListItem::new(p_name.clone()))
        .collect();

    let border_style = if is_editing {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let list = List::new(inherited_items).block(
        Block::new()
            .title("Inherited Profiles")
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(list, area);
}

fn render_variables_table(
    frame: &mut Frame,
    area: Rect,
    profile: &Profile,
    is_editing: bool,
    theme: &Theme,
) {
    let mut variables: Vec<_> = profile.variables.iter().collect();
    variables.sort_by_key(|(k, _)| k.to_string());

    let var_rows: Vec<Row> = variables
        .into_iter()
        .map(|(k, v)| Row::new(vec![k.clone(), v.clone()]))
        .collect();

    let border_style = if is_editing {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let table = Table::new(
        var_rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .header(Row::new(vec!["Key", "Value"]).style(theme.text_highlight()))
    .block(
        Block::new()
            .title("Variables")
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(table, area);
}
