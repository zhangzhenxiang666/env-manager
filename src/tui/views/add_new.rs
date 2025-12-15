use crate::GLOBAL_PROFILE_MARK;
use crate::config::models::Profile;
use crate::tui::app::{App, AppState};
use crate::tui::widgets::empty;
use crate::tui::{theme::Theme, utils, utils::Input, utils::validate_input};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Scrollbar,
    ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use std::collections::{HashMap, HashSet};
use std::mem;
use unicode_width::UnicodeWidthStr;

const MAX_HELP_LINES: usize = 2;

// ==================================================================================
// STATE
// ==================================================================================

pub const MAX_HEIGHT: usize = 4;
pub const MAX_VARIABLES_HEIGHT: usize = 8;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AddNewFocus {
    #[default]
    Name,
    Profiles,
    Variables,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AddNewVariableFocus {
    #[default]
    Key,
    Value,
}

#[derive(Default)]
pub struct AddNewView {
    pub name_input: Input,

    // Profiles section
    pub profiles_selection_index: usize,
    pub added_profiles: HashSet<String>,
    pub profile_scroll_offset: usize,

    // Variables section
    pub variables: Vec<(Input, Input)>,
    pub selected_variable_index: usize,
    pub variable_scroll_offset: usize,
    pub variable_column_focus: AddNewVariableFocus,
    pub is_editing_variable: bool,
    pub pre_edit_buffer: Option<String>,

    // Focus management
    pub focus: AddNewFocus,
}

impl AddNewView {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.name_input = Input::default();
        self.profiles_selection_index = 0;
        self.added_profiles.clear();
        self.profile_scroll_offset = 0;
        self.variables.clear();
        self.selected_variable_index = 0;
        self.variable_scroll_offset = 0;
        self.variable_column_focus = AddNewVariableFocus::default();
        self.is_editing_variable = false;
        self.pre_edit_buffer = None;
        self.focus = AddNewFocus::default();
    }

    pub fn current_focus(&self) -> AddNewFocus {
        self.focus
    }

    pub fn is_editing(&self) -> bool {
        self.is_editing_variable
    }

    pub fn variable_column_focus(&self) -> AddNewVariableFocus {
        self.variable_column_focus
    }

    pub fn name_input(&self) -> &Input {
        &self.name_input
    }

    pub fn name_input_mut(&mut self) -> &mut Input {
        &mut self.name_input
    }

    pub fn switch_focus(&mut self, forward: bool) {
        self.focus = if forward {
            match self.focus {
                AddNewFocus::Name => AddNewFocus::Profiles,
                AddNewFocus::Profiles => AddNewFocus::Variables,
                AddNewFocus::Variables => AddNewFocus::Name,
            }
        } else {
            match self.focus {
                AddNewFocus::Name => AddNewFocus::Variables,
                AddNewFocus::Variables => AddNewFocus::Profiles,
                AddNewFocus::Profiles => AddNewFocus::Name,
            }
        };
    }

    pub fn profiles_selection_index(&self) -> usize {
        self.profiles_selection_index
    }

    pub fn profile_scroll_offset(&self) -> usize {
        self.profile_scroll_offset
    }

    pub fn added_profiles(&self) -> &HashSet<String> {
        &self.added_profiles
    }

    pub fn is_profile_added(&self, name: &str) -> bool {
        self.added_profiles.contains(name)
    }

    pub fn select_next_profile(&mut self, profiles_count: usize) {
        if profiles_count == 0 {
            return;
        }
        if self.profiles_selection_index < profiles_count - 1 {
            self.profiles_selection_index += 1;
            self.ensure_profile_visible();
        } else {
            self.profiles_selection_index = 0;
            self.ensure_profile_visible();
        }
    }

    pub fn select_previous_profile(&mut self, profiles_count: usize) {
        if profiles_count == 0 {
            return;
        }
        if self.profiles_selection_index > 0 {
            self.profiles_selection_index -= 1;
            self.ensure_profile_visible();
        } else {
            self.profiles_selection_index = profiles_count - 1;
            self.ensure_profile_visible();
        }
    }

    pub fn toggle_current_profile(&mut self, profile_name: String) {
        if self.added_profiles.contains(&profile_name) {
            self.added_profiles.remove(&profile_name);
        } else {
            self.added_profiles.insert(profile_name);
        }
    }

    fn ensure_profile_visible(&mut self) {
        // Simple scrolling logic: ensure selected item is visible
        // If selected is before scroll offset, scroll up
        if self.profiles_selection_index < self.profile_scroll_offset {
            self.profile_scroll_offset = self.profiles_selection_index;
        }
        // Downward scrolling will be handled during rendering
    }

    /// Calculate the adjusted scroll offset for profiles given the actual viewport height
    pub fn calculate_profile_scroll_offset(&self, visible_rows: usize) -> usize {
        let visible_rows = visible_rows.max(1);
        let mut scroll_offset = self.profile_scroll_offset;

        // If selected is beyond the visible area, adjust scroll offset
        if self.profiles_selection_index >= scroll_offset + visible_rows {
            scroll_offset = self.profiles_selection_index + 1 - visible_rows;
        }
        // If selected is before scroll offset, scroll up
        if self.profiles_selection_index < scroll_offset {
            scroll_offset = self.profiles_selection_index;
        }

        scroll_offset
    }

    pub fn variables_count(&self) -> usize {
        self.variables.len()
    }

    pub fn selected_variable_index(&self) -> usize {
        self.selected_variable_index
    }

    pub fn variable_scroll_offset(&self) -> usize {
        self.variable_scroll_offset
    }

    /// Get all variables as Input pairs for rendering
    pub fn variables_for_rendering(&self) -> &[(Input, Input)] {
        &self.variables
    }

    pub fn add_new_variable(&mut self) {
        self.variables.push((Input::default(), Input::default()));
        self.selected_variable_index = self.variables.len() - 1;
        self.ensure_variable_visible();

        // Auto-start editing on Key column
        self.variable_column_focus = AddNewVariableFocus::Key;
        self.start_editing_variable();

        // Switch focus to Variables if not already
        if self.focus != AddNewFocus::Variables {
            self.focus = AddNewFocus::Variables;
        }
    }

    pub fn delete_selected_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }

        if self.selected_variable_index < self.variables.len() {
            self.variables.remove(self.selected_variable_index);

            if self.variables.is_empty() {
                self.selected_variable_index = 0;
                self.variable_scroll_offset = 0;
                self.is_editing_variable = false;
                self.pre_edit_buffer = None;
            } else {
                if self.selected_variable_index >= self.variables.len() {
                    self.selected_variable_index = self.variables.len() - 1;
                }
                self.ensure_variable_visible();
            }
        }
    }

    pub fn select_next_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }
        if self.selected_variable_index < self.variables.len() - 1 {
            self.selected_variable_index += 1;
            self.ensure_variable_visible();
        } else {
            self.selected_variable_index = 0;
            self.ensure_variable_visible();
        }
    }

    pub fn select_previous_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }
        if self.selected_variable_index > 0 {
            self.selected_variable_index -= 1;
            self.ensure_variable_visible();
        } else {
            self.selected_variable_index = self.variables.len() - 1;
            self.ensure_variable_visible();
        }
    }

    pub fn switch_variable_column(&mut self) {
        self.variable_column_focus = match self.variable_column_focus {
            AddNewVariableFocus::Key => AddNewVariableFocus::Value,
            AddNewVariableFocus::Value => AddNewVariableFocus::Key,
        };
    }

    pub fn start_editing_variable(&mut self) {
        if self.variables.is_empty() {
            return;
        }

        self.is_editing_variable = true;
        let (k, v) = &self.variables[self.selected_variable_index];
        self.pre_edit_buffer = Some(match self.variable_column_focus {
            AddNewVariableFocus::Key => k.text().to_string(),
            AddNewVariableFocus::Value => v.text().to_string(),
        });
    }

    pub fn confirm_editing_variable(&mut self) {
        self.is_editing_variable = false;
        self.pre_edit_buffer = None;
    }

    pub fn cancel_editing_variable(&mut self) {
        if self.is_editing_variable {
            if let Some(buf) = self.pre_edit_buffer.take()
                && let Some(input) = self.get_focused_variable_input_mut()
            {
                input.set_text(buf);
            }
            self.is_editing_variable = false;
        }
    }

    pub fn get_focused_variable_input_mut(&mut self) -> Option<&mut Input> {
        if self.selected_variable_index < self.variables.len() {
            let (k, v) = &mut self.variables[self.selected_variable_index];
            match self.variable_column_focus {
                AddNewVariableFocus::Key => Some(k),
                AddNewVariableFocus::Value => Some(v),
            }
        } else {
            None
        }
    }

    pub fn get_focused_variable_input(&self) -> Option<&Input> {
        if self.selected_variable_index < self.variables.len() {
            let (k, v) = &self.variables[self.selected_variable_index];
            match self.variable_column_focus {
                AddNewVariableFocus::Key => Some(k),
                AddNewVariableFocus::Value => Some(v),
            }
        } else {
            None
        }
    }

    /// Check if the variable at index is valid (for deletion logic)
    pub fn is_variable_valid(&self, index: usize) -> bool {
        if let Some((key_input, _)) = self.variables.get(index) {
            !key_input.text().is_empty()
                && !key_input.text().chars().any(char::is_whitespace)
                && !key_input
                    .text()
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())
        } else {
            false
        }
    }

    fn ensure_variable_visible(&mut self) {
        // Simple scrolling logic: ensure selected item is visible
        // If selected is before scroll offset, scroll up
        if self.selected_variable_index < self.variable_scroll_offset {
            self.variable_scroll_offset = self.selected_variable_index;
        }
        // Downward scrolling will be handled during rendering
    }

    /// Calculate the adjusted scroll offset for variables given the actual viewport height
    pub fn calculate_variable_scroll_offset(&self, visible_rows: usize) -> usize {
        let visible_rows = visible_rows.max(1);
        let mut scroll_offset = self.variable_scroll_offset;

        // If selected is beyond the visible area, adjust scroll offset
        if self.selected_variable_index >= scroll_offset + visible_rows {
            scroll_offset = self.selected_variable_index + 1 - visible_rows;
        }
        // If selected is before scroll offset, scroll up
        if self.selected_variable_index < scroll_offset {
            scroll_offset = self.selected_variable_index;
        }

        scroll_offset
    }
}

