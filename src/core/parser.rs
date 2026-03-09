use crate::core::{
    model::{Block, Conflict, ConflictSegment, MergeFile},
    parser::markers::OURS_BEGIN,
};

mod markers {
    pub const OURS_BEGIN: &str = "<<<<<<<";
    pub const BASE_BEGIN: &str = "|||||||";
    pub const THEIRS_BEGIN: &str = "=======";
    pub const CONFLICT_END: &str = ">>>>>>>";
}

enum Marker {
    OursBegin(Option<String>),
    BaseBegin(Option<String>),
    TheirsBegin,
    ConflictEnd(Option<String>),
    None,
}

impl Marker {
    fn from_str(line: &str) -> Self {
        let get_tag = |line: &str| -> Option<String> {
            if line.len() > OURS_BEGIN.len() + 1 {
                Some(line[&OURS_BEGIN.len() + 1..].to_string())
            } else {
                None
            }
        };
        if line.starts_with(markers::OURS_BEGIN) {
            Marker::OursBegin(get_tag(line))
        } else if line.starts_with(markers::BASE_BEGIN) {
            Marker::BaseBegin(get_tag(line))
        } else if line.starts_with(markers::THEIRS_BEGIN) {
            Marker::TheirsBegin
        } else if line.starts_with(markers::CONFLICT_END) {
            Marker::ConflictEnd(get_tag(line))
        } else {
            Marker::None
        }
    }
}

struct ConflictBuilder {
    ours: ConflictSegment,
    base: Option<ConflictSegment>,
    theirs: ConflictSegment,
}

impl ConflictBuilder {
    fn new_empty() -> ConflictBuilder {
        ConflictBuilder {
            ours: ConflictSegment {
                tag: None,
                lines: Vec::new(),
            },
            base: None,
            theirs: ConflictSegment {
                tag: None,
                lines: Vec::new(),
            },
        }
    }
}

impl ConflictBuilder {
    fn build(self) -> Conflict {
        Conflict {
            ours: self.ours,
            base: self.base,
            theirs: self.theirs,
            resolution: None,
        }
    }
}

enum ParseState {
    Regular(Vec<String>),
    ParsingOurs(ConflictBuilder),
    ParsingBase(ConflictBuilder),
    ParsingTheirs(ConflictBuilder),
}

pub struct Parser {
    blocks: Vec<Block>,
    state: ParseState,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            state: ParseState::Regular(Vec::new()),
        }
    }

    pub fn consume(&mut self, line: String) -> Result<(), String> {
        self.state = match (
            std::mem::replace(&mut self.state, ParseState::Regular(Vec::new())),
            Marker::from_str(&line),
        ) {
            (ParseState::Regular(lines), Marker::OursBegin(tag)) => {
                if !lines.is_empty() {
                    self.blocks.push(Block::Regular(lines));
                }
                let mut conflict_builder = ConflictBuilder::new_empty();
                conflict_builder.ours.tag = tag;
                ParseState::ParsingOurs(conflict_builder)
            }
            (ParseState::Regular(mut lines), Marker::None) => {
                lines.push(line);
                ParseState::Regular(lines)
            }
            (ParseState::Regular(_), _) => {
                return Err(format!("Unexpected marker outside conflict: {}", line));
            }

            (ParseState::ParsingOurs(mut cb), Marker::BaseBegin(tag)) => {
                cb.base = Some(ConflictSegment {
                    tag,
                    lines: Vec::new(),
                });
                ParseState::ParsingBase(cb)
            }
            (ParseState::ParsingOurs(cb), Marker::TheirsBegin) => ParseState::ParsingTheirs(cb),
            (ParseState::ParsingOurs(mut cb), Marker::None) => {
                cb.ours.lines.push(line);
                ParseState::ParsingOurs(cb)
            }
            (ParseState::ParsingOurs(_), _) => {
                return Err(format!("Unexpected marker in ours section: {}", line));
            }

            (ParseState::ParsingBase(cb), Marker::TheirsBegin) => ParseState::ParsingTheirs(cb),
            (ParseState::ParsingBase(mut cb), Marker::None) => {
                cb.base.as_mut().unwrap().lines.push(line);
                ParseState::ParsingBase(cb)
            }
            (ParseState::ParsingBase(_), _) => {
                return Err(format!("Unexpected marker in base section: {}", line));
            }

            (ParseState::ParsingTheirs(mut cb), Marker::ConflictEnd(tag)) => {
                assert!(cb.theirs.tag.is_none());
                cb.theirs.tag = tag;
                self.blocks.push(Block::Conflict(cb.build()));
                ParseState::Regular(Vec::new())
            }
            (ParseState::ParsingTheirs(mut cb), Marker::None) => {
                cb.theirs.lines.push(line);
                ParseState::ParsingTheirs(cb)
            }
            (ParseState::ParsingTheirs(_), _) => {
                return Err(format!("Unexpected marker in theirs section: {}", line));
            }
        };
        Ok(())
    }

    pub fn into_merge_file(self) -> Result<MergeFile, String> {
        let (state, mut blocks) = (self.state, self.blocks);
        match state {
            ParseState::Regular(lines) => {
                if !lines.is_empty() {
                    blocks.push(Block::Regular(lines));
                }
            }
            _ => {
                return Err(String::from("Still in conflict block"));
            }
        }
        Ok(MergeFile { blocks })
    }
}

