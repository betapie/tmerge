#![allow(dead_code)]
use crate::core::model::{Block, Conflict, ConflictSegment, MergeFile};

pub fn make_regular_block() -> Vec<String> {
    vec![
        String::from("Some regular"),
        String::from("  file, without   "),
        String::from("any confl"),
        String::from("icts"),
    ]
}

pub struct TestConflict {
    pub raw_lines: Vec<String>,
    pub parsed: Conflict,
}

pub fn make_diff2_conflict() -> TestConflict {
    let raw_lines = vec![
        String::from("<<<<<<< yours:some_file.txt"),
        String::from("  this would be"),
        String::from("ours here"),
        String::from("======="),
        String::from(" and this is"),
        String::from("theirs"),
        String::from(">>>>>>> theirs:some_file.txt"),
    ];
    let parsed = Conflict {
        ours: ConflictSegment {
            tag: Some(String::from("yours:some_file.txt")),
            lines: vec![String::from("  this would be"), String::from("ours here")],
        },
        base: None,
        theirs: ConflictSegment {
            tag: Some(String::from("theirs:some_file.txt")),
            lines: vec![String::from(" and this is"), String::from("theirs")],
        },
        resolution: None,
    };
    TestConflict { raw_lines, parsed }
}

pub fn make_diff3_conflict() -> TestConflict {
    let raw_lines = vec![
        String::from("<<<<<<< yours:some_file.txt"),
        String::from("  this would be"),
        String::from("ours here"),
        String::from("||||||| base:some_file.txt"),
        String::from("This is base"),
        String::from("======="),
        String::from(" and this is"),
        String::from("theirs"),
        String::from(">>>>>>> theirs:some_file.txt"),
    ];
    let parsed = Conflict {
        ours: ConflictSegment {
            tag: Some(String::from("yours:some_file.txt")),
            lines: vec![String::from("  this would be"), String::from("ours here")],
        },
        base: Some(ConflictSegment {
            tag: Some(String::from("base:some_file.txt")),
            lines: vec![String::from("This is base")],
        }),
        theirs: ConflictSegment {
            tag: Some(String::from("theirs:some_file.txt")),
            lines: vec![String::from(" and this is"), String::from("theirs")],
        },
        resolution: None,
    };
    TestConflict { raw_lines, parsed }
}

pub enum BlockType {
    Regular,
    Diff2,
    Diff3,
}

pub struct TestBlock {
    pub raw_lines: Vec<String>,
    pub block: Block,
}

pub fn make_test_block(block_type: BlockType) -> TestBlock {
    match block_type {
        BlockType::Regular => {
            let lines = make_regular_block();
            TestBlock {
                raw_lines: lines.clone(),
                block: Block::Regular(lines),
            }
        }
        BlockType::Diff2 => {
            let TestConflict {
                raw_lines,
                parsed: conflict,
            } = make_diff2_conflict();
            TestBlock {
                raw_lines,
                block: Block::Conflict(conflict),
            }
        }
        BlockType::Diff3 => {
            let TestConflict {
                raw_lines,
                parsed: conflict,
            } = make_diff3_conflict();
            TestBlock {
                raw_lines,
                block: Block::Conflict(conflict),
            }
        }
    }
}

pub struct TestMergeFile {
    pub raw_lines: Vec<String>,
    pub parsed: MergeFile,
}

pub fn make_test_merge_file(block_types: Vec<BlockType>) -> TestMergeFile {
    let mut raw_lines = Vec::new();
    let mut parsed_blocks = Vec::new();

    for block_type in block_types {
        let TestBlock {
            raw_lines: block_raw_lines,
            block,
        } = make_test_block(block_type);
        raw_lines.extend(block_raw_lines);
        parsed_blocks.push(block);
    }

    TestMergeFile {
        raw_lines,
        parsed: MergeFile {
            blocks: parsed_blocks,
        },
    }
}
