#![allow(dead_code)]
use crate::core::model::{Conflict, ConflictSegment};

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
