use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState,
    },
};

use crate::tui::{
    app::{App, AppState},
    components::edit::{EditFocus, EditVariableFocus},
    theme::Theme,
};
use crate::{GLOBAL_PROFILE_MARK, config::models::Profile};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let theme = Theme::new();

    if app.list_component.all_profiles().is_empty() {
        render_empty_state(frame, area, &theme);
        return;
    }

    // Clone the selected name to avoid borrowing issues
    let selected_name =
        app.list_component.all_profiles()[app.list_component.selected_index()].clone();

    let display_name = if selected_name == GLOBAL_PROFILE_MARK {
        "GLOBAL"
    } else {
        &selected_name
    };
    // Check if we are in Edit mode
    if app.state == AppState::Edit {
        render_edit_mode(frame, area, app, display_name, &theme);
    } else {
        // View Mode
        let profile = match app.config_manager.get_profile(&selected_name) {
            Some(p) => p,
            None => {
                render_error_state(frame, area, display_name, &theme);
                return;
            }
        };

        render_view_mode(frame, area, display_name, profile, &theme);
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

    // Calculate actual visible rows for variables area
    // Area height - borders (2) - header row (2 with bottom_margin)
    let variables_inner_height = variables_area.height.saturating_sub(2) as usize;
    let actual_visible_rows = variables_inner_height.saturating_sub(2).max(1); // Subtract header

    let vars_focus = edit.current_focus() == EditFocus::Variables;
    let profiles_focus = edit.current_focus() == EditFocus::Profiles;

    // --- PROFILES SECTION ---
    let current_prof_idx = if edit.profiles_count() == 0 {
        0
    } else {
        edit.selected_profile_index() + 1
    };
    let profiles_title = format!(
        "Inherited Profiles ({}/{})",
        current_prof_idx,
        edit.profiles_count()
    );

    let prof_border_style = if profiles_focus {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    // Calculate actual visible rows for profiles
    let profiles_inner_height = profiles_area.height.saturating_sub(2) as usize; // Remove borders
    let actual_visible_profiles = profiles_inner_height.max(1);

    // Calculate scroll offset based on actual viewport
    let render_profile_scroll = edit.calculate_profile_scroll_offset(actual_visible_profiles);

    let profile_items: Vec<ListItem> = edit
        .profiles()
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

    let mut list_state = ListState::default().with_offset(render_profile_scroll);
    list_state.select(Some(edit.selected_profile_index()));

    frame.render_stateful_widget(profiles_list, profiles_area, &mut list_state);

    // Scrollbar for profiles (using imports from variables section)

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    let max_scroll = edit
        .profiles_count()
        .saturating_sub(actual_visible_profiles)
        + 1;
    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(render_profile_scroll);

    frame.render_stateful_widget(
        scrollbar,
        profiles_area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // --- VARIABLES SECTION ---
    let current_var_idx = if edit.variables_count() == 0 {
        0
    } else {
        edit.selected_variable_index() + 1
    };
    let vars_title = format!("Variables ({}/{})", current_var_idx, edit.variables_count());

    let vars_border_style = if vars_focus && !edit.is_editing() {
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

    let variable_rows: Vec<Row> = edit
        .variables_for_rendering()
        .iter()
        .enumerate()
        .map(|(idx, (k, v))| {
            let key_text = k.text();
            let value_text = v.text();
            let selected = idx == edit.selected_variable_index();
            let _is_key_focused = edit.variable_column_focus() == EditVariableFocus::Key;

            let (key_style, value_style) = if selected && vars_focus {
                match edit.variable_column_focus() {
                    EditVariableFocus::Key => (theme.cell_focus(), theme.selection_active()),
                    EditVariableFocus::Value => (theme.selection_active(), theme.cell_focus()),
                }
            } else {
                (theme.text_normal(), theme.text_normal())
            };

            Row::new(vec![
                Cell::from(key_text).style(key_style),
                Cell::from(value_text).style(value_style),
            ])
        })
        .collect();

    // Calculate the scroll offset to use for rendering, adjusted for actual viewport
    let render_scroll_offset = edit.calculate_variable_scroll_offset(actual_visible_rows);

    let mut table_state = TableState::default().with_offset(render_scroll_offset);
    if vars_focus && !edit.variables_for_rendering().is_empty() {
        table_state.select(Some(edit.selected_variable_index()));
    }

    let col_widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
    let table = Table::new(variable_rows, col_widths)
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

    // Calculate max scroll position: total items that can be scrolled
    let max_scroll = edit.variables_count().saturating_sub(actual_visible_rows) + 1;
    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(render_scroll_offset);

    frame.render_stateful_widget(
        scrollbar,
        variables_area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // Render variable input popup if editing
    if edit.is_editing()
        && let Some(input_state) = edit.variable_input_state()
    {
        let table_inner_area = variables_block.inner(variables_area);

        let vis_idx = edit
            .selected_variable_index()
            .saturating_sub(render_scroll_offset);

        // Position: table_inner_area.y + header row (2 lines) + visual row index
        let row_y = table_inner_area.y + 2 + vis_idx as u16;

        let is_key_focused = input_state.is_key_focused;

        let col_index = if is_key_focused { 0 } else { 1 };

        let layout = Layout::horizontal(col_widths).spacing(1);
        let column_chunks = layout.split(table_inner_area);
        let cell_area = column_chunks[col_index];

        let popup_area = Rect {
            x: cell_area.x.saturating_sub(1),
            y: row_y.saturating_sub(1),
            width: cell_area.width + 2,
            height: 3,
        };

        let title = if is_key_focused {
            "Edit Variable"
        } else {
            "Edit Value"
        };

        // Create temporary Input for rendering
        let temp_input = crate::tui::utils::Input::from_parts(
            input_state.text.to_string(),
            input_state.cursor_pos,
            input_state.error.map(|s| s.to_string()),
        );

        render_variable_input_popup(frame, popup_area, &temp_input, title, theme);
    }

    // Render dependency selector if open
    if edit.is_dependency_selector_open()
        && let Some(selector_state) = edit.dependency_selector_state()
    {
        render_dependency_selector(frame, selector_state, theme);
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

    let border_style = if input.is_valid() {
        theme.block_active()
    } else {
        theme.text_error()
    };

    let mut block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    if !input.is_valid()
        && let Some(err) = input.error_message()
    {
        block = block.title_bottom(Line::from(err).style(theme.text_error()).right_aligned());
    }

    let inner_area = block.inner(area);

    let text = input.text();
    let cursor_pos = input.cursor_position();

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

    let paragraph = Paragraph::new(text).scroll((0, scroll_offset));

    frame.render_widget(block, area);
    frame.render_widget(paragraph, inner_area);
    frame.set_cursor_position((
        inner_area.x + cursor_display_pos - scroll_offset,
        inner_area.y,
    ));
}

fn render_dependency_selector(
    frame: &mut Frame,
    selector_state: crate::tui::components::edit::DependencySelectorState,
    theme: &Theme,
) {
    use ratatui::layout::Margin;
    use ratatui::widgets::{Clear, List, ListItem, ListState};
    use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

    let area = crate::tui::utils::centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    // Render outer block with borders and title
    let outer_block = Block::default()
        .title(selector_state.title)
        .borders(Borders::ALL)
        .border_style(theme.block_active())
        .border_type(ratatui::widgets::BorderType::Thick);

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Split the inner area: list + help section (2 lines)
    let chunks = Layout::vertical([
        Constraint::Min(0),    // List area
        Constraint::Length(2), // Help section
    ])
    .split(inner_area);

    let list_area = chunks[0];
    let help_area = chunks[1];

    // Render list items with border
    let items: Vec<ListItem> = selector_state
        .options
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let selected = selector_state.selected_indices.contains(&idx);
            let marker = if selected { "[✓] " } else { "[ ] " };
            ListItem::new(format!("{marker}{name}"))
        })
        .collect();

    // Calculate current position and selected count
    let current_pos = if selector_state.options.is_empty() {
        0
    } else {
        selector_state.current_index + 1
    };
    let total_count = selector_state.options.len();
    let selected_count = selector_state.selected_indices.len();

    let left_title = Line::from(format!("{current_pos}/{total_count}")).left_aligned();
    let right_title = Line::from(format!("Selected: {selected_count}")).right_aligned();

    let list = List::new(items)
        .block(
            Block::default()
                .title_top(left_title)
                .title_top(right_title)
                .borders(Borders::ALL)
                .border_style(theme.block_inactive()),
        )
        .highlight_style(theme.row_selected());

    let mut list_state = ListState::default();
    list_state.select(Some(selector_state.current_index));

    frame.render_stateful_widget(list, list_area, &mut list_state);

    // Add scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    // Calculate actual visible height (subtract borders of the list block)
    let inner_height = list_area.height.saturating_sub(2) as usize;
    let actual_visible = inner_height.max(1);
    let max_scroll = selector_state.options.len().saturating_sub(actual_visible) + 1;

    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(
        selector_state
            .current_index
            .saturating_sub(actual_visible / 2)
            .min(max_scroll.saturating_sub(1)),
    );

    frame.render_stateful_widget(
        scrollbar,
        list_area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // Render help section
    let help_info = [
        vec![
            Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
            Span::raw(": Confirm"),
        ],
        vec![
            Span::styled("↑/↓", Style::default().fg(Color::Rgb(255, 138, 199))),
            Span::raw(": Navigate"),
        ],
        vec![
            Span::styled("Enter", Style::default().fg(Color::LightBlue)),
            Span::raw("/"),
            Span::styled("Space", Style::default().fg(Color::LightBlue)),
            Span::raw(": Toggle"),
        ],
    ];

    let help_spans = create_selector_help_spans(&help_info, help_area);
    let help_paragraph = Paragraph::new(help_spans).style(Style::default());
    frame.render_widget(help_paragraph, help_area);
}

fn create_selector_help_spans<'a>(help_info: &'a [Vec<Span<'a>>], area: Rect) -> Vec<Line<'a>> {
    let total_width = area.width as usize;
    let mut lines: Vec<Line> = vec![];
    let mut current_line_spans: Vec<Span> = vec![];
    let mut current_line_width = 0;
    let max_help_lines = 2;

    for info in help_info {
        if lines.len() >= max_help_lines {
            break;
        }
        let item_width: usize = info.iter().map(|span| span.width()).sum();
        let separator_width = if !current_line_spans.is_empty() { 2 } else { 0 };

        if current_line_width + separator_width + item_width > total_width
            && !current_line_spans.is_empty()
        {
            if lines.len() < max_help_lines {
                lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                current_line_width = 0;
            } else {
                break;
            }
        }
        if !current_line_spans.is_empty() {
            current_line_spans.push(Span::raw("  "));
            current_line_width += 2;
        }
        current_line_spans.extend_from_slice(info);
        current_line_width += item_width;
    }
    if !current_line_spans.is_empty() && lines.len() < max_help_lines {
        lines.push(Line::from(current_line_spans));
    }
    lines
}
