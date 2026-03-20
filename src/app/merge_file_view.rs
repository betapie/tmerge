use std::{
    fmt,
    fs::File,
    io::{BufWriter, Write},
};

use crate::core::{
    model::{Block, Conflict, MergeFile, Resolution},
    renderer::{render_conflict, render_merge_file},
};

fn collect_conflict_block_indices(merge_file: &MergeFile) -> Vec<usize> {
    merge_file
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| matches!(block, Block::Conflict(_)))
        .map(|(idx, _)| idx)
        .collect()
}

fn collect_unresolved_block_indices(merge_file: &MergeFile) -> Vec<usize> {
    merge_file
    .blocks
    .iter()
    .enumerate()
    .filter(|(_, block)| matches!(block, Block::Conflict(conflict) if conflict.resolution.is_none()))
            .map(|(idx, _)| idx)
            .collect()
}

fn calculate_global_block_lengths(merge_file: &MergeFile) -> Vec<usize> {
    merge_file
        .blocks
        .iter()
        .map(|block| match block {
            Block::Regular(lines) => lines.len(),
            Block::Conflict(conflict) => {
                let ours_len = conflict.ours.lines.len();
                let theirs_len = conflict.theirs.lines.len();
                let merged_len = render_conflict(conflict).len();
                [ours_len, theirs_len, merged_len]
                    .into_iter()
                    .max()
                    .unwrap()
            }
        })
        .collect()
}

#[derive(Debug)]
pub struct InvalidInputError {
    pub message: String,
}

impl InvalidInputError {
    fn new(message: String) -> InvalidInputError {
        InvalidInputError { message }
    }
}

impl fmt::Display for InvalidInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl std::error::Error for InvalidInputError {}

pub struct MergeFileView {
    pub merge_file: MergeFile,
    pub file_path: std::path::PathBuf,

    pub current_block_idx: usize,
    pub current_block_line: usize,
    pub global_block_lengths: Vec<usize>,

    pub conflict_block_indices: Vec<usize>,
    pub unresolved_conflict_block_indices: Vec<usize>,

