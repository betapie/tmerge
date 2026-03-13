mod core;
mod app;

use core::parser::Parser;
use std::io::BufRead;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let filename = &args[1];
    let f = std::fs::File::open(filename).unwrap();
    let reader = std::io::BufReader::new(f);
    let mut parser = Parser::new();
    for line in reader.lines() {
        parser = parser.consume(line.unwrap()).unwrap();
    }
    let merge_file = parser.into_merge_file().unwrap();
    let mut num_conflict_blocks = 0;
    for block in &merge_file.blocks {
        match block {
            core::model::Block::Regular(_) => {}
            core::model::Block::Conflict(_) => num_conflict_blocks += 1,
        }
    }
    if num_conflict_blocks == 0 {
        println!("No conflicts found in file {}", filename);
    } else {
        println!(
            "Found {} conflicts in file {}",
            num_conflict_blocks, filename
        );
    }
}
