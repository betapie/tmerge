use crate::{
    app::{ui::editor, views::merge_file_view::state::State},
    core::{model::Resolution, renderer::render_conflict},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(merge_file_view: &mut State, key: KeyEvent, force_redraw: &mut bool) {
    if merge_file_view.current_error.is_some() {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                merge_file_view.current_error = None;
            }
            _ => {}
        }
        return;
    }
    if merge_file_view.show_help {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                merge_file_view.show_help = false;
            }
            _ => {}
        }
        return;
    }
    if let Err(error) = handle_key_regular(merge_file_view, key, force_redraw) {
        merge_file_view.current_error = Some(error);
    }
}

fn handle_key_regular(
    merge_file_view: &mut State,
    key: KeyEvent,
    force_redraw: &mut bool,
) -> Result<(), String> {
    match key.code {
        KeyCode::Char('?') => merge_file_view.show_help = true,
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
                let conflict_lines = render_conflict(conflict);
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
