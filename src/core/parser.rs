use crate::core::constants::markers;
use crate::core::model::{Block, Conflict, ConflictSegment, MergeFile};
use std::fmt;

enum ParsedLine {
    OursBegin(Option<String>),
    BaseBegin(Option<String>),
    TheirsBegin,
    ConflictEnd(Option<String>),
    Plain(String),
}

impl ParsedLine {
    fn from_str(line: String) -> Self {
        let get_tag = |line: String| -> Option<String> {
            if line.len() > markers::OURS_BEGIN.len() + 1 {
                Some(line[&markers::OURS_BEGIN.len() + 1..].to_string())
            } else {
                None
            }
        };
        if line.starts_with(markers::OURS_BEGIN) {
            ParsedLine::OursBegin(get_tag(line))
        } else if line.starts_with(markers::BASE_BEGIN) {
            ParsedLine::BaseBegin(get_tag(line))
        } else if line.starts_with(markers::THEIRS_BEGIN) {
            ParsedLine::TheirsBegin
        } else if line.starts_with(markers::CONFLICT_END) {
            ParsedLine::ConflictEnd(get_tag(line))
        } else {
            ParsedLine::Plain(line)
        }
    }

    fn into_str(self) -> String {
        match self {
            ParsedLine::OursBegin(tag) => {
                if let Some(tag) = tag {
                    format!("{} {}", markers::OURS_BEGIN, tag)
                } else {
                    markers::OURS_BEGIN.into()
                }
            }
            ParsedLine::BaseBegin(tag) => {
                if let Some(tag) = tag {
                    format!("{} {}", markers::BASE_BEGIN, tag)
                } else {
                    markers::BASE_BEGIN.into()
                }
            }
            ParsedLine::TheirsBegin => markers::THEIRS_BEGIN.into(),
            ParsedLine::ConflictEnd(tag) => {
                if let Some(tag) = tag {
                    format!("{} {}", markers::CONFLICT_END, tag)
                } else {
                    markers::CONFLICT_END.into()
                }
            }
            ParsedLine::Plain(line) => line,
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

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    fn new(message: String) -> ParseError {
        ParseError { message }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

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

    pub fn consume(mut self, line: String) -> Result<Self, ParseError> {
        let parsed_line = ParsedLine::from_str(line);
        let current_state = std::mem::replace(&mut self.state, ParseState::Regular(Vec::new()));

        let (new_state, added_block) = match current_state {
            ParseState::Regular(current_lines) => {
                consume_line_state_regular(current_lines, parsed_line)?
            }
            ParseState::ParsingOurs(conflict_builder) => {
                consume_line_state_parsing_ours(conflict_builder, parsed_line)?
            }
            ParseState::ParsingBase(conflict_builder) => {
                consume_line_state_parsing_base(conflict_builder, parsed_line)?
            }
            ParseState::ParsingTheirs(conflict_builder) => {
                consume_line_state_parsing_theirs(conflict_builder, parsed_line)?
            }
        };

        if let Some(added_block) = added_block {
            self.blocks.push(added_block);
        }
        self.state = new_state;

        Ok(self)
    }

    pub fn into_merge_file(self) -> Result<MergeFile, ParseError> {
        let (state, mut blocks) = (self.state, self.blocks);
        match state {
            ParseState::Regular(lines) => {
                if !lines.is_empty() {
                    blocks.push(Block::Regular(lines));
                }
            }
            _ => {
                return Err(ParseError::new(String::from("Still in conflict block")));
            }
        }
        Ok(MergeFile { blocks })
    }
}

fn consume_line_state_regular(
    current_lines: Vec<String>,
    parsed_line: ParsedLine,
) -> Result<(ParseState, Option<Block>), ParseError> {
    match parsed_line {
        ParsedLine::OursBegin(tag) => {
            let created_block =
                (!current_lines.is_empty()).then_some(Block::Regular(current_lines));
            let mut cb = ConflictBuilder::new_empty();
            cb.ours.tag = tag;
            Ok((ParseState::ParsingOurs(cb), created_block))
        }
        ParsedLine::Plain(line) => {
            let mut lines = current_lines;
            lines.push(line);
            Ok((ParseState::Regular(lines), None))
        }
        _ => Err(ParseError::new(format!(
            "Unexpected marker outside of conflict: {}",
            parsed_line.into_str()
        ))),
    }
}

fn consume_line_state_parsing_ours(
    conflict_builder: ConflictBuilder,
    parsed_line: ParsedLine,
) -> Result<(ParseState, Option<Block>), ParseError> {
    match parsed_line {
        ParsedLine::BaseBegin(tag) => {
            let mut conflict_builder = conflict_builder;
            conflict_builder.base = Some(ConflictSegment {
                tag,
                lines: Vec::new(),
            });
            Ok((ParseState::ParsingBase(conflict_builder), None))
        }
        ParsedLine::TheirsBegin => Ok((ParseState::ParsingTheirs(conflict_builder), None)),
        ParsedLine::Plain(line) => {
            let mut conflict_builder = conflict_builder;
            conflict_builder.ours.lines.push(line);
            Ok((ParseState::ParsingOurs(conflict_builder), None))
        }
        _ => Err(ParseError::new(format!(
            "Unexpected marker in OURS section: {}",
            parsed_line.into_str()
        ))),
    }
}

fn consume_line_state_parsing_base(
    conflict_builder: ConflictBuilder,
    parsed_line: ParsedLine,
) -> Result<(ParseState, Option<Block>), ParseError> {
    match parsed_line {
        ParsedLine::TheirsBegin => Ok((ParseState::ParsingTheirs(conflict_builder), None)),
        ParsedLine::Plain(line) => {
            let mut conflict_builder = conflict_builder;
            conflict_builder.base.as_mut().unwrap().lines.push(line);
            Ok((ParseState::ParsingBase(conflict_builder), None))
        }
        _ => Err(ParseError::new(format!(
            "Unexpected marker in BASE section: {}",
            parsed_line.into_str()
        ))),
    }
}

fn consume_line_state_parsing_theirs(
    conflict_builder: ConflictBuilder,
    parsed_line: ParsedLine,
) -> Result<(ParseState, Option<Block>), ParseError> {
    match parsed_line {
        ParsedLine::ConflictEnd(tag) => {
            let mut conflict_builder = conflict_builder;
            conflict_builder.theirs.tag = tag;
            Ok((
                ParseState::Regular(Vec::new()),
                Some(Block::Conflict(conflict_builder.build())),
            ))
        }
        ParsedLine::Plain(line) => {
            let mut conflict_builder = conflict_builder;
            conflict_builder.theirs.lines.push(line);
            Ok((ParseState::ParsingTheirs(conflict_builder), None))
        }
        _ => Err(ParseError::new(format!(
            "Unexpected marker in THEIRS section: {}",
            parsed_line.into_str()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        model::Block,
        test_helpers::{self},
    };

    use super::*;

    struct TestBlock {
        input_lines: Vec<String>,
        expected_parsed: Block,
    }

    fn make_regular_test_block() -> TestBlock {
        let input_lines = test_helpers::make_regular_block();
        let expected_parsed = Block::Regular(input_lines.clone());
        TestBlock {
            input_lines,
            expected_parsed,
        }
    }

    fn make_diff2_conflict_test_block() -> TestBlock {
        let test_helpers::TestConflict { raw_lines, parsed } = test_helpers::make_diff2_conflict();
        TestBlock {
            input_lines: raw_lines,
            expected_parsed: Block::Conflict(parsed),
        }
    }

    fn make_diff3_conflict_test_block() -> TestBlock {
        let test_helpers::TestConflict { raw_lines, parsed } = test_helpers::make_diff3_conflict();
        TestBlock {
            input_lines: raw_lines,
            expected_parsed: Block::Conflict(parsed),
        }
    }

    #[test]
    fn into_merge_file_on_new_parser_returns_file_with_no_blocks() -> Result<(), ParseError> {
        let parser = Parser::new();
        let merge_file = parser.into_merge_file()?;
        assert!(merge_file.blocks.is_empty());
        Ok(())
    }

    #[test]
    fn parse_on_input_without_conflicts_creates_file_with_single_regular_block()
    -> Result<(), ParseError> {
        let TestBlock {
            input_lines,
            expected_parsed,
        } = make_regular_test_block();
        let mut parser = Parser::new();
        for line in input_lines {
            parser = parser.consume(line.to_string())?;
        }

        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), 1);
        assert_eq!(merge_file.blocks[0], expected_parsed);

        Ok(())
    }

    #[test]
    fn parse_on_input_with_single_conflict_creates_file_with_single_conflict_block()
    -> Result<(), ParseError> {
        let TestBlock {
            input_lines,
            expected_parsed,
        } = make_diff2_conflict_test_block();
        let mut parser = Parser::new();
        for line in input_lines {
            parser = parser.consume(line)?;
        }

        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), 1);
        assert_eq!(merge_file.blocks[0], expected_parsed);

        Ok(())
    }

    #[test]
    fn parse_on_input_with_single_diff3_conflict_creates_file_with_single_conflict_block()
    -> Result<(), ParseError> {
        let TestBlock {
            input_lines,
            expected_parsed,
        } = make_diff3_conflict_test_block();
        let mut parser = Parser::new();
        for line in input_lines {
            parser = parser.consume(line)?;
        }

        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), 1);
        assert_eq!(merge_file.blocks[0], expected_parsed);

        Ok(())
    }

