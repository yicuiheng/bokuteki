use crate::{document, parse, print};

use std::fs;
use std::path::PathBuf;

pub fn compile(input_file: PathBuf) {
    let src = fs::read_to_string(input_file.as_path()).expect("failed to read file..");
    compile_src(src);
}

pub fn compile_src(src: String) -> (Vec<parse::Error>, Vec<parse::Warning>) {
    let src: Vec<Vec<char>> = src.lines().map(|line| line.chars().collect()).collect();
    let src_block_range = document::src_block_range(&src);

    let result = parse::parse_block_elements(&src, src_block_range);
    if !result.errors.is_empty() || !result.warnings.is_empty() {
        (result.errors, result.warnings)
    } else {
        print::print(&src, result.value);
        (vec![], vec![])
    }
}
