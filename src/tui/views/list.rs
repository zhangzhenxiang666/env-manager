use crate::GLOBAL_PROFILE_MARK;
use crate::tui::app::{App, AppState, MainRightViewMode};
use crate::tui::theme::Theme;
use crate::tui::utils::{Input, inner};
use crate::tui::widgets::empty;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState,
};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct ListView {
    profile_names: Vec<String>,
    selected_index: usize,
    dirty_profiles: HashSet<String>,
    rename_input: Input,
    in_search_mode: bool,
    search_input: Input,
}

impl ListView {
    pub fn new() -> Self {
        Default::default()
    }

    /// Get the currently selected profile name
    pub fn current_profile(&self) -> Option<&str> {
        self.filtered_profiles()
            .get(self.selected_index)
            .map(|s| s.as_str())
    }

    /// Get all profile names (unfiltered)
    pub fn all_profiles(&self) -> &[String] {
        &self.profile_names
    }

    /// Get filtered profiles based on search mode
    pub fn filtered_profiles(&self) -> Vec<&String> {
        if !self.in_search_mode || self.search_input.text().is_empty() {
            return self.profile_names.iter().collect();
        }

        let search_query = self.search_input.text().to_lowercase();
        self.profile_names
            .iter()
            .filter(|name| name.to_lowercase().contains(&search_query))
            .collect()
    }
    /// Update the profile list (e.g., after adding/removing profiles)
    pub fn update_profiles(&mut self, mut profiles: Vec<String>) {
        profiles.sort_by(|a, b| {
            if a == GLOBAL_PROFILE_MARK {
                std::cmp::Ordering::Less
            } else if b == GLOBAL_PROFILE_MARK {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });
        self.profile_names = profiles;
        // Ensure selected_index is valid
        if self.selected_index >= self.profile_names.len() && !self.profile_names.is_empty() {
            self.selected_index = self.profile_names.len() - 1;
        } else if self.profile_names.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Get current selected index (for rendering)
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Set selected index directly (for after operations that change list)
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.profile_names.len() {
            self.selected_index = index;
        }
    }

    pub fn next(&mut self) {
        let filtered = self.filtered_profiles();
        if filtered.is_empty() {
            self.selected_index = 0;
            return;
        }
        let i = (self.selected_index + 1) % filtered.len();
        self.selected_index = i;
    }

    pub fn previous(&mut self) {
        let filtered = self.filtered_profiles();
        if filtered.is_empty() {
            self.selected_index = 0;
            return;
        }
        let i = (self.selected_index + filtered.len() - 1) % filtered.len();
        self.selected_index = i;
    }

    /// Check if a specific profile has unsaved changes
    pub fn is_dirty(&self, name: &str) -> bool {
        self.dirty_profiles.contains(name)
    }

    /// Get count of profiles with unsaved changes
    pub fn unsaved_count(&self) -> usize {
        self.dirty_profiles.len()
    }

    /// Mark a profile as having unsaved changes
    pub fn mark_dirty(&mut self, name: String) {
        self.dirty_profiles.insert(name);
    }

    /// Clear dirty flag for a profile (after saving)
    pub fn clear_dirty(&mut self, name: &str) {
        self.dirty_profiles.remove(name);
    }

    /// Get iterator over all dirty profile names
    pub fn dirty_profiles_iter(&self) -> impl Iterator<Item = &String> {
        self.dirty_profiles.iter()
    }

    pub fn is_searching(&self) -> bool {
        self.in_search_mode
    }

    pub fn enter_search_mode(&mut self) {
        self.in_search_mode = true;
        self.search_input.reset();
        self.selected_index = 0;
    }

    pub fn exit_search_mode(&mut self) {
        if !self.in_search_mode {
            return;
        }
        let filtered = self.filtered_profiles();
        if !filtered.is_empty() {
            let selected_name = filtered[self.selected_index];
            if let Some(index) = self
                .profile_names
                .iter()
                .position(|name| name == selected_name)
            {
                self.selected_index = index;
            }
        }
        self.in_search_mode = false;
        self.search_input.reset();
    }

    /// Get mutable reference to search input for event handlers
    pub fn search_input_mut(&mut self) -> &mut Input {
        &mut self.search_input
    }

    /// Get reference to search input for rendering
    pub fn search_input(&self) -> &Input {
        &self.search_input
    }

    pub fn start_rename(&mut self) {
        if let Some(current_name) = self.current_profile() {
            let name = current_name.to_string();
            self.rename_input.set_text(name.clone());
            self.rename_input.set_cursor_position(name.len());
            self.rename_input.clear_error();
        }
    }

    /// Get mutable reference to rename input for event handlers
    pub fn rename_input_mut(&mut self) -> &mut Input {
        &mut self.rename_input
    }

    /// Get reference to rename input for rendering
    pub fn rename_input(&self) -> &Input {
        &self.rename_input
    }

    /// Reset rename input
    pub fn reset_rename(&mut self) {
        self.rename_input.reset();
    }
}

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let theme = Theme::new();
    let profiles = app.list_view.filtered_profiles();
    let items: Vec<ListItem> = profiles
        .iter()
        .map(|name| {
            let display_name = if *name == GLOBAL_PROFILE_MARK {
                "GLOBAL"
            } else {
                name.as_str()
            };
            let display_text = if app.list_view.is_dirty(name) {
                vec![
                    Span::styled("*", theme.text_highlight()),
                    Span::from(display_name),
                ]
            } else {
                vec![Span::from(display_name)]
            };
            ListItem::new(Text::from(Line::from(display_text)))
        })
        .collect();

