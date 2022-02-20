use crate::document::*;

pub fn debug_at(src: &Vec<Vec<char>>, loc: &Location) {
    if let Some(line) = src.iter().nth(loc.line) {
        let str: String = line[loc.column..].iter().collect();
        eprintln!("at ({}:{}) | {}", loc.line, loc.column, str);
    } else {
        eprintln!("out of source..");
    }
}
