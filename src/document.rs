use std::collections::HashMap;

use crate::parse;
use crate::source::{
    range::{BlockRange, InlineRange},
    Source,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Document(pub Vec<BlockElement>);

impl Document {
    pub fn update(
        &mut self,
        src: &Source,
        cache: &mut HashMap<InlineRange, String>,
    ) -> (Vec<String>, Vec<String>) {
        let result = parse::parse_document(src, cache);
        *self = result.value;
        (result.errors, result.warnings)
    }
}

// NOTE: 新しく BlockElement の種類を追加する場合は `parse::parse_paragraph` 関数内の `is_paragraph_end` 関数を修正すること
#[derive(Debug, PartialEq, Eq)]
pub enum BlockElement {
    Heading {
        level: usize,
        content: Vec<InlineElement>,
    },
    Paragraph {
        content: Vec<InlineElement>,
        height_on_source: usize,
    },
    Code {
        lines: BlockRange,
        height_on_source: usize,
    },
    Math {
        lines: BlockRange,
        height_on_source: usize,
    },
    Theorem {
        kind: TheoremKind,
        title: Vec<InlineElement>,
        content: Vec<BlockElement>,
        height_on_source: usize,
    },
    Proof {
        content: Vec<BlockElement>,
        height_on_source: usize,
    },
    Derivation(Derivation),
    List {
        mark_kind: ListMarkKind,
        items: Vec<ListItem>,
        height_on_source: usize,
    },
    Blockquote {
        inner: Vec<BlockElement>,
        height_on_source: usize,
    },
    EmptyLines {
        height_on_source: usize,
    },
    ParseError,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Derivation {
    InferenceRule {
        premises: Vec<Derivation>,
        conclusion: Vec<InlineElement>,
        rule_name: Vec<InlineElement>,
        height_on_source: usize,
    },
    Leaf {
        inline_elements: Vec<InlineElement>,
        height_on_source: usize,
    },
}

impl BlockElement {
    #[allow(dead_code)]
    pub fn kind(&self) -> &'static str {
        use BlockElement::{
            Blockquote, Code, EmptyLines, Heading, List, Math, Paragraph, Proof, Theorem,
        };
        match self {
            Heading { .. } => "heading",
            Paragraph { .. } => "paragraph",
            Code { .. } => "code",
            Math { .. } => "math",
            Theorem { .. } => "theorem",
            Proof { .. } => "proof",
            BlockElement::Derivation(_) => "derivation",
            List { .. } => "list",
            Blockquote { .. } => "blockquote",
            EmptyLines { .. } => "empty lines",
            _ => unreachable!(),
        }
    }

    pub fn height_on_source(&self) -> usize {
        use BlockElement::{
            Blockquote, Code, EmptyLines, Heading, List, Math, Paragraph, Proof, Theorem,
        };
        match self {
            Heading { .. } => 1,
            Paragraph {
                height_on_source, ..
            }
            | Code {
                height_on_source, ..
            }
            | Math {
                height_on_source, ..
            }
            | Theorem {
                height_on_source, ..
            }
            | Proof {
                height_on_source, ..
            }
            | BlockElement::Derivation(Derivation::Leaf {
                height_on_source, ..
            })
            | BlockElement::Derivation(Derivation::InferenceRule {
                height_on_source, ..
            })
            | List {
                height_on_source, ..
            }
            | Blockquote {
                height_on_source, ..
            }
            | EmptyLines {
                height_on_source, ..
            } => *height_on_source,
            _ => unreachable!(),
        }
    }
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
    Text {
        range: InlineRange,
        width_on_source: usize,
    },
    Code {
        range: InlineRange,
        width_on_source: usize,
    },
    Math {
        range: InlineRange,
        width_on_source: usize,
    },
    SmallCaps {
        range: InlineRange,
        width_on_source: usize,
    },
    ParseError,
}

impl InlineElement {
    pub fn is_parse_error(&self) -> bool {
        self == &InlineElement::ParseError
    }

    pub fn width_on_source(&self) -> usize {
        use InlineElement::*;
        match self {
            Text {
                width_on_source, ..
            }
            | Code {
                width_on_source, ..
            }
            | Math {
                width_on_source, ..
            }
            | SmallCaps {
                width_on_source, ..
            } => *width_on_source,
            _ => unreachable!(),
        }
    }
}
