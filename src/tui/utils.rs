use ratatui::prelude::*;

/// A reusable struct to manage state for a text input field, with robust unicode support.
#[derive(Debug, PartialEq, Eq)]
pub struct Input {
    pub text: String,
    pub cursor_position: usize,
    pub is_valid: bool,
    pub error_message: Option<String>,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = cursor_moved_right.clamp(0, self.text.chars().count());
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = cursor_moved_left.clamp(0, self.text.chars().count());
    }

    pub fn enter_char(&mut self, c: char) {
        let index = self
            .text
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_position)
            .unwrap_or(self.text.len());
        self.text.insert(index, c);
        self.move_cursor_right()
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.text.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.text.chars().skip(current_index);

            self.text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn set_error_message(&mut self, error_message: &str) {
        self.error_message = Some(error_message.to_string());
        self.is_valid = false;
    }

    pub fn reset(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
        self.is_valid = true;
        self.error_message = None;
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            is_valid: true,
            error_message: None,
        }
    }
}
/// Helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

pub fn validate_no_spaces(text: &str) -> std::result::Result<(), String> {
    if text.chars().any(char::is_whitespace) {
        Err("Cannot contain spaces".to_string())
    } else {
        Ok(())
    }
}

pub fn validate_non_empty(text: &str) -> std::result::Result<(), String> {
    if text.trim().is_empty() {
        Err("Cannot be empty".to_string())
    } else {
        Ok(())
    }
}

pub fn validate_starts_with_non_digit(text: &str) -> std::result::Result<(), String> {
    if let Some(first_char) = text.chars().next() {
        if first_char.is_ascii_digit() {
            return Err("Cannot start with a digit".to_string());
        }
    }
    Ok(())
}

pub fn input_to_span<'a>(
    input: &Input,
    is_focused: bool,
    theme: &crate::tui::theme::Theme,
) -> Line<'a> {
    if is_focused {
        // Simple cursor simulation: split text at cursor
        let (left, right) = input.text.split_at(input.cursor_position);
        // Note: Cursor rendering usually requires multiple spans or manual composition
        // For simple usage in a Cell, we might just return the text styled.
        // But the user probably wants to see the cursor.
        // Since we return a single Span, we can't easily do multi-colored chars unless we return `Line`.
        // Wait, `Cell` accepts `Line` or `Span`. Let's assume `input_to_span` returns `Line`.
        // Accessing main_right.rs showed `Cell::from(span)`. Cell::from accepts Span or String or Text or Line.
        // Let's change return type to Line to support cursor highlighting.

        let cursor_char = if right.is_empty() { " " } else { &right[0..1] };
        let right_rest = if right.is_empty() { "" } else { &right[1..] };

        Line::from(vec![
            Span::raw(left.to_string()),
            Span::styled(cursor_char.to_string(), theme.input_cursor()),
            Span::raw(right_rest.to_string()),
        ])
    } else {
        Line::from(input.text.clone())
    }
}
