use crate::app::views::merge_file_view;
use crate::core::model::MergeFile;

pub struct AppState {
    pub view_state: merge_file_view::State,
    pub should_quit: bool,
    pub force_redraw: bool,
    pub current_error: Option<String>,
}

impl AppState {
    pub fn new(
        merge_file: MergeFile,
        file_path: std::path::PathBuf,
    ) -> Result<Self, merge_file_view::InvalidInputError> {
        let view_state = merge_file_view::State::new(merge_file, file_path)?;
        Ok(Self {
            view_state,
            should_quit: false,
            force_redraw: false,
            current_error: None,
        })
    }
}