    let total_items = items.len();
    let is_empty = total_items == 0;
    let unsaved_count = app.list_view.unsaved_count();

    let title = if is_empty {
        Line::from("Profile List (0/0)").left_aligned()
    } else {
        Line::from(format!(
            "Profile List ({}/{})",
            app.list_view.selected_index() + 1,
            total_items
        ))
        .left_aligned()
    };

    let mut list = List::new(items)
        .highlight_style(theme.selection_active())
        .highlight_symbol("> ");

    let mut block = Block::default().borders(Borders::ALL).title_top(title);

    if unsaved_count > 0 {
        block = block.title_top(
            Line::from(format!("Unsaved: {unsaved_count}"))
                .style(theme.text_error())
                .right_aligned(),
        );
    }

    if app.state == AppState::List {
        block = block
            .border_style(theme.block_active())
            .border_type(ratatui::widgets::BorderType::Thick);
    } else {
        block = block.border_style(theme.block_inactive());
    }

    list = list.block(block);

    let mut list_state = ListState::default();
    if !app.list_view.filtered_profiles().is_empty() {
        list_state.select(Some(app.list_view.selected_index()));
    }

    if is_empty {
        empty::render(
            frame,
            inner(area),
            Text::from(vec![
                Line::styled("No profiles match", Style::default().dim()).centered(),
                Line::styled("your search criteria", Style::default().dim()).centered(),
            ]),
            2,
        );
    }

    frame.render_stateful_widget(list, area, &mut list_state);

