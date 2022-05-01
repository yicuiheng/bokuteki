use crate::source::{Cursor, Source};
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InlineRange {
    pub cursor: Cursor,
    pub length: usize,
}

pub type BlockRange = VecDeque<InlineRange>;

impl InlineRange {
    pub fn next(&mut self, src: &Source) -> Option<char> {
        if self.is_empty() {
            None
        } else if let Some(c) = self.cursor.next(src) {
            self.length -= 1;
            Some(c)
        } else {
            None
        }
    }

    pub fn consume(mut self, src: &Source, n: usize) -> Self {
        for _ in 0..n {
            self.next(src).expect("unreachable");
        }
        self
    }

    pub fn starts_with(&self, src: &Source, text: &str) -> bool {
        self.length >= text.len() && self.cursor.starts_with(src, text)
    }

    pub fn match_(&self, src: &Source, text: &str) -> bool {
        let mut range = self.clone();
        for expected_char in text.chars() {
            if let Some(actual_char) = range.next(src) {
                if expected_char != actual_char {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn to_string(&self, src: &Source) -> String {
        let mut range = self.clone();
        let mut result = String::new();
        while let Some(c) = range.next(src) {
            result.push(c);
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn calc_range_on_source(&self, src: &Source) -> ((u32, u32), (u32, u32)) {
        let start = self.cursor.calc_point_on_source(src);
        let mut end = start.clone();
        end.1 += self.length as u32;
        (start, end)
    }
}