    #[test]
    fn consume_on_invalid_conflict_block_returns_error() -> Result<(), ParseError> {
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
            parser = parser.consume(line)?;
        }
        assert!(parser.consume(markers::BASE_BEGIN.into()).is_err());
        Ok(())
    }

    #[test]
    fn into_merge_file_with_unfinished_conflict_block_returns_error() -> Result<(), ParseError> {
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
            parser = parser.consume(line)?;
        }
        assert!(parser.into_merge_file().is_err());
        Ok(())
    }

    #[test]
    fn parse_lines_with_regular_then_conflict_block_produces_expected() -> Result<(), ParseError> {
        let mut input_lines = Vec::new();
        let mut expected_blocks = Vec::new();

        let regular_block = make_regular_test_block();
        input_lines.extend(regular_block.input_lines);
        expected_blocks.push(regular_block.expected_parsed);

        let conflict_block = make_diff2_conflict_test_block();
        input_lines.extend(conflict_block.input_lines);
        expected_blocks.push(conflict_block.expected_parsed);

        let mut parser = Parser::new();
        for line in input_lines {
            parser = parser.consume(line)?;
        }
        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), expected_blocks.len());
        for (block, expected_block) in merge_file.blocks.into_iter().zip(expected_blocks) {
            assert_eq!(block, expected_block);
        }
        Ok(())
    }

    #[test]
    fn parse_lines_with_conflict_then_regular_block_produces_expected() -> Result<(), ParseError> {
        let mut input_lines = Vec::new();
        let mut expected_blocks = Vec::new();

        let conflict_block = make_diff3_conflict_test_block();
        input_lines.extend(conflict_block.input_lines);
        expected_blocks.push(conflict_block.expected_parsed);

        let regular_block = make_regular_test_block();
        input_lines.extend(regular_block.input_lines);
        expected_blocks.push(regular_block.expected_parsed);

        let mut parser = Parser::new();
        for line in input_lines {
            parser = parser.consume(line)?;
        }
        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), expected_blocks.len());
        for (block, expected_block) in merge_file.blocks.into_iter().zip(expected_blocks) {
            assert_eq!(block, expected_block);
        }
        Ok(())
    }

    #[test]
    fn parse_lines_with_two_consecutive_conflict_blocks_produces_expected()
    -> Result<(), ParseError> {
        let mut input_lines = Vec::new();
        let mut expected_blocks = Vec::new();

        let conflict_block = make_diff2_conflict_test_block();
        input_lines.extend(conflict_block.input_lines);
        expected_blocks.push(conflict_block.expected_parsed);

        let conflict_block = make_diff3_conflict_test_block();
        input_lines.extend(conflict_block.input_lines);
        expected_blocks.push(conflict_block.expected_parsed);

        let mut parser = Parser::new();
        for line in input_lines {
            parser = parser.consume(line)?;
        }
        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), expected_blocks.len());
        for (block, expected_block) in merge_file.blocks.into_iter().zip(expected_blocks) {
            assert_eq!(block, expected_block);
        }
        Ok(())
    }

    #[test]
    fn parse_lines_with_mixed_blocks_produces_expected() -> Result<(), ParseError> {
        let test_helpers::TestMergeFile {
            raw_lines: input_lines,
            parsed: expected_parsed,
        } = test_helpers::make_mixed_test_merge_file();

        let mut parser = Parser::new();
        for line in input_lines {
            parser = parser.consume(line)?;
        }
        let merge_file = parser.into_merge_file()?;

        assert_eq!(merge_file.blocks.len(), expected_parsed.blocks.len());
        for (parsed_block, expected_block) in merge_file
            .blocks
            .into_iter()
            .zip(expected_parsed.blocks.into_iter())
        {
            assert_eq!(parsed_block, expected_block);
        }

        Ok(())
    }
}