    // Render Rename Overlay
    if app.state == AppState::Rename {
        render_rename_section(frame, &app.list_view, area, &list_state, &theme);
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
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_rename_section(
    frame: &mut Frame<'_>,
    list_view: &ListView,
    area: Rect,
    list_state: &ListState,
    theme: &Theme,
) {
    let selected = list_view.selected_index();
    let offset = list_state.offset();

    // Calculate visual position
    let height = area.height as usize;
    let inner_height = height.saturating_sub(2); // borders

    if selected >= offset && selected < offset + inner_height {
        let visual_index = selected - offset;
        let list_inner_y = area.y + 1; // Top border
        let item_y = list_inner_y + visual_index as u16;

        let input = list_view.rename_input();

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
            theme.block_active()
        } else {
            theme.text_error()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .title_top(Line::from("Rename Profile").left_aligned())
            .border_style(border_style);

        if let Some(err) = input.error_message() {
            block = block.title_bottom(Line::from(err).style(theme.text_error()).right_aligned());
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

        let mut style = theme.text_normal();
        if !input.is_valid() {
            style = theme.text_error();
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

pub fn handle_event(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    let list_view = &mut app.list_view;

    if list_view.is_searching() {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('d') => {
                    if let Some(name) = list_view.current_profile() {
                        if name == GLOBAL_PROFILE_MARK {
                            app.status_message = Some("Cannot delete GLOBAL profile".to_string());
                        } else {
                            app.state = AppState::ConfirmDelete;
                        }
                    }
                }
                KeyCode::Char('s') => {
                    app.save_selected()?;
                }
                KeyCode::Char('w') => {
                    app.save_all()?;
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                list_view.exit_search_mode();
            }
            KeyCode::Char(c) => {
                list_view.search_input_mut().enter_char(c);
                list_view.set_selected_index(0);
            }
            KeyCode::Backspace => {
                list_view.search_input_mut().delete_char();
                list_view.set_selected_index(0);
            }
            KeyCode::Left => {
                list_view.search_input_mut().move_cursor_left();
            }
            KeyCode::Right => {
                list_view.search_input_mut().move_cursor_right();
            }
            KeyCode::Down => {
                list_view.next();
                if app.main_right_view_mode == MainRightViewMode::Expand {
                    app.load_expand_vars();
                }
            }
            KeyCode::Up => {
                list_view.previous();
                if app.main_right_view_mode == MainRightViewMode::Expand {
                    app.load_expand_vars();
                }
            }
            KeyCode::Enter => {
                if let Some(name) = list_view.current_profile() {
                    let name = name.to_string();
                    app.start_editing(&name);
                }
            }
            KeyCode::Tab => match app.main_right_view_mode {
                MainRightViewMode::Raw => {
                    app.load_expand_vars();
                }
                MainRightViewMode::Expand => {
                    app.unload_expand_vars();
                }
            },
            KeyCode::F(2) => {
                if let Some(name) = list_view.current_profile() {
                    if name == GLOBAL_PROFILE_MARK {
                        app.status_message = Some("Cannot rename GLOBAL profile".to_string());
                    } else {
                        app.state = AppState::Rename;
                        list_view.start_rename();
                    }
                }
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Esc => {
                if app.list_view.unsaved_count() > 0 {
                    app.state = AppState::ConfirmExit;
                } else {
                    app.shutdown = true;
                }
            }
            KeyCode::Char('/') => {
                list_view.enter_search_mode();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.list_view.next();
                if app.main_right_view_mode == MainRightViewMode::Expand {
                    app.load_expand_vars();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.list_view.previous();
                if app.main_right_view_mode == MainRightViewMode::Expand {
                    app.load_expand_vars();
                }
            }
            KeyCode::Enter => {
                if let Some(name) = list_view.current_profile() {
                    let name = name.to_string();
                    app.start_editing(&name);
                }
            }
            KeyCode::Tab => match app.main_right_view_mode {
                MainRightViewMode::Raw => {
                    app.load_expand_vars();
                }
                MainRightViewMode::Expand => {
                    app.unload_expand_vars();
                }
            },
            KeyCode::Char('s') => {
                app.save_selected()?;
            }
            KeyCode::Char('w') => {
                app.save_all()?;
            }
            KeyCode::Char('d') => {
                if let Some(name) = list_view.current_profile() {
                    if name == GLOBAL_PROFILE_MARK {
                        app.status_message = Some("Cannot delete GLOBAL profile".to_string());
                    } else {
                        app.state = AppState::ConfirmDelete;
                    }
                }
            }
            KeyCode::Char('n') => {
                app.state = AppState::AddNew;
                app.add_new_view.reset();
            }
            KeyCode::F(2) => {
                if let Some(name) = list_view.current_profile() {
                    if name == GLOBAL_PROFILE_MARK {
                        app.status_message = Some("Cannot rename GLOBAL profile".to_string());
                    } else {
                        app.state = AppState::Rename;
                        list_view.start_rename();
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn handle_rename_event(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char(c) => {
            app.list_view.rename_input_mut().enter_char(c);
            validate_rename_name(app);
        }
        KeyCode::Backspace => {
            app.list_view.rename_input_mut().delete_char();
            validate_rename_name(app);
        }
        KeyCode::Left => {
            app.list_view.rename_input_mut().move_cursor_left();
        }
        KeyCode::Right => {
            app.list_view.rename_input_mut().move_cursor_right();
        }
        KeyCode::Esc => {
            app.list_view.reset_rename();
            app.state = AppState::List;
        }
        KeyCode::Enter => {
            if app.list_view.rename_input_mut().is_valid() {
                let new_name = app.list_view.rename_input().text().to_string();
                app.rename_profile(new_name)?;
                app.list_view.reset_rename();
                app.state = AppState::List;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_rename_name(app: &mut App) {
    app.list_view.rename_input_mut().clear_error();

    if let Some(name) = app.list_view.current_profile()
        && name != app.list_view.rename_input().text()
        && app
            .config_manager
            .has_profile(app.list_view.rename_input().text())
    {
        app.list_view
            .rename_input_mut()
            .set_error_message("Profile name already exists");
        return;
    }
    crate::tui::utils::validate_input(app.list_view.rename_input_mut());
}