// ==================================================================================
// EVENT HANDLING
// ==================================================================================

pub fn handle_event(app: &mut App, key: KeyEvent) {
    if app.add_new_view.is_editing() {
        handle_editing_mode(app, key);
    } else {
        handle_navigation_mode(app, key);
    }
}

fn handle_editing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => handle_editing_enter(app),
        KeyCode::Tab => handle_editing_tab(app),
        KeyCode::BackTab => handle_editing_tab(app),
        KeyCode::Esc => handle_editing_esc(app),
        _ => handle_editing_input(app, key.code),
    }
}

fn handle_editing_enter(app: &mut App) {
    let add_new = &mut app.add_new_view;

    // Validate before confirming if editing Key
    if add_new.variable_column_focus() == AddNewVariableFocus::Key
        && !validate_variable_key_input(add_new)
    {
        return;
    }

    add_new.confirm_editing_variable();

    if add_new.variable_column_focus() == AddNewVariableFocus::Key {
        add_new.switch_variable_column();
        add_new.start_editing_variable();
    }
}

fn handle_editing_tab(app: &mut App) {
    let add_new = &mut app.add_new_view;

    // Validate before switching if currently on Key
    if add_new.variable_column_focus() == AddNewVariableFocus::Key
        && !validate_variable_key_input(add_new)
    {
        return;
    }

    add_new.confirm_editing_variable();
    add_new.switch_variable_column();
    add_new.start_editing_variable();
}

