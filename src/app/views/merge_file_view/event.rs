use crate::{
    app::{app_state::AppState, event::Action, ui::editor, views::merge_file_view::state::State},
    core::{model::Resolution, renderer::render_conflict},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(merge_file_view: &State, key: KeyEvent) -> Option<Action> {
    if merge_file_view.current_error.is_some() {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(Box::new(|app_state: &mut AppState| {
                app_state.view_state.current_error = None
            })),
            _ => None,
        }
    } else if merge_file_view.show_help {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                Some(Box::new(|app| app.view_state.show_help = false))
            }
            _ => None,
        }
    } else {
        handle_key_regular(merge_file_view, key)
    }
}

fn handle_key_regular(_merge_file_view: &State, key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('?') => Some(Box::new(|app_state: &mut AppState| {
            app_state.view_state.show_help = true
        })),
        KeyCode::Char('j') | KeyCode::Down => Some(Box::new(|app_state: &mut AppState| {
            app_state.view_state.scroll_down(1)
        })),
        KeyCode::Char('k') | KeyCode::Up => Some(Box::new(|app_state: &mut AppState| {
            app_state.view_state.scroll_up(1)
        })),
        KeyCode::Char('d') => Some(Box::new(|app_state: &mut AppState| {
            app_state.view_state.scroll_down(10)
        })),
        KeyCode::Char('u') => Some(Box::new(|app_state: &mut AppState| {
            app_state.view_state.scroll_up(10)
        })),
        KeyCode::Char('n') => Some(Box::new(move |app_state: &mut AppState| {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app_state.view_state.jump_to_next_unresolved();
            } else {
                app_state.view_state.jump_to_next_conflict();
            }
        })),
        KeyCode::Char('p') => Some(Box::new(move |app_state: &mut AppState| {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app_state.view_state.jump_to_prev_unresolved();
            } else {
                app_state.view_state.jump_to_prev_conflict();
            }
        })),
        KeyCode::Char('o') => Some(Box::new(|app_state: &mut AppState| {
            app_state.view_state.resolve_current(Resolution::Ours);
        })),
        KeyCode::Char('t') => Some(Box::new(|app_state: &mut AppState| {
            app_state.view_state.resolve_current(Resolution::Theirs);
        })),
        KeyCode::Char('e') => Some(Box::new(move |app_state: &mut AppState| {
            if let Some(conflict) = app_state.view_state.current_conflict() {
                let conflict_lines = render_conflict(conflict);
                let edit_result = editor::edit(&conflict_lines);
                app_state.force_redraw = true;
                match edit_result {
                    Ok(edited) => {
                        if edited != conflict_lines {
                            app_state
                                .view_state
                                .resolve_current(Resolution::Edited(edited));
                        }
                    }
                    Err(error) => {
                        app_state.view_state.current_error = Some(error.to_string());
                    }
                }
            }
        })),
        KeyCode::Char('c') => Some(Box::new(|app_state: &mut AppState| {
            app_state.view_state.unresolve_current()
        })),
        KeyCode::Char('w') => Some(Box::new(|app_state: &mut AppState| {
            match app_state.view_state.write() {
                Ok(_) => {}
                Err(error) => {
                    app_state.view_state.current_error = Some(error.to_string());
                }
            }
        })),
        _ => None,
    }
}