#[cfg(test)]
mod tests {
    use crate::core::model::Block;

    use super::*;

    struct TestBlock {
        input_lines: Vec<String>,
        expected_parsed: Option<Block>,
    }

    fn make_regular_test_block() -> TestBlock {
        let input_lines = vec![
            String::from("Some regular"),
            String::from("  file, without   "),
            String::from("any confl"),
            String::from("icts"),
        ];
        let expected_parsed = Some(Block::Regular(input_lines.clone()));
        TestBlock {
            input_lines,
            expected_parsed,
        }
    }

    fn make_diff2_conflict_test_block() -> TestBlock {
        let input_lines = vec![
            String::from("<<<<<<< yours:some_file.txt"),
            String::from("  this would be"),
            String::from("ours here"),
            String::from("======="),
            String::from(" and this is"),
            String::from("theirs"),
            String::from(">>>>>>> theirs:some_file.txt"),
        ];
        let expected_parsed = Some(Block::Conflict(Conflict {
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
        }));
        TestBlock {
            input_lines,
            expected_parsed,
        }
    }

    fn make_diff3_conflict_test_block() -> TestBlock {
        let input_lines = vec![
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
        let expected_parsed = Some(Block::Conflict(Conflict {
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
        }));
        TestBlock {
            input_lines,
            expected_parsed,
        }
    }

    #[test]
    fn into_merge_file_on_new_parser_returns_file_with_no_blocks() -> Result<(), String> {
        let parser = Parser::new();
        let merge_file = parser.into_merge_file()?;
        assert!(merge_file.blocks.is_empty());
        Ok(())
    }

    #[test]
    fn parse_on_input_without_conflicts_creates_file_with_single_regular_block()
    -> Result<(), String> {
        let TestBlock {
            input_lines,
            expected_parsed,
        } = make_regular_test_block();
        let mut parser = Parser::new();
        for line in input_lines {
            parser.consume(line.to_string())?;
        }

        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), 1);
        assert_eq!(merge_file.blocks[0], expected_parsed.unwrap());