fn handle_editing_esc(app: &mut App) {
    let add_new = &mut app.add_new_view;
    add_new.cancel_editing_variable();

    // Check if the current row is invalid (empty, spaces, etc.) and delete if so
    if should_delete_variable_row(add_new) {
        add_new.delete_selected_variable();
    }
}

fn handle_editing_input(app: &mut App, key_code: KeyCode) {
    let add_new = &mut app.add_new_view;
    match key_code {
        KeyCode::Char(c) => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.enter_char(c);

                if add_new.variable_column_focus() == AddNewVariableFocus::Key {
                    validate_variable_key_input(add_new);
                }
            }
        }
        KeyCode::Backspace => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.delete_char();

                if add_new.variable_column_focus() == AddNewVariableFocus::Key {
                    validate_variable_key_input(add_new);
                }
            }
        }
        KeyCode::Left => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.move_cursor_left();
            }
        }
        KeyCode::Right => {
            if let Some(input) = add_new.get_focused_variable_input_mut() {
                input.move_cursor_right();
            }
        }
        // For any other key, confirm the current edit
        _ => {
            if validate_variable_key_input(add_new) {
                add_new.confirm_editing_variable();
            }
        }
    }
}

fn handle_navigation_mode(app: &mut App, key: KeyEvent) {
    match key {
        // Save
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => save_profile(app),

        // Close / Cancel
        KeyEvent {
            code: KeyCode::Esc, ..
        } => close_popup(app),

        // Navigation
        KeyEvent {
            code: KeyCode::Tab, ..
        } => attempt_switch_focus(app, true),

        KeyEvent {
            code: KeyCode::BackTab,
            ..
        } => attempt_switch_focus(app, false),

        // Context Specific
        _ => dispatch_context_key(app, key),
    }
}

