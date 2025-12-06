use crate::tui::components::add_new::{
    AddNewComponent, AddNewFocus, AddNewVariableFocus, MAX_HEIGHT, MAX_VARIABLES_HEIGHT,
};
use crate::tui::{app::App, theme::Theme, utils, utils::Input};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
};
use std::mem;
use unicode_width::UnicodeWidthStr;

const MAX_HELP_LINES: usize = 2;

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = utils::centered_rect(70, 80, frame.area());
    frame.render_widget(Clear, area);

    let theme = Theme::new();

    let popup_border_style = theme.block_active();

    let popup_block = Block::default()
        .title("Create New Profile")
        .borders(Borders::ALL)
        .border_style(popup_border_style)
        .border_type(ratatui::widgets::BorderType::Thick);

    let inner_popup_area = popup_block.inner(area);
    frame.render_widget(popup_block, area);

    let chunks = Layout::vertical([
        Constraint::Length(4),  // Name section
        Constraint::Length(5),  // Profiles section
        Constraint::Length(12), // Variables section (8 items + 2 header + 2 border)
        Constraint::Min(0),     // Spacer
        Constraint::Length(2),  // Help section
    ])
    .split(inner_popup_area);

    let name_area = chunks[0];
    let profiles_area = chunks[1];
    let variables_area = chunks[2];
    // chunks[3] is spacer
    let help_area = chunks[4];

    name(frame, &app.add_new_component, name_area);
    profiles(frame, app, profiles_area);
    variables(frame, app, variables_area);

    match app.add_new_component.focus {
        AddNewFocus::Name => name_help(frame, help_area),
        AddNewFocus::Profiles => profiles_help(frame, help_area),
        AddNewFocus::Variables => variables_help(frame, app, help_area),
    }
}

fn name(frame: &mut Frame<'_>, add_new: &AddNewComponent, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // For the input box
        Constraint::Length(1), // For the validation message
    ])
    .split(area);

    let input_area = chunks[0];
    let validation_msg_area = chunks[1];

    let mut input_block = Block::default().title("Name").borders(Borders::ALL);
    if add_new.focus == AddNewFocus::Name {
        input_block = input_block.border_style(Theme::new().block_active());
    } else {
        input_block = input_block.border_style(Theme::new().block_inactive());
    }
    let text_input_rect = input_block.inner(input_area);
    frame.render_widget(input_block, input_area);

    let input_text = &add_new.name_input.text;
    let cursor_char_pos = add_new.name_input.cursor_position;
    let prefix = &input_text[..input_text
        .char_indices()
        .nth(cursor_char_pos)
        .map_or(input_text.len(), |(i, _)| i)];
    let cursor_display_pos = UnicodeWidthStr::width(prefix) as u16;
    let input_display_width = text_input_rect.width;
    let scroll_offset = if cursor_display_pos >= input_display_width {
        cursor_display_pos - input_display_width + 1
    } else {
        0
    };

    let input_paragraph = Paragraph::new(input_text.as_str())
        .style(Theme::new().text_normal())
        .scroll((0, scroll_offset));
    frame.render_widget(input_paragraph, text_input_rect);

    if add_new.focus == AddNewFocus::Name {
        frame.set_cursor_position((
            text_input_rect.x + cursor_display_pos - scroll_offset,
            text_input_rect.y,
        ));
    }

    let validation_message = if !add_new.name_input.is_valid {
        add_new
            .name_input
            .error_message
            .as_deref()
            .unwrap_or("Error: Unknown")
    } else {
        ""
    };
    let validation_paragraph = Paragraph::new(validation_message).style(Theme::new().text_error());
    frame.render_widget(validation_paragraph, validation_msg_area);
}