    pub is_dirty: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl MergeFileView {
    pub fn new(
        merge_file: MergeFile,
        file_path: std::path::PathBuf,
    ) -> Result<MergeFileView, InvalidInputError> {
        if merge_file.blocks.is_empty() {
            Err(InvalidInputError::new(String::from(
                "Merge file must have at least one block",
            )))
        } else {
            let conflict_block_indices = collect_conflict_block_indices(&merge_file);
            let unresolved_conflict_block_indices = collect_unresolved_block_indices(&merge_file);
            let current_block_idx = unresolved_conflict_block_indices
                .first()
                .copied()
                .unwrap_or(0);
            let line_in_block = 0;
            let global_block_lengths = calculate_global_block_lengths(&merge_file);
            Ok(MergeFileView {
                merge_file,
                file_path,
                current_block_idx,
                current_block_line: line_in_block,
                global_block_lengths,
                conflict_block_indices,
                unresolved_conflict_block_indices,
                is_dirty: false,
            })
        }
    }

    pub fn write(&mut self) -> Result<(), WriteError> {
        let lines = render_merge_file(&self.merge_file);
        let file = File::create(&self.file_path)?;
        let mut writer = BufWriter::new(file);
        for line in lines {
            writeln!(writer, "{}", line)?;
        }
        writer.flush()?;
        self.is_dirty = false;
        Ok(())
    }

    pub fn num_conflicts(&self) -> usize {
        self.conflict_block_indices.len()
    }

    pub fn num_unresolved(&self) -> usize {
        self.unresolved_conflict_block_indices.len()
    }

    pub fn all_resolved(&self) -> bool {
        self.unresolved_conflict_block_indices.is_empty()
    }

    pub fn current_conflict_idx(&self) -> Option<usize> {
        self.conflict_block_indices
            .iter()
            .position(|&i| i == self.current_block_idx)
    }

    pub fn current_conflict(&self) -> Option<&Conflict> {
        match &self.merge_file.blocks[self.current_block_idx] {
            Block::Regular(_) => None,
            Block::Conflict(conflict) => Some(conflict),
        }
    }

    pub fn scroll_down(&mut self, num_lines: usize) {
        let mut remaining = num_lines;
        loop {
            let block_len = self.global_block_lengths[self.current_block_idx];
            let lines_left_in_block = block_len.saturating_sub(self.current_block_line + 1);

            if remaining <= lines_left_in_block {
                self.current_block_line += remaining;
                break;
            }

            remaining -= lines_left_in_block + 1;
            if self.current_block_idx + 1 >= self.num_blocks() {
                self.current_block_line = block_len.saturating_sub(1);
                break;
            }

            self.current_block_idx += 1;
            self.current_block_line = 0;
        }
    }

    pub fn scroll_up(&mut self, num_lines: usize) {
        let mut remaining = num_lines;
        loop {
            if remaining <= self.current_block_line {
                self.current_block_line -= remaining;
                break;
            }

            // Step off the top of this block
            remaining -= self.current_block_line + 1;

            if self.current_block_idx == 0 {
                self.current_block_line = 0;
                break;
            }

            self.current_block_idx -= 1;
            self.current_block_line =
                self.global_block_lengths[self.current_block_idx].saturating_sub(1);
        }
    }

    pub fn jump_to_next_conflict(&mut self) {
        if let Some(&next) = self
            .conflict_block_indices
            .iter()
            .find(|&&i| i > self.current_block_idx)
        {
            self.current_block_idx = next;
            self.current_block_line = 0;
        }
    }

    pub fn jump_to_prev_conflict(&mut self) {
        if let Some(&prev) = self
            .conflict_block_indices
            .iter()
            .rev()
            .find(|&&i| i < self.current_block_idx)
        {
            self.current_block_idx = prev;
            self.current_block_line = 0;
        }
    }

    pub fn jump_to_next_unresolved(&mut self) {
        if let Some(&next) = self
            .unresolved_conflict_block_indices
            .iter()
            .find(|&&i| i > self.current_block_idx)
        {
            self.current_block_idx = next;
            self.current_block_line = 0;
        }
    }

    pub fn jump_to_prev_unresolved(&mut self) {
        if let Some(&prev) = self
            .unresolved_conflict_block_indices
            .iter()
            .rev()
            .find(|&&i| i < self.current_block_idx)
        {
            self.current_block_idx = prev;
            self.current_block_line = 0;
        }
    }

    pub fn num_blocks(&self) -> usize {
        self.merge_file.blocks.len()
    }

    pub fn resolve_current(&mut self, resolution: Resolution) {
        if let Some(Block::Conflict(conflict)) =
            self.merge_file.blocks.get_mut(self.current_block_idx)
        {
            conflict.resolution = Some(resolution);
            self.is_dirty = true;
            self.invalidate();
        }
    }

    pub fn unresolve_current(&mut self) {
        if let Some(Block::Conflict(conflict)) =
            self.merge_file.blocks.get_mut(self.current_block_idx)
        {
            conflict.resolution = None;
            self.is_dirty = true;
            self.invalidate();
        }
    }

    fn invalidate(&mut self) {
        self.unresolved_conflict_block_indices = collect_unresolved_block_indices(&self.merge_file);
        self.global_block_lengths = calculate_global_block_lengths(&self.merge_file);
        // invalidate cursor position
        let current_block_len = self.global_block_lengths[self.current_block_idx];
        if current_block_len > 0 {
            self.current_block_line = self.current_block_line.min(current_block_len - 1);
        } else {
            self.current_block_line = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_helpers;

    #[test]
    fn new_merge_file_view_from_empty_merge_file_returns_err() {
        let merge_file = MergeFile { blocks: Vec::new() };
        let filepath = std::path::PathBuf::from("test/path.ext");
        let view = MergeFileView::new(merge_file, filepath);
        assert!(view.is_err());
    }

    #[test]
    fn new_merge_file_from_non_empty_merge_file_returns_ok() {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
            test_helpers::BlockType::Regular,
        ]);
        let filepath = std::path::PathBuf::from("test/path.ext");

        let view = MergeFileView::new(merge_file, filepath);
        assert!(view.is_ok());
    }

    #[test]
    fn new_merge_file_from_non_empty_merge_file_initializes_as_expected()
    -> Result<(), InvalidInputError> {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
            test_helpers::BlockType::Regular,
        ]);
        let filepath = std::path::PathBuf::from("test/path.ext");