fn save_profile(app: &mut App) {
    if !validate_name(app) {
        return;
    }

    let add_new = &mut app.add_new_view;
    let new_name = add_new.name_input().text().trim().to_string();

    let variables_map: HashMap<String, String> = add_new
        .variables_for_rendering()
        .iter()
        .map(|(k, v)| (k.text().to_string(), v.text().to_string()))
        .filter(|(k, _)| !k.is_empty())
        .collect();

    let new_profile = Profile {
        profiles: add_new.added_profiles().iter().cloned().collect(),
        variables: variables_map,
    };

    // 1. Add profile to memory
    app.config_manager
        .add_profile(new_name.clone(), new_profile.clone());
    app.list_view.mark_dirty(new_name.clone());

    // 2. Add node to graph
    app.config_manager.add_profile_node(new_name.clone());

    // 3. Add dependency edges to graph
    for dep_name in &new_profile.profiles {
        if let Err(e) = app.config_manager.add_dependency_edge(&new_name, dep_name) {
            app.status_message = Some(format!(
                "Warning: Failed to add dependency edge to '{dep_name}': {e}"
            ));
        }
    }

    // 4. Update UI list
    let mut profiles = app.list_view.all_profiles().to_vec();
    profiles.push(new_name.clone());
    profiles.sort();
    app.list_view.update_profiles(profiles);

    if let Some(index) = app
        .list_view
        .all_profiles()
        .iter()
        .position(|r| r == &new_name)
    {
        app.list_view.set_selected_index(index);
    }

    app.status_message = Some(format!("Profile '{new_name}' created."));
    app.state = AppState::List;
    add_new.reset();
}

fn close_popup(app: &mut App) {
    app.state = AppState::List;
    app.add_new_view.reset();
}

fn attempt_switch_focus(app: &mut App, forward: bool) {
    // If focused on Name, validate before leaving
    if app.add_new_view.current_focus() == AddNewFocus::Name && !validate_name(app) {
        return;
    }
    app.add_new_view.switch_focus(forward);
}

