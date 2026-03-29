use ratatui::Frame;

use crate::app::{app_state::AppState, views};

pub fn render(app_state: &AppState, frame: &mut Frame) {
    let view = &app_state.view_state;
    views::merge_file_view::render(view, frame);
}
