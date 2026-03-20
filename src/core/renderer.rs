use crate::core::constants::markers;
use crate::core::model::{Block, Conflict, MergeFile, Resolution};

pub fn render_merge_file(merge_file: &MergeFile) -> Vec<String> {
    let mut result = Vec::new();
    for block in &merge_file.blocks {
        match block {
            Block::Regular(lines) => {
                result.extend_from_slice(lines);
            }
            Block::Conflict(conflict) => {
                result.extend(render_conflict(conflict));
            }
        }
    }
    result
}

pub fn render_conflict(conflict: &Conflict) -> Vec<String> {
    let result_lines = match &conflict.resolution {
        Some(resolution) => match resolution {
            Resolution::Ours => conflict.ours.lines.clone(),
            Resolution::Theirs => conflict.theirs.lines.clone(),
            Resolution::Edited(lines) => lines.clone(),
        },
        None => {
            let mut result = Vec::new();
            result.push(if let Some(tag) = &conflict.ours.tag {
                format!("{} {}", markers::OURS_BEGIN, tag)
            } else {
                markers::OURS_BEGIN.into()
            });
            result.extend(conflict.ours.lines.clone());
            if let Some(base) = &conflict.base {
                result.push(if let Some(tag) = &base.tag {
                    format!("{} {}", markers::BASE_BEGIN, tag)
                } else {
                    markers::BASE_BEGIN.into()
                });
                result.extend(base.lines.clone());
            }
            result.push(markers::THEIRS_BEGIN.into());
            result.extend(conflict.theirs.lines.clone());
            result.push(if let Some(tag) = &conflict.theirs.tag {
                format!("{} {}", markers::CONFLICT_END, tag)
            } else {
                markers::CONFLICT_END.into()
            });

            result
        }
    };
    result_lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_helpers::{self};

    #[test]
    fn render_conflict_on_unresolved_diff2_block_returns_raw() {
        let test_helpers::TestConflict {
            raw_lines,
            parsed: conflict,
        } = test_helpers::make_diff2_conflict();
        assert!(conflict.resolution.is_none());
        let rendered = render_conflict(&conflict);
        assert_eq!(rendered, raw_lines);
    }

    #[test]
    fn render_on_diff2_block_when_resolved_with_ours_returns_ours() {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff2_conflict();
        conflict.resolution = Some(Resolution::Ours);
        let rendered = render_conflict(&conflict);
        assert_eq!(rendered, conflict.ours.lines);
    }

    #[test]
    fn render_conflict_on_diff2_block_when_resolved_with_theirs_returs_theirs() {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff2_conflict();
        conflict.resolution = Some(Resolution::Theirs);
        let rendered = render_conflict(&conflict);
        assert_eq!(rendered, conflict.theirs.lines);
    }

    #[test]
    fn render_on_unresolved_diff3_block_returns_raw() {
        let test_helpers::TestConflict {
            raw_lines,
            parsed: conflict,
        } = test_helpers::make_diff3_conflict();
        assert!(conflict.resolution.is_none());
        let rendered = render_conflict(&conflict);
        assert_eq!(rendered, raw_lines);
    }

    #[test]
    fn render_conflict_on_diff3_block_when_resolved_with_ours_returns_ours() {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff3_conflict();
        conflict.resolution = Some(Resolution::Ours);
        let rendered = render_conflict(&conflict);
        assert_eq!(rendered, conflict.ours.lines);
    }

    #[test]
    fn render_conflict_on_diff3_block_when_resolved_with_theirs_returs_theirs() {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff3_conflict();
        conflict.resolution = Some(Resolution::Theirs);
        let rendered = render_conflict(&conflict);
        assert_eq!(rendered, conflict.theirs.lines);
    }

    #[test]
    fn render_merge_file_with_no_blocks_returns_empty() {
        let merge_file = MergeFile { blocks: Vec::new() };
        let rendered = render_merge_file(&merge_file);
        assert!(rendered.is_empty());
    }

    #[test]
    fn render_merge_file_with_single_regular_block_produces_expected() {
        let regular_block_lines = test_helpers::make_regular_block();
        let merge_file = MergeFile {
            blocks: vec![Block::Regular(regular_block_lines.clone())],
        };
        let rendered = render_merge_file(&merge_file);
        assert_eq!(rendered, regular_block_lines);
    }

    #[test]
    fn render_merge_file_with_mixed_unresolved_blocks_produces_expected() {
        let test_helpers::TestMergeFile {
            raw_lines: expected_rendered,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Diff2,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);
        let rendered = render_merge_file(&merge_file);
        assert_eq!(rendered, expected_rendered);
    }

    #[test]
    fn render_merge_file_with_mixed_resolved_and_unresolved_blocks_produces_expected() {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: mut merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Diff2,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);

        let mut expected_rendered = Vec::new();

        if let Block::Conflict(conflict) = &mut merge_file.blocks[0] {
            conflict.resolution = Some(Resolution::Ours);
            expected_rendered.extend(conflict.ours.lines.clone());
        } else {
            panic!("Expected first block to be conflict block");
        }

        if let Block::Regular(lines) = &merge_file.blocks[1] {
            expected_rendered.extend(lines.clone());
        } else {
            panic!("Expected second block to be regular block");
        }

        if let Block::Conflict(conflict) = &merge_file.blocks[2] {
            expected_rendered.extend(render_conflict(conflict));
        } else {
            panic!("Expected third block to be conflict block");
        }

        let rendered = render_merge_file(&merge_file);
        assert_eq!(rendered, expected_rendered);
    }

    #[test]
    fn render_merge_file_with_all_resolved_and_unresolved_blocks_produces_expected() {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: mut merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Diff2,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);

        let mut expected_renderd = Vec::new();

        if let Block::Conflict(conflict) = &mut merge_file.blocks[0] {
            let edited_lines = vec![String::from("this was"), String::from("edited")];
            conflict.resolution = Some(Resolution::Edited(edited_lines.clone()));
            expected_renderd.extend(edited_lines);
        } else {
            panic!("Expected first block to be conflict block");
        }

        if let Block::Regular(lines) = &merge_file.blocks[1] {
            expected_renderd.extend(lines.clone());
        } else {
            panic!("Expected second block to be regular block");
        }

        if let Block::Conflict(conflict) = &mut merge_file.blocks[2] {
            conflict.resolution = Some(Resolution::Theirs);
            expected_renderd.extend(conflict.theirs.lines.clone());
        } else {
            panic!("Expected third block to be conflict block");
        }

        let rendered = render_merge_file(&merge_file);
        assert_eq!(rendered, expected_renderd);
    }
}
