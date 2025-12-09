use ratatui::prelude::*;

/// A reusable struct to manage state for a text input field, with robust unicode support.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Input {
    text: String,
    cursor_position: usize,
    error_message: Option<String>,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an Input from its individual parts
    pub fn from_parts(text: String, cursor_position: usize, error_message: Option<String>) -> Self {
        Self {
            text,
            cursor_position,
            error_message,
        }
    }

    /// Create an Input from text, with cursor at end
    pub fn with_text(text: String) -> Self {
        let cursor_position = text.len();
        Self {
            text,
            cursor_position,
            error_message: None,
        }
    }

    /// Check if input is valid (no error message)
    pub fn is_valid(&self) -> bool {
        self.error_message.is_none()
    }

    /// Get the error message if any
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Clear the error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Get the cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Set the cursor position
    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position.clamp(0, self.text.chars().count());
    }

    /// Get the text content as a string slice
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text content, moving cursor to end
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_position = self.text.len();
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
    }

    pub fn reset(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
        self.error_message = None;
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
    if let Some(first_char) = text.chars().next()
        && first_char.is_ascii_digit()
    {
        return Err("Cannot start with a digit".to_string());
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
        let (left, right) = input.text.split_at(
            input
                .text
                .char_indices()
                .nth(input.cursor_position)
                .map(|(i, _)| i)
                .unwrap_or(input.text.len()),
        );

        let cursor_char = if right.is_empty() {
            " "
        } else {
            &right[..right.chars().next().unwrap().len_utf8()]
        };

        let right_rest = if right.is_empty() {
            ""
        } else {
            &right[cursor_char.len()..]
        };

        Line::from(vec![
            Span::raw(left.to_string()),
            Span::styled(cursor_char.to_string(), theme.input_cursor()),
            Span::raw(right_rest.to_string()),
        ])
    } else {
        Line::from(input.text.clone())
    }
}
