use crate::tui::{
    components::select_popup::SelectPopupComponent, theme::Theme, utils::centered_rect,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub fn render(frame: &mut Frame<'_>, popup: &SelectPopupComponent) {
    let theme = Theme::new();
    let popup_area = centered_rect(50, 50, frame.area());

    frame.render_widget(Clear, popup_area);

    let count_info = if popup.options.is_empty() {
        "".to_string()
    } else {
        format!(" ({}/{})", popup.current_index + 1, popup.options.len())
    };

    let title = format!("{}{}", popup.title, count_info);

    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.block_active());

    let inner_area = popup_block.inner(popup_area);
    frame.render_widget(popup_block, popup_area);

    if popup.options.is_empty() {
        let p = Paragraph::new("No available profiles.").alignment(Alignment::Center);
        frame.render_widget(p, inner_area);
        return;
    }

    let items: Vec<ListItem> = popup
        .options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let is_selected = if popup.multi_select {
                popup.selected_indices.contains(&i)
            } else {
                false
            };

            let prefix = if is_selected {
                "[x] "
            } else if popup.multi_select {
                "[ ] "
            } else {
                "   "
            };

            ListItem::new(format!("{}{}", prefix, opt))
        })
        .collect();

    let list = List::new(items).highlight_style(theme.row_selected()); // Use background highlight style

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(popup.current_index));

    frame.render_stateful_widget(list, inner_area, &mut state);
}
