use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::app_state::AppState;
use crate::app::views;

pub fn handle_events(app_state: &mut AppState) -> std::io::Result<()> {
    if event::poll(Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            handle_key(app_state, key);
        }
    }
    Ok(())
}

fn handle_key(app_state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => {
            app_state.should_quit = true;
            return;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app_state.should_quit = true;
            return;
        }
        _ => {}
    }
    if app_state.current_error.is_some() {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                app_state.current_error = None;
            }
            _ => {}
        }
        return;
    }

    let view = &mut app_state.view_state;
    if let Err(error) = views::merge_file_view::handle_key(view, key, &mut app_state.force_redraw) {
        app_state.current_error = Some(error);
    }
}
