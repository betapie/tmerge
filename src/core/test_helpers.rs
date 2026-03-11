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

pub struct TestMergeFile {
    pub raw_lines: Vec<String>,
    pub parsed: MergeFile,
}

pub fn make_mixed_test_merge_file() -> TestMergeFile {
    let mut raw_lines = Vec::new();
    let mut parsed_blocks = Vec::new();

    {
        let TestConflict {
            raw_lines: conflict_raw_lines,
            parsed: parsed_conflict,
        } = make_diff2_conflict();
        raw_lines.extend(conflict_raw_lines);
        parsed_blocks.push(Block::Conflict(parsed_conflict));
    }
    {
        let regular_block_lines = make_regular_block();
        raw_lines.extend(regular_block_lines.clone());
        parsed_blocks.push(Block::Regular(regular_block_lines));
    }
    {
        let TestConflict {
            raw_lines: conflict_raw_lines,
            parsed: parsed_conflict,
        } = make_diff3_conflict();
        raw_lines.extend(conflict_raw_lines);
        parsed_blocks.push(Block::Conflict(parsed_conflict));
    }

    TestMergeFile {
        raw_lines,
        parsed: MergeFile {
            blocks: parsed_blocks,
        },
    }
}
