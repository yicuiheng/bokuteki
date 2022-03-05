use std::collections::VecDeque;

// NOTE: 新しく BlockElement の種類を追加する場合は `parse::parse_paragraph` 関数内の `is_paragraph_end` 関数を修正すること
#[derive(Debug, PartialEq, Eq)]
pub enum BlockElement {
    Heading {
        level: usize,
        content: Vec<InlineElement>,
    },
    Paragraph {
        content: Vec<InlineElement>,
    },
    Code {
        lines: BlockRange,
    },
    Math {
        lines: BlockRange,
    },
    Theorem {
        kind: TheoremKind,
        title: Vec<InlineElement>,
        content: Vec<BlockElement>,
    },
    Proof {
        content: Vec<BlockElement>,
    },
    List {
        mark_kind: ListMarkKind,
        items: Vec<ListItem>,
    },
    Blockquote {
        inner: Vec<BlockElement>,
    },
    ParseError,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TheoremKind {
    Theorem,
    Proposition,
    Lemma,
    Corollary,
    Definition,
    Axiom,
    ParseError,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListMarkKind {
    Bullet,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ListItem {
    pub top_line: Vec<InlineElement>,
    pub blocks: Vec<BlockElement>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InlineElement {
    Text { range: InlineRange },
    Code { range: InlineRange },
    Math { range: InlineRange },
    SmallCaps { range: InlineRange },
    ParseError,
}

impl InlineElement {
    pub fn is_parse_error(&self) -> bool {
        self == &InlineElement::ParseError
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InlineRange {
    pub line: usize,
    pub start_column: usize,
    pub end_column: usize,
}

impl InlineRange {
    pub fn consume(mut self, n: usize) -> Self {
        self.start_column += n;
        assert!(self.start_column <= self.end_column);
        self
    }

    pub fn move_to_next_char(&mut self) {
        self.start_column += 1;
        assert!(self.start_column <= self.end_column);
    }

    pub fn is_empty(&self) -> bool {
        self.start_column == self.end_column
    }
}

pub type BlockRange = VecDeque<InlineRange>;

pub fn src_block_range(src: &Vec<Vec<char>>) -> BlockRange {
    src.iter()
        .enumerate()
        .map(|(line_idx, line)| InlineRange {
            line: line_idx,
            start_column: 0,
            end_column: line.len(),
        })
        .collect()
}
