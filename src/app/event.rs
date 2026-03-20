use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::app::App;

pub fn handle_events(app: &mut App) -> std::io::Result<()> {
    if event::poll(Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            handle_key(app, key);
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return;
        }
        _ => {}
    }
    if app.current_error.is_some() {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                app.current_error = None;
            }
            _ => {}
        }
        return;
    }

    let view = &mut app.view;
    if let Err(error) = merge_file_view::handle_key(view, key, &mut app.force_redraw) {
        app.current_error = Some(error);
    }
}

mod merge_file_view {
    use crate::{
        app::{editor, merge_file_view::MergeFileView},
        core::{model::Resolution, renderer::render_conflict},
    };
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    pub fn handle_key(
        merge_file_view: &mut MergeFileView,
        key: KeyEvent,
        force_redraw: &mut bool,
    ) -> Result<(), String> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => merge_file_view.scroll_down(1),
            KeyCode::Char('k') | KeyCode::Up => merge_file_view.scroll_up(1),
            KeyCode::Char('d') => merge_file_view.scroll_down(10),
            KeyCode::Char('u') => merge_file_view.scroll_up(10),
            KeyCode::Char('n') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    merge_file_view.jump_to_next_unresolved();
                } else {
                    merge_file_view.jump_to_next_conflict();
                }
            }
            KeyCode::Char('p') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    merge_file_view.jump_to_prev_unresolved();
                } else {
                    merge_file_view.jump_to_prev_conflict();
                }
            }
            KeyCode::Char('o') => {
                merge_file_view.resolve_current(Resolution::Ours);
            }
            KeyCode::Char('t') => {
                merge_file_view.resolve_current(Resolution::Theirs);
            }
            KeyCode::Char('e') => {
                if let Some(conflict) = merge_file_view.current_conflict() {
                    let conflict_lines = render_conflict(conflict).unwrap();
                    let edit_result = editor::edit(&conflict_lines);
                    *force_redraw = true;
                    match edit_result {
                        Ok(edited) => {
                            if edited != conflict_lines {
                                merge_file_view.resolve_current(Resolution::Edited(edited));
                            }
                        }
                        Err(error) => {
                            return Err(error.to_string());
                        }
                    }
                }
            }
            KeyCode::Char('c') => merge_file_view.unresolve_current(),
            KeyCode::Char('w') => match merge_file_view.write() {
                Ok(_) => {}
                Err(error) => {
                    return Err(error.to_string());
                }
            },
            _ => {}
        }
        Ok(())
    }
}