fn dispatch_context_key(app: &mut App, key: KeyEvent) {
    let focus = app.add_new_view.current_focus();

    match key.code {
        KeyCode::Esc => {
            app.add_new_view.reset();
            app.state = AppState::List;
        }
        KeyCode::Char(c) if focus == AddNewFocus::Name => {
            app.add_new_view.name_input_mut().enter_char(c);
            validate_name(app);
        }
        KeyCode::Backspace if focus == AddNewFocus::Name => {
            app.add_new_view.name_input_mut().delete_char();
            validate_name(app);
        }
        KeyCode::Left if focus == AddNewFocus::Name => {
            app.add_new_view.name_input_mut().move_cursor_left()
        }
        KeyCode::Right if focus == AddNewFocus::Name => {
            app.add_new_view.name_input_mut().move_cursor_right()
        }
        KeyCode::Enter if focus == AddNewFocus::Name && validate_name(app) => {
            app.add_new_view.switch_focus(true);
        }
        _ => {
            // Dispatch to specific handlers for Profiles and Variables
            match focus {
                AddNewFocus::Profiles => profiles(app, key.code),
                AddNewFocus::Variables => variables(app, key.code),
                _ => {}
            }
        }
    }
}

fn profiles(app: &mut App, key_code: KeyCode) {
    let add_new = &mut app.add_new_view;
    let available_profiles: Vec<_> = app
        .list_view
        .all_profiles()
        .iter()
        .filter(|name| **name != add_new.name_input().text() && *name != GLOBAL_PROFILE_MARK)
        .collect();
    let count = available_profiles.len();

    match key_code {
        KeyCode::Up | KeyCode::Char('k') => add_new.select_previous_profile(count),
        KeyCode::Down | KeyCode::Char('j') => add_new.select_next_profile(count),
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(selected_name) = available_profiles.get(add_new.profiles_selection_index())
            {
                add_new.toggle_current_profile(selected_name.to_string());
            }
        }
        _ => {}
    }
}

fn variables(app: &mut App, key_code: KeyCode) {
    let add_new = &mut app.add_new_view;
    match key_code {
        KeyCode::Up | KeyCode::Char('k') => add_new.select_previous_variable(),
        KeyCode::Down | KeyCode::Char('j') => add_new.select_next_variable(),
        KeyCode::Left | KeyCode::Char('h') => add_new.switch_variable_column(),
        KeyCode::Right | KeyCode::Char('l') => add_new.switch_variable_column(),
        KeyCode::Char('a') => add_new.add_new_variable(),
        KeyCode::Char('d') => add_new.delete_selected_variable(),
        KeyCode::Char('e') => add_new.start_editing_variable(),
        _ => {}
    }
}

fn validate_name(app: &mut App) -> bool {
    let input = app.add_new_view.name_input_mut();
    input.clear_error();
    if app.config_manager.has_profile(input.text()) {
        input.set_error_message("Profile already exists");
        false
    } else {
        validate_input(input)
    }
}

/// Validates the currently focused variable input (if it's a Key).
/// Returns true if valid, false if invalid.
fn validate_variable_key_input(add_new: &mut AddNewView) -> bool {
    if let Some(input) = add_new.get_focused_variable_input_mut() {
        input.clear_error();
        validate_input(input)
    } else {
        true
    }
}

fn should_delete_variable_row(add_new: &AddNewView) -> bool {
    let idx = add_new.selected_variable_index();
    !add_new.is_variable_valid(idx)
}

// ==================================================================================
// RENDERING
// ==================================================================================

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = utils::centered_rect(70, 80, frame.area());
    frame.render_widget(Clear, area);

    let theme = Theme::new();
    let add_new_state = &app.add_new_view;

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

fn render_name_section(frame: &mut Frame<'_>, add_new: &AddNewView, area: Rect, theme: &Theme) {
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
    let add_new = &app.add_new_view;
    let available_profiles: Vec<_> = app
        .list_view
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
    let add_new = &app.add_new_view;
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
    match app.add_new_view.current_focus() {
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
    let add_new = &app.add_new_view;
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
