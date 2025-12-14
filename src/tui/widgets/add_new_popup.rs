use crate::GLOBAL_PROFILE_MARK;
use crate::tui::components::add_new::{AddNewComponent, AddNewFocus, AddNewVariableFocus};
use crate::tui::widgets::empty;
use crate::tui::{app::App, theme::Theme, utils, utils::Input};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Scrollbar,
    ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use std::mem;
use unicode_width::UnicodeWidthStr;

const MAX_HELP_LINES: usize = 2;

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = utils::centered_rect(70, 80, frame.area());
    frame.render_widget(Clear, area);

    let theme = Theme::new();
    let add_new_state = &app.add_new_component;

    let popup_block = Block::default()
        .title("Create New Profile")
        .borders(Borders::ALL)
        .border_style(theme.block_active())
        .border_type(ratatui::widgets::BorderType::Thick);

    let inner_popup_area = popup_block.inner(area);
    frame.render_widget(popup_block, area);

    let main_layout = Layout::vertical([
        Constraint::Length(3), // Name section
        Constraint::Min(0),    // Flexible middle section
        Constraint::Length(2), // Help section
    ])
    .split(inner_popup_area);

    let name_area = main_layout[0];
    let middle_area = main_layout[1];
    let help_area = main_layout[2];

    let middle_layout = Layout::vertical([
        Constraint::Percentage(40), // Profiles section
        Constraint::Percentage(60), // Variables section
    ])
    .split(middle_area);

    let profiles_area = middle_layout[0];
    let variables_area = middle_layout[1];

    render_name_section(frame, add_new_state, name_area, &theme);
    render_profiles_section(frame, app, profiles_area, &theme);
    render_variables_section(frame, app, variables_area, &theme);
    render_help_section(frame, app, help_area);
}

