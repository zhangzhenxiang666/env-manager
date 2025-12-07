use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState,
    },
};

use crate::config::models::Profile;
use crate::tui::{
    app::{App, AppState},
    components::edit::{EditFocus, EditVariableFocus},
    theme::Theme,
};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let theme = Theme::new();

    if app.list_component.profile_names.is_empty() {
        render_empty_state(frame, area, &theme);
        return;
    }

    let selected_name = &app.list_component.profile_names[app.list_component.selected_index];

    // Check if we are in Edit mode
    if app.state == AppState::Edit {
        render_edit_mode(frame, area, app, selected_name, &theme);
    } else {
        // View Mode
        let profile = match app.config_manager.app_config.profiles.get(selected_name) {
            Some(p) => p,
            None => {
                render_error_state(frame, area, selected_name, &theme);
                return;
            }
        };
        render_view_mode(frame, area, selected_name, profile, &theme);
    }
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

fn render_view_mode(
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

    let list = List::new(inherited_items).block(
        Block::new()
            .title("Inherited Profiles")
            .borders(Borders::ALL)
            .border_style(theme.block_inactive()),
    );
    frame.render_widget(list, chunks[0]);

    // Render Variables (View)
    let mut variables: Vec<_> = profile.variables.iter().collect();
    variables.sort_by_key(|(k, _)| k.to_string());

    let var_rows: Vec<Row> = variables
        .into_iter()
        .map(|(k, v)| Row::new(vec![k.clone(), v.clone()]))
        .collect();

    let table = Table::new(
        var_rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .header(Row::new(vec!["Key", "Value"]).style(theme.text_highlight()))
    .block(
        Block::new()
            .title("Variables")
            .borders(Borders::ALL)
            .border_style(theme.block_inactive()),
    );
    frame.render_widget(table, chunks[1]);
}

fn render_edit_mode(frame: &mut Frame, area: Rect, app: &App, profile_name: &str, theme: &Theme) {
    let edit = &app.edit_component;
    let title = format!("Editing '{profile_name}'");

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_active())
        .border_type(ratatui::widgets::BorderType::Thick)
        .title_top(
            Line::from(title)
                .left_aligned()
                .style(theme.block_title_active()),
        );

    let inner_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    // Vertical Layout: Profiles Top (30%), Variables Bottom (70%) - mirroring view mode or as requested
    // User requested "original up/down structure". Original view mode is 30% Inherited, 70% Vars.
    let chunks = Layout::vertical([
        Constraint::Percentage(30), // Inherited Profiles
        Constraint::Percentage(70), // Variables
    ])
    .split(inner_area);

    let profiles_area = chunks[0];
    let variables_area = chunks[1];

    let vars_focus = edit.focus == EditFocus::Variables;
    let profiles_focus = edit.focus == EditFocus::Profiles;

    // --- PROFILES SECTION ---
    let current_prof_idx = if edit.profiles.is_empty() {
        0
    } else {
        edit.selected_profile_index + 1
    };
    let profiles_title = format!(
        "Inherited Profiles ({}/{})",
        current_prof_idx,
        edit.profiles.len()
    );

    let prof_border_style = if profiles_focus {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let profile_items: Vec<ListItem> = edit
        .profiles
        .iter()
        .map(|p| ListItem::new(p.as_str()))
        .collect();

    let profiles_list = List::new(profile_items).block(
        Block::new()
            .title(profiles_title)
            .borders(Borders::ALL)
            .border_style(prof_border_style),
    );

    let profiles_list = if profiles_focus {
        profiles_list.highlight_style(theme.row_selected())
    } else {
        profiles_list
    };

    let mut list_state = ListState::default().with_offset(edit.profile_scroll_offset);
    list_state.select(Some(edit.selected_profile_index));

    frame.render_stateful_widget(profiles_list, profiles_area, &mut list_state);

    // --- VARIABLES SECTION ---
    let current_var_idx = if edit.variables.is_empty() {
        0
    } else {
        edit.selected_variable_index + 1
    };
    let vars_title = format!("Variables ({}/{})", current_var_idx, edit.variables.len());

    let vars_border_style = if vars_focus && !edit.is_editing_variable {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let variables_block = Block::default()
        .title_top(Line::from(vars_title).left_aligned())
        .borders(Borders::ALL)
        .border_style(vars_border_style);

    let header = Row::new(vec!["Key", "Value"])
        .style(Style::new().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = edit
        .variables
        .iter()
        .enumerate()
        .map(|(i, (key_input, value_input))| {
            let is_row_selected = vars_focus && i == edit.selected_variable_index;

            let (key_style, value_style) = if is_row_selected {
                match edit.variable_column_focus {
                    EditVariableFocus::Key => (theme.cell_focus(), theme.selection_active()),
                    EditVariableFocus::Value => (theme.selection_active(), theme.cell_focus()),
                }
            } else {
                (theme.text_normal(), theme.text_normal())
            };

            Row::new(vec![
                Cell::from(key_input.text.as_str()).style(key_style),
                Cell::from(value_input.text.as_str()).style(value_style),
            ])
        })
        .collect();

    let mut table_state = TableState::default().with_offset(edit.variable_scroll_offset);
    if vars_focus && !edit.variables.is_empty() {
        table_state.select(Some(edit.selected_variable_index));
    }

    let col_widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
    let table = Table::new(rows, col_widths)
        .header(header)
        .block(variables_block.clone());

    frame.render_stateful_widget(table, variables_area, &mut table_state);

    // Scrollbar
    use ratatui::layout::Margin;
    use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    let mut scrollbar_state = ScrollbarState::new(edit.variables.len())
        .viewport_content_length(crate::tui::components::edit::EditComponent::MAX_VARIABLES_HEIGHT)
        .position(edit.variable_scroll_offset);

    frame.render_stateful_widget(
        scrollbar,
        variables_area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // Popup Input Box for editing
    if vars_focus && edit.is_editing_variable {
        if let Some(focused_input) = app.edit_component.get_focused_variable_input_ref() {
            let table_inner_area = variables_block.inner(variables_area);
            let row_index = edit.selected_variable_index;

            let visual_row_index = row_index.saturating_sub(edit.variable_scroll_offset);

            let row_y = table_inner_area.y + 2 + visual_row_index as u16;

            let col_index = match edit.variable_column_focus {
                EditVariableFocus::Key => 0,
                EditVariableFocus::Value => 1,
            };

            let layout = Layout::horizontal(col_widths).spacing(1);
            let column_chunks = layout.split(table_inner_area);
            let cell_area = column_chunks[col_index];

            let popup_area = Rect {
                x: cell_area.x.saturating_sub(1),
                y: row_y.saturating_sub(1),
                width: cell_area.width + 2,
                height: 3,
            };

            let title = match edit.variable_column_focus {
                EditVariableFocus::Key => "Edit Variable",
                EditVariableFocus::Value => "Edit Value",
            };

            render_variable_input_popup(frame, popup_area, focused_input, title, theme);
        }
    }

    // Render Select Popup if visible
    if edit.show_select_popup {
        crate::tui::widgets::select_popup::render(frame, &edit.select_popup);
    }
}

fn render_variable_input_popup(
    frame: &mut Frame,
    area: Rect,
    input: &crate::tui::utils::Input,
    title: &str,
    theme: &Theme,
) {
    use unicode_width::UnicodeWidthStr;

    frame.render_widget(Clear, area);

    let border_style = if input.is_valid {
        theme.block_active()
    } else {
        theme.text_error()
    };

    let mut block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    if !input.is_valid {
        if let Some(err) = &input.error_message {
            block = block.title_bottom(
                Line::from(err.as_str())
                    .style(theme.text_error())
                    .right_aligned(),
            );
        }
    }

    let inner_area = block.inner(area);

    let text = &input.text;
    let cursor_pos = input.cursor_position;

    let prefix_width = text
        .chars()
        .take(cursor_pos)
        .map(|c| UnicodeWidthStr::width(c.to_string().as_str()))
        .sum::<usize>();

    let cursor_display_pos = prefix_width as u16;
    let scroll_offset = if cursor_display_pos >= inner_area.width {
        cursor_display_pos - inner_area.width + 1
    } else {
        0
    };

    let paragraph = Paragraph::new(text.as_str()).scroll((0, scroll_offset));

    frame.render_widget(block, area);
    frame.render_widget(paragraph, inner_area);
    frame.set_cursor_position((
        inner_area.x + cursor_display_pos - scroll_offset,
        inner_area.y,
    ));
}