fn profiles(frame: &mut Frame, app: &App, area: Rect) {
    let add_new = &app.add_new_component;
    let available_profiles: Vec<_> = app
        .list_component
        .profile_names
        .iter()
        .filter(|name| **name != add_new.name_input.text)
        .collect();
    let total_profiles = available_profiles.len();
    let is_focused = add_new.focus == AddNewFocus::Profiles;

    let left_title = Line::from(format!(
        "Inherit Profiles ({}/{})",
        add_new.profiles_selection_index.saturating_add(1),
        total_profiles,
    ))
    .left_aligned();
    let right_title =
        Line::from(format!("Selected: {}", add_new.added_profiles.len())).right_aligned();
    let mut profiles_block = Block::default()
        .title_top(left_title)
        .title_top(right_title)
        .borders(Borders::ALL);

    if is_focused {
        profiles_block = profiles_block.border_style(Theme::new().block_active());
    } else {
        profiles_block = profiles_block.border_style(Theme::new().block_inactive());
    }

    let list_items: Vec<ListItem> = available_profiles
        .iter()
        .skip(add_new.profiles_scroll_offset)
        .take(MAX_HEIGHT)
        .map(|name| {
            let prefix = if add_new.added_profiles.contains(*name) {
                "[✔] "
            } else {
                "[ ] "
            };
            ListItem::new(format!("{prefix}{name}"))
        })
        .collect();

    let mut results_list = List::new(list_items).block(profiles_block);
    if is_focused {
        results_list = results_list.highlight_style(Theme::new().selection_active());
    }

    let mut list_state = ListState::default();
    if is_focused && !available_profiles.is_empty() {
        list_state.select(Some(
            add_new.profiles_selection_index - add_new.profiles_scroll_offset,
        ));
    }
    frame.render_stateful_widget(results_list, area, &mut list_state);

    // --- Scrollbar ---
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    let mut scrollbar_state = ScrollbarState::new(total_profiles)
        .viewport_content_length(MAX_HEIGHT)
        .position(add_new.profiles_scroll_offset);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn variables(frame: &mut Frame, app: &App, area: Rect) {
    let add_new = &app.add_new_component;
    let is_focused = add_new.focus == AddNewFocus::Variables;

    let current_index = if add_new.variables.is_empty() {
        0
    } else {
        add_new.selected_variable_index + 1
    };
    let total_count = add_new.variables.len();

    let left_title =
        Line::from(format!("Variables ({}/{})", current_index, total_count)).left_aligned();

    let mut variables_block = Block::default().title_top(left_title).borders(Borders::ALL);
    if is_focused && !add_new.is_editing_variable {
        variables_block = variables_block.border_style(Theme::new().block_active());
    } else {
        variables_block = variables_block.border_style(Theme::new().block_inactive());
    }

    let header = Row::new(vec!["Key", "Value"])
        .style(Style::new().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = add_new
        .variables
        .iter()
        .enumerate()
        .map(|(i, (key_input, value_input))| {
            let is_selected = is_focused && i == add_new.selected_variable_index;

            let (key_style, value_style) = if is_selected {
                match add_new.focused_column {
                    AddNewVariableFocus::Key => (
                        Theme::new().cell_focus(),       // Focused
                        Theme::new().selection_active(), // Unfocused but selected row
                    ),
                    AddNewVariableFocus::Value => (
                        Theme::new().selection_active(), // Unfocused but selected row
                        Theme::new().cell_focus(),       // Focused
                    ),
                }
            } else {
                (Theme::new().text_normal(), Theme::new().text_normal())
            };
            Row::new(vec![
                Cell::from(key_input.text.as_str()).style(key_style),
                Cell::from(value_input.text.as_str()).style(value_style),
            ])
        })
        .collect();

    let mut table_state = TableState::default().with_offset(add_new.variables_scroll_offset);
    if is_focused && !add_new.variables.is_empty() {
        table_state.select(Some(add_new.selected_variable_index));
    }

    let widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
    let table = Table::new(rows, widths)
        .header(header)
        .block(variables_block.clone());

    frame.render_stateful_widget(table, area, &mut table_state);

    // --- Scrollbar ---
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    let mut scrollbar_state = ScrollbarState::new(add_new.variables.len())
        .viewport_content_length(MAX_VARIABLES_HEIGHT)
        .position(add_new.variables_scroll_offset);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    // --- Popup Input Box Rendering ---
    if is_focused && add_new.is_editing_variable {
        if let Some(focused_input) = add_new.get_focused_variable_input() {
            let table_inner_area = variables_block.inner(area);
            let row_index = add_new.selected_variable_index;
            // Calculate visual row index by subtracting scroll offset
            let visual_row_index = row_index.saturating_sub(add_new.variables_scroll_offset);
            let row_y = table_inner_area.y + 2 + visual_row_index as u16;

            let col_index = match add_new.focused_column {
                AddNewVariableFocus::Key => 0,
                AddNewVariableFocus::Value => 1,
            };

            let layout = Layout::horizontal(widths).spacing(1);
            let column_chunks = layout.split(table_inner_area);
            let cell_area = column_chunks[col_index];

            let popup_area = Rect {
                x: cell_area.x.saturating_sub(1),
                y: row_y.saturating_sub(1),
                width: cell_area.width + 2,
                height: 3,
            };

            let title = match add_new.focused_column {
                AddNewVariableFocus::Key => "Edit Variable",
                AddNewVariableFocus::Value => "Edit Value",
            };
            render_input_popup(frame, popup_area, focused_input, title);
        }
    }
}

fn render_input_popup(frame: &mut Frame, area: Rect, input: &Input, title: &str) {
    frame.render_widget(Clear, area);
    let mut block = Block::default().title(title).borders(Borders::ALL);

    if input.is_valid {
        block = block.border_style(Theme::new().block_active());
    } else {
        block = block.border_style(Theme::new().text_error());
        if let Some(err) = &input.error_message {
            block = block.title_bottom(
                Line::from(err.as_str())
                    .style(Theme::new().text_error())
                    .right_aligned(),
            );
        }
    }
    let inner_area = block.inner(area);

    let text = &input.text;
    let cursor_pos = input.cursor_position;
    let prefix = &text[..text
        .char_indices()
        .nth(cursor_pos)
        .map_or(text.len(), |(i, _)| i)];
    let cursor_display_pos = UnicodeWidthStr::width(prefix) as u16;
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

fn generic_help_spans<'a>(help_info: &'a [Vec<Span<'a>>], area: Rect) -> Vec<Line<'a>> {
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

fn name_help(frame: &mut Frame<'_>, area: Rect) {
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
            Span::styled("←/→", Style::default().fg(Color::Rgb(255, 138, 199))),
            Span::raw(": Move cursor"),
        ],
        vec![
            Span::styled("Ctrl+s", Style::default().fg(Color::Rgb(106, 255, 160))),
            Span::raw(": Save"),
        ],
    ];
    let lines = generic_help_spans(&help_info, area);
    let help_paragraph = Paragraph::new(lines).style(Style::default());
    frame.render_widget(help_paragraph, area);
}

fn profiles_help(frame: &mut Frame, area: Rect) {
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
            Span::styled("↑/↓", Style::default().fg(Color::Rgb(255, 138, 199))),
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
    let lines = generic_help_spans(&help_info, area);
    let help_paragraph = Paragraph::new(lines).style(Style::default());
    frame.render_widget(help_paragraph, area);
}

fn variables_help(frame: &mut Frame, app: &App, area: Rect) {
    let add_new = &app.add_new_component;
    let help_info = if add_new.is_editing_variable {
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
    let lines = generic_help_spans(&help_info, area);
    let help_paragraph = Paragraph::new(lines).style(Style::default());
    frame.render_widget(help_paragraph, area);
}
