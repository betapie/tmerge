use std::fmt;

use crate::core::constants::markers;
use crate::core::model::{Block, Conflict, MergeFile, Resolution};

#[derive(Debug)]
pub struct RenderError {
    pub message: String,
}

impl RenderError {
    fn new(message: String) -> RenderError {
        RenderError { message }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Render error: {}", self.message)
    }
}

impl std::error::Error for RenderError {}

pub fn render_merge_file(merge_file: &MergeFile) -> Result<Vec<String>, RenderError> {
    let mut result = Vec::new();
    for block in &merge_file.blocks {
        match block {
            Block::Regular(lines) => {
                result.extend_from_slice(lines);
            }
            Block::Conflict(conflict) => {
                result.extend(render_conflict(conflict)?);
            }
        }
    }
    Ok(result)
}

pub fn render_conflict(conflict: &Conflict) -> Result<Vec<String>, RenderError> {
    let result_lines = match &conflict.resolution {
        Some(resolution) => match resolution {
            Resolution::Ours => conflict.ours.lines.clone(),
            Resolution::Theirs => conflict.theirs.lines.clone(),
            Resolution::Base => {
                if let Some(base) = &conflict.base {
                    base.lines.clone()
                } else {
                    return Err(RenderError::new(String::from(
                        "conflict has no base state. Cannot resolve with base",
                    )));
                }
            }
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
    Ok(result_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_helpers::{self};

    #[test]
    fn render_conflict_on_unresolved_diff2_block_returns_raw() -> Result<(), RenderError> {
        let test_helpers::TestConflict {
            raw_lines,
            parsed: conflict,
        } = test_helpers::make_diff2_conflict();
        assert!(conflict.resolution.is_none());
        let renderd = render_conflict(&conflict)?;
        assert_eq!(renderd, raw_lines);
        Ok(())
    }

    #[test]
    fn render_on_diff2_block_when_resolved_with_ours_returns_ours() -> Result<(), RenderError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff2_conflict();
        conflict.resolution = Some(Resolution::Ours);
        let renderd = render_conflict(&conflict)?;
        assert_eq!(renderd, conflict.ours.lines);
        Ok(())
    }

    #[test]
    fn render_conflict_on_diff2_block_when_resolved_with_base_returns_error() {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff2_conflict();
        conflict.resolution = Some(Resolution::Base);
        let renderd = render_conflict(&conflict);
        assert!(renderd.is_err());
    }

    #[test]
    fn render_conflict_on_diff2_block_when_resolved_with_theirs_returs_theirs()
    -> Result<(), RenderError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff2_conflict();
        conflict.resolution = Some(Resolution::Theirs);
        let renderd = render_conflict(&conflict)?;
        assert_eq!(renderd, conflict.theirs.lines);
        Ok(())
    }

    #[test]
    fn render_on_unresolved_diff3_block_returns_raw() -> Result<(), RenderError> {
        let test_helpers::TestConflict {
            raw_lines,
            parsed: conflict,
        } = test_helpers::make_diff3_conflict();
        assert!(conflict.resolution.is_none());
        let renderd = render_conflict(&conflict)?;
        assert_eq!(renderd, raw_lines);
        Ok(())
    }

    #[test]
    fn render_conflict_on_diff3_block_when_resolved_with_ours_returns_ours()
    -> Result<(), RenderError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff3_conflict();
        conflict.resolution = Some(Resolution::Ours);
        let renderd = render_conflict(&conflict)?;
        assert_eq!(renderd, conflict.ours.lines);
        Ok(())
    }

    #[test]
    fn render_conflict_on_diff3_block_when_resolved_with_base_returns_base()
    -> Result<(), RenderError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff3_conflict();
        assert!(conflict.base.is_some());
        conflict.resolution = Some(Resolution::Base);
        let renderd = render_conflict(&conflict)?;
        assert_eq!(renderd, conflict.base.unwrap().lines);
        Ok(())
    }

    #[test]
    fn render_conflict_on_diff3_block_when_resolved_with_theirs_returs_theirs()
    -> Result<(), RenderError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff3_conflict();
        conflict.resolution = Some(Resolution::Theirs);
        let renderd = render_conflict(&conflict)?;
        assert_eq!(renderd, conflict.theirs.lines);
        Ok(())
    }

    #[test]
    fn render_merge_file_with_no_blocks_returns_empty() -> Result<(), RenderError> {
        let merge_file = MergeFile { blocks: Vec::new() };
        let renderd = render_merge_file(&merge_file)?;
        assert!(renderd.is_empty());
        Ok(())
    }

    #[test]
    fn render_merge_file_with_single_regular_block_produces_expected() -> Result<(), RenderError> {
        let regular_block_lines = test_helpers::make_regular_block();
        let merge_file = MergeFile {
            blocks: vec![Block::Regular(regular_block_lines.clone())],
        };
        let renderd = render_merge_file(&merge_file)?;
        assert_eq!(renderd, regular_block_lines);
        Ok(())
    }

    #[test]
    fn render_merge_file_with_mixed_unresolved_blocks_produces_expected() -> Result<(), RenderError>
    {
        let test_helpers::TestMergeFile {
            raw_lines: expected_renderd,
            parsed: merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Diff2,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);
        let renderd = render_merge_file(&merge_file)?;
        assert_eq!(renderd, expected_renderd);
        Ok(())
    }

    #[test]
    fn render_merge_file_with_mixed_resolved_and_unresolved_blocks_produces_expected()
    -> Result<(), RenderError> {
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
            conflict.resolution = Some(Resolution::Ours);
            expected_renderd.extend(conflict.ours.lines.clone());
        } else {
            panic!("Expected first block to be conflict block");
        }

        if let Block::Regular(lines) = &merge_file.blocks[1] {
            expected_renderd.extend(lines.clone());
        } else {
            panic!("Expected second block to be regular block");
        }

        if let Block::Conflict(conflict) = &merge_file.blocks[2] {
            expected_renderd.extend(render_conflict(conflict)?);
        } else {
            panic!("Expected third block to be conflict block");
        }

        let renderd = render_merge_file(&merge_file)?;
        assert_eq!(renderd, expected_renderd);

        Ok(())
    }

    #[test]
    fn render_merge_file_with_all_resolved_and_unresolved_blocks_produces_expected()
    -> Result<(), RenderError> {
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
            conflict.resolution = Some(Resolution::Base);
            if let Some(base) = &conflict.base {
                expected_renderd.extend(base.lines.clone());
            } else {
                panic!("Expected third block to be diff3 block");
            }
        } else {
            panic!("Expected third block to be conflict block");
        }

        let renderd = render_merge_file(&merge_file)?;
        assert_eq!(renderd, expected_renderd);

        Ok(())
    }

    #[test]
    fn render_merge_file_with_invalid_resolution_for_diff2_block_returns_error() {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: mut merge_file,
        } = test_helpers::make_test_merge_file(vec![
            test_helpers::BlockType::Diff2,
            test_helpers::BlockType::Regular,
            test_helpers::BlockType::Diff3,
        ]);

        if let Block::Conflict(conflict) = &mut merge_file.blocks[0] {
            assert!(conflict.base.is_none());
            conflict.resolution = Some(Resolution::Base);
        } else {
            panic!("Expected first block to be conflict block");
        }

        let renderd = render_merge_file(&merge_file);
        assert!(renderd.is_err());
    }
}
