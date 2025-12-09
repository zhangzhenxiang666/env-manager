use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
};
use unicode_width::UnicodeWidthStr;

use crate::tui::{
    app::{App, AppState},
    theme::Theme,
};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let profiles = app.list_component.filtered_profiles();
    let items: Vec<ListItem> = profiles
        .iter()
        .map(|name| {
            let display_text = if app.list_component.is_dirty(name) {
                vec![
                    Span::styled("*", Theme::new().text_highlight()),
                    Span::from(name.as_str()),
                ]
            } else {
                vec![Span::from(name.as_str())]
            };
            ListItem::new(Text::from(Line::from(display_text)))
        })
        .collect();

    let total_items = items.len();
    let unsaved_count = app.list_component.unsaved_count();

    let title = if items.is_empty() {
        Line::from("Profile List (0/0)").left_aligned()
    } else {
        Line::from(format!(
            "Profile List ({}/{})",
            app.list_component.selected_index() + 1,
            total_items
        ))
        .left_aligned()
    };

    let mut list = List::new(items)
        .highlight_style(Theme::new().selection_active())
        .highlight_symbol("> ");

    let mut block = Block::default().borders(Borders::ALL).title_top(title);

    if unsaved_count > 0 {
        block = block.title_top(
            Line::from(format!("Unsaved: {unsaved_count}"))
                .style(Theme::new().text_error())
                .right_aligned(),
        );
    }

    if app.state == AppState::List {
        block = block
            .border_style(Theme::new().block_active())
            .border_type(ratatui::widgets::BorderType::Thick);
    } else {
        block = block.border_style(Theme::new().block_inactive());
    }

    list = list.block(block);

    let mut list_state = ListState::default();
    if !app.list_component.all_profiles().is_empty() {
        list_state.select(Some(app.list_component.selected_index()));
    }

    frame.render_stateful_widget(list, area, &mut list_state);

    // Render Rename Overlay
    if app.state == AppState::Rename {
        let selected = app.list_component.selected_index();
        let offset = list_state.offset();

        // Calculate visual position
        let height = area.height as usize;
        let inner_height = height.saturating_sub(2); // borders

        if selected >= offset && selected < offset + inner_height {
            let visual_index = selected - offset;
            let list_inner_y = area.y + 1; // Top border
            let item_y = list_inner_y + visual_index as u16;

            let input = app.list_component.rename_input();
            // Expand width slightly if possible or keep inside?
            // "area" includes borders of main_left.
            // Let's use full width of main_left minus 2 for borders?
            let width = area.width.saturating_sub(2);

            // Height 3 for border (1 top, 1 content, 1 bottom)
            // Centered on item_y: item_y - 1.
            let overlay_y = item_y.saturating_sub(1);

            let input_area = Rect {
                x: area.x + 1,
                y: overlay_y,
                width,
                height: 3,
            };

            // Render Background Clear (to wipe underlying list item + borders if overlapping)
            frame.render_widget(Clear, input_area);

            // Determine border style (Normal or Error)
            let border_style = if input.is_valid() {
                Theme::new().block_active()
            } else {
                Theme::new().text_error()
            };

            let mut block = Block::default()
                .borders(Borders::ALL)
                .title_top(Line::from("Rename Profile").left_aligned())
                .border_style(border_style);

            if let Some(err) = input.error_message() {
                block = block.title_bottom(
                    Line::from(err)
                        .style(Theme::new().text_error())
                        .right_aligned(),
                );
            }

            frame.render_widget(block.clone(), input_area);

            let inner_area = block.inner(input_area);

            // Render Input Text
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

            let mut style = Theme::new().text_normal();
            if !input.is_valid() {
                style = Theme::new().text_error();
            }

            let paragraph = Paragraph::new(text).style(style).scroll((0, scroll_offset));

            frame.render_widget(paragraph, inner_area);

            // Render Cursor
            frame.set_cursor_position((
                inner_area.x + cursor_display_pos - scroll_offset,
                inner_area.y,
            ));
        }
    }

    // Render Scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(ratatui::symbols::scrollbar::VERTICAL)
        .begin_symbol(None)
        .end_symbol(None);

    let viewport_height = area.height.saturating_sub(2) as usize;
    let mut scrollbar_state = ScrollbarState::new(total_items.saturating_sub(viewport_height) + 1)
        .position(list_state.offset());

    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}
