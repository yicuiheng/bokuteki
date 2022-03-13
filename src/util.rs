use crate::document::*;
use log::debug;

#[allow(dead_code)]
pub fn debug_at(src: &Vec<Vec<char>>, range: &InlineRange, msg: &str) {
    if let Some(line) = src.iter().nth(range.line) {
        let content: String = line[range.start_column..range.end_column].iter().collect();
        debug!(
            "{} at ({}:{}-{}:{}) | {}",
            msg, range.line, range.start_column, range.line, range.end_column, content
        );
    } else {
        debug!("out of source : line {}", range.line);
    }
}
