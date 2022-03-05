mod document;
mod parse;
mod print;
mod util;

use std::fs;

fn main() {
    let filepath = std::env::args().nth(1).expect("Usage: bokuteki <filename>");

    let src = fs::read_to_string(filepath.clone()).expect("failed to read file..");
    let src: Vec<Vec<char>> = src.lines().map(|line| line.chars().collect()).collect();
    let src_block_range = document::src_block_range(&src);

    let result = parse::parse_block_elements(&src, src_block_range);
    print::print(&src, result.value);
}
