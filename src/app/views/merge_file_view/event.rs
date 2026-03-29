use crate::{
    app::{
        app_state::AppState,
        event::Action,
        ui::{
            editor,
            selection_dialog::{self, Item},
        },
        views::merge_file_view::{Modal, state::State},
    },
    core::{model::Resolution, renderer::render_conflict},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(merge_file_view: &mut State, key: KeyEvent) -> Option<Action> {
    if merge_file_view.current_error.is_some() {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => merge_file_view.current_error = None,
            _ => {}
        }
        None
    } else if let Some(modal) = &mut merge_file_view.current_modal {
        match modal {
            super::Modal::Help => match key.code {
                KeyCode::Char('?') | KeyCode::Esc => {
                    merge_file_view.current_modal = None;
                }
                _ => {}
            },
            super::Modal::ResolutionSelection(selection_dialog) => {
                match selection_dialog::handle_key(selection_dialog, key).clone() {
                    selection_dialog::Selection::Pending => {}
                    selection_dialog::Selection::Cancelled => merge_file_view.current_modal = None,
                    selection_dialog::Selection::Selected(resolution) => {
                        let resolution = resolution.clone();
                        match resolution {
                            Resolution::Edited(_) => {
                                merge_file_view.current_modal = None;
                                return edit_current(merge_file_view);
                            }
                            _ => {
                                merge_file_view.current_modal = None;
                                merge_file_view.resolve_current(resolution);
                            }
                        }
                    }
                }
            }
        }
        None
    } else {
        handle_key_regular(merge_file_view, key)
    }
}

fn handle_key_regular(merge_file_view: &mut State, key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('?') => merge_file_view.current_modal = Some(Modal::Help),
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
        KeyCode::Char('o') => merge_file_view.resolve_current(Resolution::Ours),
        KeyCode::Char('t') => merge_file_view.resolve_current(Resolution::Theirs),
        KeyCode::Char('e') => {
            return edit_current(merge_file_view);
        }
        KeyCode::Char('r') => {
            if let Some(dlg) = selection_dialog::State::<Resolution>::new(
                String::from("All resolution options"),
                vec![
                    Item::<Resolution> {
                        label: String::from("use ours"),
                        key: Some('o'),
                        value: Resolution::Ours,
                    },
                    Item::<Resolution> {
                        label: String::from("use theirs"),
                        key: Some('t'),
                        value: Resolution::Theirs,
                    },
                    Item::<Resolution> {
                        label: String::from("use theirs before ours"),
                        key: Some('a'),
                        value: Resolution::TheirsBeforeOurs,
                    },
                    Item::<Resolution> {
                        label: String::from("use ours before theirs"),
                        key: Some('b'),
                        value: Resolution::OursBeforeTheirs,
                    },
                    Item::<Resolution> {
                        label: String::from("edit"),
                        key: Some('e'),
                        value: Resolution::Edited(Vec::new()),
                    },
                ],
            ) {
                merge_file_view.current_modal = Some(Modal::ResolutionSelection(dlg));
            } else {
                merge_file_view.current_error =
                    Some(String::from("failed to create selection dialog"));
            }
        }
        KeyCode::Char('c') => merge_file_view.unresolve_current(),
        KeyCode::Char('w') => match merge_file_view.write() {
            Ok(_) => {}
            Err(error) => {
                merge_file_view.current_error = Some(error.to_string());
            }
        },
        _ => {}
    };
    None
}

fn edit_current(merge_file_view: &mut State) -> Option<Action> {
    if let Some(conflict) = merge_file_view.current_conflict() {
        let conflict_lines = render_conflict(conflict);
        let edit_result = editor::edit(&conflict_lines);
        match edit_result {
            Ok(edited) => {
                if edited != conflict_lines {
                    merge_file_view.resolve_current(Resolution::Edited(edited));
                }
            }
            Err(error) => {
                merge_file_view.current_error = Some(error.to_string());
            }
        }
        return Some(Box::new(move |app_state: &mut AppState| {
            app_state.force_redraw = true;
        }));
    }
    None
}
