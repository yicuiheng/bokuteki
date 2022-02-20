mod document;
mod parse;
mod print;
mod util;

use std::fs;

fn main() {
    let filepath = std::env::args().nth(1).expect("Usage: bokuteki <filename>");

    let src = fs::read_to_string(filepath.clone()).expect("failed to read file..");
    let src: Vec<Vec<char>> = src.lines().map(|line| line.chars().collect()).collect();

    for i in 0..src.len() {
        util::debug_at(
            &src,
            &document::Location {
                line: i,
                column: 0,
                filepath: filepath.clone(),
            },
        );
    }
    let src_full_range = document::Range::full_range(&src, filepath);
    let doc = parse::parse(&src, src_full_range);
    eprintln!("doc {:?}", doc);
    print::print(&src, doc);
}