        Ok(())
    }

    #[test]
    fn parse_on_input_with_single_conflict_creates_file_with_single_conflict_block()
    -> Result<(), String> {
        let TestBlock {
            input_lines,
            expected_parsed,
        } = make_diff2_conflict_test_block();
        let mut parser = Parser::new();
        for line in input_lines {
            parser.consume(line)?;
        }

        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), 1);
        assert_eq!(merge_file.blocks[0], expected_parsed.unwrap());

        Ok(())
    }

    #[test]
    fn parse_on_input_with_single_diff3_conflict_creates_file_with_single_conflict_block()
    -> Result<(), String> {
        let TestBlock {
            input_lines,
            expected_parsed,
        } = make_diff3_conflict_test_block();
        let mut parser = Parser::new();
        for line in input_lines {
            parser.consume(line)?;
        }

        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), 1);
        assert_eq!(merge_file.blocks[0], expected_parsed.unwrap());

        Ok(())
    }

    #[test]
    fn consume_on_invalid_conflict_block_returns_error() -> Result<(), String> {
        let input = [
            String::from("<<<<<<< yours:some_file.txt"),
            String::from("  this would be"),
            String::from("ours here"),
            String::from("======="),
            String::from(" and this is"),
            String::from("theirs"),
        ];
        let mut parser = Parser::new();
        for line in input {
            parser.consume(line)?;
        }
        assert!(parser.consume(markers::BASE_BEGIN.into()).is_err());
        Ok(())
    }

    #[test]
    fn into_merge_file_with_unfinished_conflict_block_returns_error() -> Result<(), String> {
        let input = [
            String::from("<<<<<<< yours:some_file.txt"),
            String::from("  this would be"),
            String::from("ours here"),
            String::from("======="),
            String::from(" and this is"),
            String::from("theirs"),
        ];
        let mut parser = Parser::new();
        for line in input {
            parser.consume(line)?;
        }
        assert!(parser.into_merge_file().is_err());
        Ok(())
    }

    #[test]
    fn parse_lines_with_regular_then_conflict_block_produces_expected() -> Result<(), String> {
        let mut input_lines = Vec::new();
        let mut expected_blocks = Vec::new();

        let regular_block = make_regular_test_block();
        input_lines.extend(regular_block.input_lines);
        expected_blocks.extend(regular_block.expected_parsed);

        let conflict_block = make_diff2_conflict_test_block();
        input_lines.extend(conflict_block.input_lines);
        expected_blocks.extend(conflict_block.expected_parsed);

        let mut parser = Parser::new();
        for line in input_lines {
            parser.consume(line)?;
        }
        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), expected_blocks.len());
        for (block, expected_block) in merge_file.blocks.into_iter().zip(expected_blocks) {
            assert_eq!(block, expected_block);
        }
        Ok(())
    }

    #[test]
    fn parse_lines_with_conflict_then_regular_block_produces_expected() -> Result<(), String> {
        let mut input_lines = Vec::new();
        let mut expected_blocks = Vec::new();

        let conflict_block = make_diff3_conflict_test_block();
        input_lines.extend(conflict_block.input_lines);
        expected_blocks.extend(conflict_block.expected_parsed);

        let regular_block = make_regular_test_block();
        input_lines.extend(regular_block.input_lines);
        expected_blocks.extend(regular_block.expected_parsed);

        let mut parser = Parser::new();
        for line in input_lines {
            parser.consume(line)?;
        }
        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), expected_blocks.len());
        for (block, expected_block) in merge_file.blocks.into_iter().zip(expected_blocks) {
            assert_eq!(block, expected_block);
        }
        Ok(())
    }

    #[test]
    fn parse_lines_with_two_consecutive_conflict_blocks_produces_expected() -> Result<(), String> {
        let mut input_lines = Vec::new();
        let mut expected_blocks = Vec::new();

        let conflict_block = make_diff2_conflict_test_block();
        input_lines.extend(conflict_block.input_lines);
        expected_blocks.extend(conflict_block.expected_parsed);

        let conflict_block = make_diff3_conflict_test_block();
        input_lines.extend(conflict_block.input_lines);
        expected_blocks.extend(conflict_block.expected_parsed);

        let mut parser = Parser::new();
        for line in input_lines {
            parser.consume(line)?;
        }
        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), expected_blocks.len());
        for (block, expected_block) in merge_file.blocks.into_iter().zip(expected_blocks) {
            assert_eq!(block, expected_block);
        }
        Ok(())
    }

    #[test]
    fn parse_lines_with_mixed_blocks_produces_expected() -> Result<(), String> {
        let input_lines = vec![
            String::from("This is a regular block"),
            String::from("<<<<<<< HEAD"),
            String::from("ours line 1"),
            String::from("ours line 2"),
            String::from("======="),
            String::from("theirs line 1"),
            String::from(">>>>>>> feature-branch"),
            String::from("Another regular block between conflicts"),
            String::from("<<<<<<< HEAD"),
            String::from("only ours"),
            String::from("||||||| base"),
            String::from("base content"),
            String::from("======="),
            String::from("only theirs"),
            String::from(">>>>>>> feature-branch"),
            String::from("Trailing regular block"),
        ];
        let expected_blocks = vec![
            Block::Regular(vec![String::from("This is a regular block")]),
            Block::Conflict(Conflict {
                ours: ConflictSegment {
                    tag: Some(String::from("HEAD")),
                    lines: vec![String::from("ours line 1"), String::from("ours line 2")],
                },
                base: None,
                theirs: ConflictSegment {
                    tag: Some(String::from("feature-branch")),

                    lines: vec![String::from("theirs line 1")],
                },
                resolution: None,
            }),
            Block::Regular(vec![String::from(
                "Another regular block between conflicts",
            )]),
            Block::Conflict(Conflict {
                ours: ConflictSegment {
                    tag: Some(String::from("HEAD")),
                    lines: vec![String::from("only ours")],
                },
                base: Some(ConflictSegment {
                    tag: Some(String::from("base")),
                    lines: vec![String::from("base content")],
                }),
                theirs: ConflictSegment {
                    tag: Some(String::from("feature-branch")),
                    lines: vec![String::from("only theirs")],
                },
                resolution: None,
            }),
            Block::Regular(vec![String::from("Trailing regular block")]),
        ];

        let mut parser = Parser::new();
        for line in input_lines {
            parser.consume(line)?;
        }
        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), expected_blocks.len());
        for (parsed_block, expected_block) in merge_file
            .blocks
            .into_iter()
            .zip(expected_blocks.into_iter())
        {
            assert_eq!(parsed_block, expected_block);
        }

        Ok(())
    }
}
