use std::fmt;

use crate::core::constants::markers;
use crate::core::model::{Block, Conflict, MergeFile, Resolution};

#[derive(Debug)]
pub struct UnparseError {
    pub message: String,
}

impl UnparseError {
    fn new(message: String) -> UnparseError {
        UnparseError { message }
    }
}

impl fmt::Display for UnparseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unparse error: {}", self.message)
    }
}

impl std::error::Error for UnparseError {}

pub fn unparse_merge_file(merge_file: &MergeFile) -> Result<Vec<String>, UnparseError> {
    let mut result = Vec::new();
    for block in &merge_file.blocks {
        match block {
            Block::Regular(lines) => {
                result.extend_from_slice(lines);
            }
            Block::Conflict(conflict) => {
                result.extend(unparse_conflict(conflict)?);
            }
        }
    }
    Ok(result)
}

pub fn unparse_conflict(conflict: &Conflict) -> Result<Vec<String>, UnparseError> {
    let result_lines = match &conflict.resolution {
        Some(resolution) => match resolution {
            Resolution::Ours => conflict.ours.lines.clone(),
            Resolution::Theirs => conflict.theirs.lines.clone(),
            Resolution::Base => {
                if let Some(base) = &conflict.base {
                    base.lines.clone()
                } else {
                    return Err(UnparseError::new(String::from(
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
    fn unparse_conflict_on_unresolved_diff2_block_returns_raw() -> Result<(), UnparseError> {
        let test_helpers::TestConflict {
            raw_lines,
            parsed: conflict,
        } = test_helpers::make_diff2_conflict();
        assert!(conflict.resolution.is_none());
        let unparsed = unparse_conflict(&conflict)?;
        assert_eq!(unparsed, raw_lines);
        Ok(())
    }

    #[test]
    fn unparse_on_diff2_block_when_resolved_with_ours_returns_ours() -> Result<(), UnparseError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff2_conflict();
        conflict.resolution = Some(Resolution::Ours);
        let unparsed = unparse_conflict(&conflict)?;
        assert_eq!(unparsed, conflict.ours.lines);
        Ok(())
    }

    #[test]
    fn unparse_conflict_on_diff2_block_when_resolved_with_base_returns_error() {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff2_conflict();
        conflict.resolution = Some(Resolution::Base);
        let unparsed = unparse_conflict(&conflict);
        assert!(unparsed.is_err());
    }

    #[test]
    fn unparse_conflict_on_diff2_block_when_resolved_with_theirs_returs_theirs()
    -> Result<(), UnparseError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff2_conflict();
        conflict.resolution = Some(Resolution::Theirs);
        let unparsed = unparse_conflict(&conflict)?;
        assert_eq!(unparsed, conflict.theirs.lines);
        Ok(())
    }

    #[test]
    fn unparse_on_unresolved_diff3_block_returns_raw() -> Result<(), UnparseError> {
        let test_helpers::TestConflict {
            raw_lines,
            parsed: conflict,
        } = test_helpers::make_diff3_conflict();
        assert!(conflict.resolution.is_none());
        let unparsed = unparse_conflict(&conflict)?;
        assert_eq!(unparsed, raw_lines);
        Ok(())
    }

    #[test]
    fn unparse_conflict_on_diff3_block_when_resolved_with_ours_returns_ours()
    -> Result<(), UnparseError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff3_conflict();
        conflict.resolution = Some(Resolution::Ours);
        let unparsed = unparse_conflict(&conflict)?;
        assert_eq!(unparsed, conflict.ours.lines);
        Ok(())
    }

    #[test]
    fn unparse_conflict_on_diff3_block_when_resolved_with_base_returns_base()
    -> Result<(), UnparseError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff3_conflict();
        assert!(conflict.base.is_some());
        conflict.resolution = Some(Resolution::Base);
        let unparsed = unparse_conflict(&conflict)?;
        assert_eq!(unparsed, conflict.base.unwrap().lines);
        Ok(())
    }

    #[test]
    fn unparse_conflict_on_diff3_block_when_resolved_with_theirs_returs_theirs()
    -> Result<(), UnparseError> {
        let test_helpers::TestConflict {
            raw_lines: _,
            parsed: mut conflict,
        } = test_helpers::make_diff3_conflict();
        conflict.resolution = Some(Resolution::Theirs);
        let unparsed = unparse_conflict(&conflict)?;
        assert_eq!(unparsed, conflict.theirs.lines);
        Ok(())
    }

    #[test]
    fn unparse_merge_file_with_no_blocks_returns_empty() -> Result<(), UnparseError> {
        let merge_file = MergeFile { blocks: Vec::new() };
        let unparsed = unparse_merge_file(&merge_file)?;
        assert!(unparsed.is_empty());
        Ok(())
    }

    #[test]
    fn unparse_merge_file_with_single_regular_block_produces_expected() -> Result<(), UnparseError>
    {
        let regular_block_lines = test_helpers::make_regular_block();
        let merge_file = MergeFile {
            blocks: vec![Block::Regular(regular_block_lines.clone())],
        };
        let unparsed = unparse_merge_file(&merge_file)?;
        assert_eq!(unparsed, regular_block_lines);
        Ok(())
    }

    #[test]
    fn unparse_merge_file_with_mixed_unresolved_blocks_produces_expected()
    -> Result<(), UnparseError> {
        let test_helpers::TestMergeFile {
            raw_lines: expected_unparsed,
            parsed: merge_file,
        } = test_helpers::make_mixed_test_merge_file();
        let unparsed = unparse_merge_file(&merge_file)?;
        assert_eq!(unparsed, expected_unparsed);
        Ok(())
    }

    #[test]
    fn unparse_merge_file_with_mixed_resolved_and_unresolved_blocks_produces_expected()
    -> Result<(), UnparseError> {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: mut merge_file,
        } = test_helpers::make_mixed_test_merge_file();

        let mut expected_unparsed = Vec::new();

        if let Block::Conflict(conflict) = &mut merge_file.blocks[0] {
            conflict.resolution = Some(Resolution::Ours);
            expected_unparsed.extend(conflict.ours.lines.clone());
        } else {
            panic!("Expected first block to be conflict block");
        }

        if let Block::Regular(lines) = &merge_file.blocks[1] {
            expected_unparsed.extend(lines.clone());
        } else {
            panic!("Expected second block to be regular block");
        }

        if let Block::Conflict(conflict) = &merge_file.blocks[2] {
            expected_unparsed.extend(unparse_conflict(conflict)?);
        } else {
            panic!("Expected third block to be conflict block");
        }

        let unparsed = unparse_merge_file(&merge_file)?;
        assert_eq!(unparsed, expected_unparsed);

        Ok(())
    }

    #[test]
    fn unparse_merge_file_with_all_resolved_and_unresolved_blocks_produces_expected()
    -> Result<(), UnparseError> {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: mut merge_file,
        } = test_helpers::make_mixed_test_merge_file();

        let mut expected_unparsed = Vec::new();

        if let Block::Conflict(conflict) = &mut merge_file.blocks[0] {
            conflict.resolution = Some(Resolution::Ours);
            expected_unparsed.extend(conflict.ours.lines.clone());
        } else {
            panic!("Expected first block to be conflict block");
        }

        if let Block::Regular(lines) = &merge_file.blocks[1] {
            expected_unparsed.extend(lines.clone());
        } else {
            panic!("Expected second block to be regular block");
        }

        if let Block::Conflict(conflict) = &mut merge_file.blocks[2] {
            conflict.resolution = Some(Resolution::Base);
            if let Some(base) = &conflict.base {
                expected_unparsed.extend(base.lines.clone());
            } else {
                panic!("Expected third block to be diff3 block");
            }
        } else {
            panic!("Expected third block to be conflict block");
        }

        let unparsed = unparse_merge_file(&merge_file)?;
        assert_eq!(unparsed, expected_unparsed);

        Ok(())
    }

    #[test]
    fn unparse_merge_file_with_invalid_resolution_for_diff2_block_returns_error() {
        let test_helpers::TestMergeFile {
            raw_lines: _,
            parsed: mut merge_file,
        } = test_helpers::make_mixed_test_merge_file();

        if let Block::Conflict(conflict) = &mut merge_file.blocks[0] {
            assert!(conflict.base.is_none());
            conflict.resolution = Some(Resolution::Base);
        } else {
            panic!("Expected first block to be conflict block");
        }

        let unparsed = unparse_merge_file(&merge_file);
        assert!(unparsed.is_err());
    }
}