fn render_name_section(
    frame: &mut Frame<'_>,
    add_new: &AddNewComponent,
    area: Rect,
    theme: &Theme,
) {
    let is_focused = add_new.current_focus() == AddNewFocus::Name;

    let border_style = if !add_new.name_input().is_valid() {
        theme.text_error()
    } else if is_focused {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let mut input_block = Block::default()
        .title("Name")
        .borders(Borders::ALL)
        .border_style(border_style);

    if !add_new.name_input().is_valid()
        && let Some(err) = add_new.name_input().error_message()
    {
        input_block =
            input_block.title_bottom(Line::from(err).style(theme.text_error()).right_aligned());
    }

    let text_input_rect = input_block.inner(area);
    frame.render_widget(input_block, area);

    let input_text = add_new.name_input().text();
    let cursor_char_pos = add_new.name_input().cursor_position();

    // Calculate scroll offset for horizontal scrolling
    let prefix_width = input_text
        .chars()
        .take(cursor_char_pos)
        .map(|c| UnicodeWidthStr::width(c.to_string().as_str()))
        .sum::<usize>();

    let cursor_display_pos = prefix_width as u16;
    let input_display_width = text_input_rect.width;
    let scroll_offset = if cursor_display_pos >= input_display_width {
        cursor_display_pos - input_display_width + 1
    } else {
        0
    };

    let input_paragraph = Paragraph::new(input_text)
        .style(theme.text_normal())
        .scroll((0, scroll_offset));
    frame.render_widget(input_paragraph, text_input_rect);

    if is_focused {
        frame.set_cursor_position((
            text_input_rect.x + cursor_display_pos - scroll_offset,
            text_input_rect.y,
        ));
    }

    // Validation message handled by block title now
}

fn render_profiles_section(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let add_new = &app.add_new_component;
    let available_profiles: Vec<_> = app
        .list_component
        .all_profiles()
        .iter()
        .filter(|name| **name != add_new.name_input().text() && *name != GLOBAL_PROFILE_MARK)
        .collect();
    let total_profiles = available_profiles.len();
    let is_focused = add_new.current_focus() == AddNewFocus::Profiles;

    let current_idx = if add_new.profiles_selection_index() >= available_profiles.len() {
        0
    } else {
        add_new.profiles_selection_index() + 1
    };
    let profiles_title = format!("Inherit Profiles ({current_idx}/{total_profiles})");

    let left_title = Line::from(profiles_title).left_aligned();

    let right_title =
        Line::from(format!("Selected: {}", add_new.added_profiles().len())).right_aligned();

    let border_style = if is_focused {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let profiles_block = Block::default()
        .title_top(left_title)
        .title_top(right_title)
        .borders(Borders::ALL)
        .border_style(border_style);

    // Calculate actual visible height for profiles
    let profiles_inner_height = area.height.saturating_sub(2) as usize; // Remove borders
    let actual_visible_profiles = profiles_inner_height.max(1);

    // Calculate scroll offset based on actual viewport
    let render_profile_scroll = add_new.calculate_profile_scroll_offset(actual_visible_profiles);

    let list_items: Vec<ListItem> = available_profiles
        .iter()
        .skip(render_profile_scroll)
        .take(actual_visible_profiles)
        .map(|name| {
            let is_selected = add_new.is_profile_added(name);
            let prefix = if is_selected { "[✓] " } else { "[ ] " };
            ListItem::new(format!("{prefix}{name}"))
        })
        .collect();

    let is_empty = list_items.is_empty();

    let mut results_list = List::new(list_items).block(profiles_block);
    if is_focused {
        results_list = results_list.highlight_style(theme.selection_active());
    }

    let mut list_state = ListState::default();
    if is_focused && !available_profiles.is_empty() {
        list_state.select(Some(
            add_new.profiles_selection_index() - render_profile_scroll,
        ));
    }

    if is_empty {
        empty::profile_not_selectable(frame, area);
    }

    frame.render_stateful_widget(results_list, area, &mut list_state);

    // Scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some("│"));

    // Calculate max scroll position
    let max_scroll = total_profiles.saturating_sub(actual_visible_profiles) + 1;
    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(render_profile_scroll);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_variables_section(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let add_new = &app.add_new_component;
    let is_focused = add_new.current_focus() == AddNewFocus::Variables;

    let current_index = if add_new.variables_count() == 0 {
        0
    } else {
        add_new.selected_variable_index() + 1
    };
    let total_count = add_new.variables_count();

    let left_title =
        Line::from(format!("Variables ({current_index}/{total_count})")).left_aligned();

    let border_style = if is_focused && !add_new.is_editing() {
        theme.block_active()
    } else {
        theme.block_inactive()
    };

    let variables_block = Block::default()
        .title_top(left_title)
        .borders(Borders::ALL)
        .border_style(border_style);

    // Calculate actual visible height for variables
    let variables_inner_height = area.height.saturating_sub(2) as usize; // Remove borders
    let actual_visible_variables = variables_inner_height.saturating_sub(2).max(1); // Subtract header

    // Calculate scroll offset based on actual viewport
    let render_variable_scroll = add_new.calculate_variable_scroll_offset(actual_visible_variables);

    let header = Row::new(vec!["Key", "Value"])
        .style(Style::new().add_modifier(Modifier::BOLD))
        .style(theme.text_highlight())
        .bottom_margin(1);

    let rows: Vec<Row> = add_new
        .variables_for_rendering()
        .iter()
        .enumerate()
        .map(|(i, (key_input, value_input))| {
            let is_row_selected = is_focused && i == add_new.selected_variable_index();

            let (key_style, value_style) = if is_row_selected {
                match add_new.variable_column_focus() {
                    AddNewVariableFocus::Key => (
                        theme.cell_focus(),       // Focused cell
                        theme.selection_active(), // Selected row, unfocused cell
                    ),
                    AddNewVariableFocus::Value => (
                        theme.selection_active(), // Selected row, unfocused cell
                        theme.cell_focus(),       // Focused cell
                    ),
                }
            } else {
                (theme.text_normal(), theme.text_normal())
            };

            Row::new(vec![
                Cell::from(key_input.text()).style(key_style),
                Cell::from(value_input.text()).style(value_style),
            ])
        })
        .skip(render_variable_scroll)
        .collect();
    let is_empty = rows.is_empty();

    let mut table_state = TableState::default();
    if is_focused && add_new.variables_count() > 0 {
        table_state.select(Some(add_new.selected_variable_index()));
    }

    let col_widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
    let table = Table::new(rows, col_widths)
        .header(header)
        .block(variables_block.clone());

    if is_empty {
        empty::variable_not_defined(frame, area);
    }
    frame.render_stateful_widget(table, area, &mut table_state);

    // Scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    // Calculate max scroll position
    let max_scroll = add_new
        .variables_count()
        .saturating_sub(actual_visible_variables)
        + 1;
    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(render_variable_scroll);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // Popup Input Box for editing
    if is_focused
        && add_new.is_editing()
        && let Some(focused_input) = add_new.get_focused_variable_input()
    {
        let table_inner_area = variables_block.inner(area);
        let row_index = add_new.selected_variable_index();

        let visual_row_index = row_index.saturating_sub(render_variable_scroll);

        let row_y = table_inner_area.y + 2 + visual_row_index as u16;

        let col_index = match add_new.variable_column_focus() {
            AddNewVariableFocus::Key => 0,
            AddNewVariableFocus::Value => 1,
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

        let title = match add_new.variable_column_focus() {
            AddNewVariableFocus::Key => "Edit Variable",
            AddNewVariableFocus::Value => "Edit Value",
        };

        render_variable_input_popup(frame, popup_area, focused_input, title, theme);
    }
}

fn render_variable_input_popup(
    frame: &mut Frame,
    area: Rect,
    input: &Input,
    title: &str,
    theme: &Theme,
) {
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

fn render_help_section(frame: &mut Frame<'_>, app: &App, area: Rect) {
    match app.add_new_component.current_focus() {
        AddNewFocus::Name => render_name_help(frame, area),
        AddNewFocus::Profiles => render_profiles_help(frame, area),
        AddNewFocus::Variables => render_variables_help(frame, app, area),
    }
}

fn create_help_spans<'a>(help_info: &'a [Vec<Span<'a>>], area: Rect) -> Vec<Line<'a>> {
    let total_width = area.width as usize;
    let mut lines: Vec<Line> = vec![];
    let mut current_line_spans: Vec<Span> = vec![];
    let mut current_line_width = 0;

    for info in help_info {
        if lines.len() >= MAX_HELP_LINES {
            break;
        }
        let item_width: usize = info.iter().map(|span| span.width()).sum();
        let separator_width = if !current_line_spans.is_empty() { 2 } else { 0 };

        if current_line_width + separator_width + item_width > total_width
            && !current_line_spans.is_empty()
        {
            if lines.len() < MAX_HELP_LINES {
                lines.push(Line::from(mem::take(&mut current_line_spans)));
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
    if !current_line_spans.is_empty() && lines.len() < MAX_HELP_LINES {
        lines.push(Line::from(current_line_spans));
    }
    lines
}

fn render_name_help(frame: &mut Frame<'_>, area: Rect) {
    let help_info = [
        vec![
            Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
            Span::raw(": Cancel"),
        ],
        vec![
            Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
            Span::raw(": Focus"),
        ],
        vec![
            Span::styled("←→", Style::default().fg(Color::Rgb(255, 138, 199))),
            Span::raw(" : Move cursor"),
        ],
        vec![
            Span::styled("Ctrl+s", Style::default().fg(Color::Rgb(106, 255, 160))),
            Span::raw(": Save"),
        ],
    ];
    let lines = create_help_spans(&help_info, area);
    let help_paragraph = Paragraph::new(lines).style(Style::default());
    frame.render_widget(help_paragraph, area);
}

fn render_profiles_help(frame: &mut Frame, area: Rect) {
    let help_info = [
        vec![
            Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
            Span::raw(": Cancel"),
        ],
        vec![
            Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
            Span::raw(": Focus"),
        ],
        vec![
            Span::styled("↑↓", Style::default().fg(Color::Rgb(255, 138, 199))),
            Span::raw(": Navigate"),
        ],
        vec![
            Span::styled("Enter", Style::default().fg(Color::LightBlue)),
            Span::raw("/"),
            Span::styled("Space", Style::default().fg(Color::LightBlue)),
            Span::raw(": Toggle"),
        ],
        vec![
            Span::styled("Ctrl+s", Style::default().fg(Color::Rgb(106, 255, 160))),
            Span::raw(": Save"),
        ],
    ];
    let lines = create_help_spans(&help_info, area);
    let help_paragraph = Paragraph::new(lines).style(Style::default());
    frame.render_widget(help_paragraph, area);
}

fn render_variables_help(frame: &mut Frame, app: &App, area: Rect) {
    let add_new = &app.add_new_component;
    let help_info = if add_new.is_editing() {
        vec![
            vec![
                Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
                Span::raw(": Cancel Edit"),
            ],
            vec![
                Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
                Span::raw(": Switch Field"),
            ],
            vec![
                Span::styled("Enter", Style::default().fg(Color::Rgb(106, 255, 160))),
                Span::raw(": Confirm"),
            ],
        ]
    } else {
        vec![
            vec![
                Span::styled("Esc", Style::default().fg(Color::Rgb(255, 107, 107))),
                Span::raw(": Cancel"),
            ],
            vec![
                Span::styled("Tab", Style::default().fg(Color::Rgb(130, 170, 255))),
                Span::raw(": Focus"),
            ],
            vec![
                Span::styled("↑↓←→", Style::default().fg(Color::Rgb(255, 138, 199))),
                Span::raw(" : Navigate"),
            ],
            vec![
                Span::styled("a", Style::default().fg(Color::LightYellow)),
                Span::raw(": Add"),
            ],
            vec![
                Span::styled("d", Style::default().fg(Color::LightRed)),
                Span::raw(": Delete"),
            ],
            vec![
                Span::styled("e", Style::default().fg(Color::LightBlue)),
                Span::raw(": Edit"),
            ],
            vec![
                Span::styled("Ctrl+s", Style::default().fg(Color::Rgb(106, 255, 160))),
                Span::raw(": Save"),
            ],
        ]
    };
    let lines = create_help_spans(&help_info, area);
    let help_paragraph = Paragraph::new(lines).style(Style::default());
    frame.render_widget(help_paragraph, area);
}
