use crate::document::*;

#[allow(dead_code)]
pub fn debug_at(src: &Vec<Vec<char>>, range: &InlineRange, msg: &str) {
    if let Some(line) = src.iter().nth(range.line) {
        let content: String = line[range.start_column..range.end_column].iter().collect();
        eprintln!(
            "{} at ({}:{}-{}:{}) | {}",
            msg, range.line, range.start_column, range.line, range.end_column, content
        );
    } else {
        eprintln!("out of source : line {}", range.line);
    }
}