        let merge_file_view = MergeFileView::new(merge_file, filepath)?;
        assert_eq!(merge_file_view.num_conflicts(), 1);
        assert_eq!(merge_file_view.num_unresolved(), 1);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(0)));
        Ok(())
    }

    #[test]
    fn test_scroll_down_and_up() -> Result<(), InvalidInputError> {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
            test_helpers::BlockType::Regular,
        ]);
        let filepath = std::path::PathBuf::from("test/path.ext");
        let mut merge_file_view = MergeFileView::new(merge_file, filepath)?;

        let expected_global_block_lengths = vec![4, 10, 4];
        assert_eq!(
            merge_file_view.global_block_lengths,
            expected_global_block_lengths,
        );

        assert_eq!(merge_file_view.current_block_idx, 1);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(0)));

        merge_file_view.scroll_down(1);
        assert_eq!(merge_file_view.current_block_idx, 1);
        assert_eq!(merge_file_view.current_block_line, 1);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(0)));

        merge_file_view.scroll_up(1);
        assert_eq!(merge_file_view.current_block_idx, 1);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(0)));

        merge_file_view.scroll_up(2);
        assert_eq!(merge_file_view.current_block_idx, 0);
        assert_eq!(merge_file_view.current_block_line, 2);
        assert!(merge_file_view.current_conflict_idx().is_none());

        merge_file_view.scroll_up(2);
        assert_eq!(merge_file_view.current_block_idx, 0);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(merge_file_view.current_conflict_idx().is_none());

        merge_file_view.scroll_up(2);
        assert_eq!(merge_file_view.current_block_idx, 0);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(merge_file_view.current_conflict_idx().is_none());

        merge_file_view.scroll_down(8);
        assert_eq!(merge_file_view.current_block_idx, 1);
        assert_eq!(merge_file_view.current_block_line, 4);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(0)));

        merge_file_view.scroll_down(42);
        assert_eq!(merge_file_view.current_block_idx, 2);
        assert_eq!(merge_file_view.current_block_line, 3);
        assert!(merge_file_view.current_conflict_idx().is_none());

        merge_file_view.scroll_up(42);
        assert_eq!(merge_file_view.current_block_idx, 0);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(merge_file_view.current_conflict_idx().is_none());

        Ok(())
    }

    #[test]
    fn test_jump_to_conflicts() -> Result<(), InvalidInputError> {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);
        let filepath = std::path::PathBuf::from("test/path.ext");
        let mut merge_file_view = MergeFileView::new(merge_file, filepath)?;

        let expected_global_block_lengths = vec![4, 10, 4, 10];
        assert_eq!(
            merge_file_view.global_block_lengths,
            expected_global_block_lengths,
        );

        assert_eq!(merge_file_view.current_block_idx, 1);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(0)));

        merge_file_view.jump_to_next_conflict();
        assert_eq!(merge_file_view.current_block_idx, 3);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(1)));

        merge_file_view.jump_to_next_conflict();
        assert_eq!(merge_file_view.current_block_idx, 3);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(1)));

        merge_file_view.scroll_down(7);
        assert_eq!(merge_file_view.current_block_idx, 3);
        assert_eq!(merge_file_view.current_block_line, 7);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(1)));

        merge_file_view.jump_to_prev_conflict();
        assert_eq!(merge_file_view.current_block_idx, 1);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(0)));

        merge_file_view.jump_to_prev_conflict();
        assert_eq!(merge_file_view.current_block_idx, 1);
        assert_eq!(merge_file_view.current_block_line, 0);
        assert!(matches!(merge_file_view.current_conflict_idx(), Some(0)));

        Ok(())
    }

    #[test]
    fn resolve_current_when_not_in_conflict_does_nothing() -> Result<(), InvalidInputError> {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);
        let filepath = std::path::PathBuf::from("test/path.ext");
        let mut merge_file_view = MergeFileView::new(merge_file, filepath)?;

        let num_conflicts_before = merge_file_view.num_conflicts();
        let num_unresolved_before = merge_file_view.num_unresolved();

        merge_file_view.scroll_up(1);
        assert!(merge_file_view.current_conflict_idx().is_none());

        merge_file_view.resolve_current(Resolution::Ours);

        let num_conflicts = merge_file_view.num_conflicts();
        let num_unresolved = merge_file_view.num_unresolved();

        assert_eq!(num_conflicts_before, num_conflicts);
        assert_eq!(num_unresolved_before, num_unresolved);

        Ok(())
    }

    #[test]
    fn resolve_current_when_in_conflict_resolves_and_recalculates() -> Result<(), InvalidInputError>
    {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);
        let filepath = std::path::PathBuf::from("test/path.ext");
        let mut merge_file_view = MergeFileView::new(merge_file, filepath)?;

        let expected_global_block_lengths = vec![4, 10, 4, 10];
        assert_eq!(
            merge_file_view.global_block_lengths,
            expected_global_block_lengths,
        );
        let num_conflicts_before = merge_file_view.num_conflicts();
        let num_unresolved_before = merge_file_view.num_unresolved();
        assert!(merge_file_view.current_conflict_idx().is_some());
        assert!(
            merge_file_view
                .current_conflict()
                .unwrap()
                .resolution
                .is_none()
        );

        merge_file_view.resolve_current(Resolution::Ours);

        let num_conflicts = merge_file_view.num_conflicts();
        let num_unresolved = merge_file_view.num_unresolved();

        assert_eq!(num_conflicts_before, num_conflicts);
        assert_eq!(num_unresolved_before, num_unresolved + 1);

        assert!(merge_file_view.current_conflict_idx().is_some());
        assert!(matches!(
            merge_file_view.current_conflict().unwrap().resolution,
            Some(Resolution::Ours)
        ));

        let expected_global_block_lengths = vec![4, 3, 4, 10];
        assert_eq!(
            merge_file_view.global_block_lengths,
            expected_global_block_lengths,
        );

        Ok(())
    }

    fn make_merge_file_view_and_resolve_first() -> MergeFileView {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);
        let filepath = std::path::PathBuf::from("test/path.ext");
        let mut merge_file_view = MergeFileView::new(merge_file, filepath).unwrap();

        let expected_global_block_lengths = vec![4, 10, 4, 10];
        assert_eq!(
            merge_file_view.global_block_lengths,
            expected_global_block_lengths,
        );
        let num_conflicts_before = merge_file_view.num_conflicts();
        let num_unresolved_before = merge_file_view.num_unresolved();
        assert!(merge_file_view.current_conflict_idx().is_some());
        assert!(
            merge_file_view
                .current_conflict()
                .unwrap()
                .resolution
                .is_none()
        );

        merge_file_view.resolve_current(Resolution::Ours);

        let num_conflicts = merge_file_view.num_conflicts();
        let num_unresolved = merge_file_view.num_unresolved();

        assert_eq!(num_conflicts_before, num_conflicts);
        assert_eq!(num_unresolved_before, num_unresolved + 1);

        assert!(merge_file_view.current_conflict_idx().is_some());
        assert!(matches!(
            merge_file_view.current_conflict().unwrap().resolution,
            Some(Resolution::Ours)
        ));

        let expected_global_block_lengths = vec![4, 3, 4, 10];
        assert_eq!(
            merge_file_view.global_block_lengths,
            expected_global_block_lengths,
        );
        merge_file_view
    }

    #[test]
    fn resolve_current_when_already_resolved_changes_resolution_state() {
        let mut merge_file_view = make_merge_file_view_and_resolve_first();

        let num_conflicts_before = merge_file_view.num_conflicts();
        let num_unresolved_before = merge_file_view.num_unresolved();

        merge_file_view.resolve_current(Resolution::Theirs);

        let num_conflicts = merge_file_view.num_conflicts();
        let num_unresolved = merge_file_view.num_unresolved();

        assert_eq!(num_conflicts_before, num_conflicts);
        assert_eq!(num_unresolved_before, num_unresolved);

        assert!(merge_file_view.current_conflict_idx().is_some());
        assert!(matches!(
            merge_file_view.current_conflict().unwrap().resolution,
            Some(Resolution::Theirs)
        ));

        let expected_global_block_lengths = vec![4, 3, 4, 10];
        assert_eq!(
            merge_file_view.global_block_lengths,
            expected_global_block_lengths,
        );
    }

    #[test]
    fn unresolve_current_when_already_resolved_resets_resolution_and_recomputes() {
        let mut merge_file_view = make_merge_file_view_and_resolve_first();

        let num_conflicts_before = merge_file_view.num_conflicts();
        let num_unresolved_before = merge_file_view.num_unresolved();

        merge_file_view.unresolve_current();

        let num_conflicts = merge_file_view.num_conflicts();
        let num_unresolved = merge_file_view.num_unresolved();

        assert_eq!(num_conflicts_before, num_conflicts);
        assert_eq!(num_unresolved_before + 1, num_unresolved);

        assert!(merge_file_view.current_conflict_idx().is_some());
        assert!(
            merge_file_view
                .current_conflict()
                .unwrap()
                .resolution
                .is_none()
        );

        let expected_global_block_lengths = vec![4, 10, 4, 10];
        assert_eq!(
            merge_file_view.global_block_lengths,
            expected_global_block_lengths,
        );
    }
}
