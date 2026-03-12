use crate::app::merge_file_view::{InvalidInputError, MergeFileView};
use crate::core::model::MergeFile;

pub struct App {
    pub view: MergeFileView,
    pub should_quit: bool,
    pub write_requested: bool,
}

impl App {
    pub fn new(
        merge_file: MergeFile,
        file_path: std::path::PathBuf,
    ) -> Result<Self, InvalidInputError> {
        let merge_file_view = MergeFileView::new(merge_file, file_path)?;
        Ok(Self {
            view: merge_file_view,
            should_quit: false,
            write_requested: false,
        })
    }
}
