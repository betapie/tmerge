use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{app_state::AppState, ui::common::centered_rect, views};

pub fn render(app_state: &AppState, frame: &mut Frame) {
    let view = &app_state.view_state;
    views::merge_file_view::render(view, frame);
}

fn render_error(error_message: &str, frame: &mut Frame) {
    let area = frame.area();
    let error_modal_area = centered_rect(50, 50, area);
    frame.render_widget(Clear, error_modal_area);

    let block = Block::default()
        .title("Something went wrong")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Red));

    let text = format!("{}\n\nPress Enter or Esc to dismiss", error_message);
    let para = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

    frame.render_widget(para, error_modal_area);
}
