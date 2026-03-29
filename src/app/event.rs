use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::app_state::AppState;
use crate::app::views;

pub type Action = Box<dyn FnOnce(&mut AppState)>;

pub fn handle_events(app_state: &mut AppState) -> std::io::Result<()> {
    if event::poll(Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if let Some(action) = handle_key(app_state, key) {
                action(app_state);
            }
        }
    }
    Ok(())
}

fn handle_key(app_state: &mut AppState, key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') => {
            app_state.should_quit = true;
            return None;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app_state.should_quit = true;
            return None;
        }
        _ => {}
    }

    let view = &mut app_state.view_state;
    views::merge_file_view::handle_key(view, key)
}
