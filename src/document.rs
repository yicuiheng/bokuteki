#[derive(Debug)]
pub struct Document {
    pub range: Range,
    pub block_elements: Vec<BlockElement>,
}

#[derive(Debug)]
pub enum BlockElement {
    Heading {
        level: usize,
        content: Vec<SpanElement>,
    },
    Paragraph {
        content: Vec<SpanElement>,
    },
}

#[derive(Debug)]
pub enum SpanElement {
    Text { range: Range },
    InlineCode { range: Range },
    InlineMath { range: Range },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Range {
    pub start: Location,
    pub end: Location,
}

impl Range {
    pub fn full_range(src: &Vec<Vec<char>>, filepath: String) -> Self {
        Range {
            start: Location {
                filepath: filepath.clone(),
                line: 0,
                column: 0,
            },
            end: Location {
                filepath,
                line: src.len(),
                column: 0,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub filepath: String,
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn move_to_next_char(&mut self) {
        self.column += 1;
    }

    pub fn move_to_next_line(&mut self) {
        self.column = 0;
        self.line += 1;
    }

    pub fn is_head_of_line(&self, _src: &Vec<Vec<char>>) -> bool {
        self.column == 0
    }
    pub fn is_tail_of_line(&self, src: &Vec<Vec<char>>) -> bool {
        assert!(self.line <= src.len());
        if self.line == src.len() {
            true
        } else {
            src[self.line].len() == self.column
        }
    }
}
