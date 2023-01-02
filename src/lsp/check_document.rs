use crate::{document, parse};

pub fn check_bok_content(content: String) -> (Vec<parse::Error>, Vec<parse::Warning>) {
    let src: Vec<Vec<char>> = content.lines().map(|line| line.chars().collect()).collect();
    let src_block_range = document::src_block_range(&src);

    let result = parse::parse_document(&src, src_block_range);
    (result.errors, result.warnings)
}
