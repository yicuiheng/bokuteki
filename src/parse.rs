use crate::{
    document::*,
    lsp::state::Cache,
    source::{
        range::{BlockRange, InlineRange},
        Source,
    },
};
use std::collections::VecDeque;

pub type Error = String; // TODO: マシなエラー型をつける
pub type Warning = String; // TODO: マシな警告型をつける

pub struct ParseResult<V, R> {
    pub value: V,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
    pub rest_range: R,
}

type ParseBlockElementResult = ParseResult<BlockElement, BlockRange>;
type ParseInlineElementResult = ParseResult<InlineElement, InlineRange>;

pub fn parse_document(src: &Source, cache: &mut Cache) -> ParseResult<Document, BlockRange> {
    let range = src.whole_range();
    let block_elements_result = parse_block_elements(src, range);
    ParseResult {
        value: Document(block_elements_result.value),
        errors: block_elements_result.errors,
        warnings: block_elements_result.warnings,
        rest_range: block_elements_result.rest_range,
    }
}

fn parse_block_elements(
    src: &Source,
    src_range: BlockRange,
) -> ParseResult<Vec<BlockElement>, BlockRange> {
    let mut rest_range = src_range;
    let mut block_elements = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];
    loop {
        // 空行ブロック
        let mut height_on_source = 0;
        loop {
            if let Some(top_line_range) = rest_range.front() {
                if top_line_range.is_empty() {
                    rest_range.pop_front();
                    height_on_source += 1;
                } else {
                    break;
                }
            } else {
                if height_on_source != 0 {
                    block_elements.push(BlockElement::EmptyLines { height_on_source });
                }
                return ParseResult {
                    value: block_elements,
                    errors,
                    warnings,
                    rest_range: rest_range,
                };
            }
        }
        if height_on_source != 0 {
            block_elements.push(BlockElement::EmptyLines { height_on_source });
        }

        // ここでは `rest_range` はブロック要素から始まっているはず
        let mut result = parse_block_element(src, rest_range);
        block_elements.push(result.value);
        errors.append(&mut result.errors);
        warnings.append(&mut result.warnings);
        rest_range = result.rest_range;
    }
}

fn parse_block_element(src: &Source, rest_range: BlockRange) -> ParseBlockElementResult {
    assert!(!rest_range.is_empty());
    let top_line_range = rest_range.front().expect("`rest_range` can not be empty");
    assert!(!top_line_range.is_empty()); // 空行始まりではない

    let result = parse_heading_block_element(src, &rest_range);
    if result.value != BlockElement::ParseError {
        return result;
    }

    let result = parse_code_block_element(src, &rest_range);
    if result.value != BlockElement::ParseError {
        return result;
    }

    let result = parse_math_block_element(src, &rest_range);
    if result.value != BlockElement::ParseError {
        return result;
    }

    let result = parse_theorem_block_element(src, rest_range.clone());
    if result.value != BlockElement::ParseError {
        return result;
    }

    let result = parse_proof_block_element(src, rest_range.clone());
    if result.value != BlockElement::ParseError {
        return result;
    }

    let result = parse_derivation_block_element(src, rest_range.clone());
    if let Some(derivation) = result.value {
        return ParseBlockElementResult {
            value: BlockElement::Derivation(derivation),
            errors: result.errors,
            warnings: result.warnings,
            rest_range: result.rest_range,
        };
    }

    let result = parse_list_block_element(src, rest_range.clone());
    if result.value != BlockElement::ParseError {
        return result;
    }

    let result = parse_blockquote_element(src, rest_range.clone());
    if result.value != BlockElement::ParseError {
        return result;
    }

    parse_paragraph(src, rest_range)
}

fn parse_heading_block_element(src: &Source, rest_range: &BlockRange) -> ParseBlockElementResult {
    let parse_error = ParseBlockElementResult {
        value: BlockElement::ParseError,
        errors: vec![],
        warnings: vec![],
        rest_range: rest_range.clone(),
    };

    let mut rest_range = rest_range.clone();
    if let Some(mut line_rest_range) = rest_range.pop_front() {
        if line_rest_range.starts_with(src, "#") {
            let mut level = 0;
            while line_rest_range.starts_with(src, "#") {
                assert_eq!(line_rest_range.next(src), Some('#'));
                level += 1;
            }

            let inline_elements_result = parse_inline_elements(src, line_rest_range);
            assert!(inline_elements_result.rest_range.is_empty());
            let errors = inline_elements_result.errors;
            let warnings = inline_elements_result.warnings;

            ParseBlockElementResult {
                value: BlockElement::Heading {
                    level,
                    content: inline_elements_result.value,
                },
                errors,
                warnings,
                rest_range,
            }
        } else {
            parse_error
        }
    } else {
        parse_error
    }
}

fn parse_code_block_element(src: &Source, rest_range: &BlockRange) -> ParseBlockElementResult {
    fn check_start_line(src: &Source, line: &InlineRange) -> Option<()> {
        if line.starts_with(src, "```") {
            Some(())
        } else {
            None
        }
    }
    fn check_end_line(src: &Source, line: &InlineRange) -> Option<()> {
        if line.match_(src, "```") {
            Some(())
        } else {
            None
        }
    }
    fn make_code_block(range: BlockRange, height_on_source: usize, _: (), _: ()) -> BlockElement {
        BlockElement::Code {
            lines: range,
            height_on_source,
        }
    }

    parse_surrounded_block_element(
        src,
        rest_range,
        check_start_line,
        check_end_line,
        make_code_block,
    )
}

fn parse_math_block_element(src: &Source, rest_range: &BlockRange) -> ParseBlockElementResult {
    fn check_start_line(src: &Source, line: &InlineRange) -> Option<()> {
        if line.match_(src, "$$") {
            Some(())
        } else {
            None
        }
    }
    fn check_end_line(src: &Source, line: &InlineRange) -> Option<()> {
        if line.match_(src, "$$") {
            Some(())
        } else {
            None
        }
    }
    fn make_math_block(range: BlockRange, height_on_source: usize, _: (), _: ()) -> BlockElement {
        BlockElement::Math {
            lines: range,
            height_on_source,
        }
    }

    parse_surrounded_block_element(
        src,
        rest_range,
        check_start_line,
        check_end_line,
        make_math_block,
    )
}

fn parse_surrounded_block_element<T, U>(
    src: &Source,
    rest_range: &BlockRange,
    check_start_line: fn(src: &Source, &InlineRange) -> Option<T>,
    check_end_line: fn(src: &Source, &InlineRange) -> Option<U>,
    make_func: fn(BlockRange, usize, T, U) -> BlockElement,
) -> ParseBlockElementResult {
    let mut rest_range = rest_range.clone();
    let parse_error = ParseBlockElementResult {
        value: BlockElement::ParseError,
        errors: vec![],
        warnings: vec![],
        rest_range: rest_range.clone(),
    };
    let errors = vec![];
    let warnings = vec![];
    let mut height_on_source = 0;

    if let Some(line) = rest_range.pop_front() {
        height_on_source += 1;
        if let Some(t) = check_start_line(src, &line) {
            let mut content_lines = VecDeque::new();
            loop {
                if let Some(line) = rest_range.pop_front() {
                    height_on_source += 1;
                    if let Some(u) = check_end_line(src, &line) {
                        return ParseBlockElementResult {
                            value: make_func(content_lines, height_on_source, t, u),
                            errors,
                            warnings,
                            rest_range,
                        };
                    } else {
                        content_lines.push_back(line);
                    }
                } else {
                    return parse_error;
                }
            }
        } else {
            parse_error
        }
    } else {
        parse_error
    }
}

fn parse_theorem_block_element(
    src: &Source,
    mut rest_range: BlockRange,
) -> ParseBlockElementResult {
    let parse_error = ParseBlockElementResult {
        value: BlockElement::ParseError,
        errors: vec![],
        warnings: vec![],
        rest_range: rest_range.clone(),
    };
    let mut errors = vec![];
    let mut warnings = vec![];

    let mut height_on_source = 1;
    let (kind, title) = if let Some(line) = rest_range.pop_front() {
        let mut kind_result = parse_theorem_kind(src, &line);
        errors.append(&mut kind_result.errors);
        warnings.append(&mut kind_result.warnings);
        if kind_result.value == TheoremKind::ParseError {
            return parse_error;
        }
        let mut inline_elements_result = parse_inline_elements(src, kind_result.rest_range);
        assert!(inline_elements_result.rest_range.is_empty());
        errors.append(&mut inline_elements_result.errors);
        warnings.append(&mut inline_elements_result.warnings);
        (kind_result.value, inline_elements_result.value)
    } else {
        return parse_error;
    };

    let inner_range = lift_block_range(src, "  ", rest_range);
    height_on_source += inner_range.value.len();
    rest_range = inner_range.rest_range;

    let mut inner_result = parse_block_elements(src, inner_range.value);
    assert!(inner_result.rest_range.is_empty());
    errors.append(&mut inner_result.errors);
    warnings.append(&mut inner_result.warnings);

    ParseBlockElementResult {
        value: BlockElement::Theorem {
            kind,
            title,
            content: inner_result.value,
            height_on_source,
        },
        errors,
        warnings,
        rest_range,
    }
}
const MARK_TO_THEOREM_KIND: [(&'static str, TheoremKind); 26] = [
    // 定理
    ("Theorem. ", TheoremKind::Theorem),
    ("theorem. ", TheoremKind::Theorem),
    ("Thm. ", TheoremKind::Theorem),
    ("thm. ", TheoremKind::Theorem),
    ("Th. ", TheoremKind::Theorem),
    ("th. ", TheoremKind::Theorem),
    // 命題
    ("Proposition. ", TheoremKind::Proposition),
    ("proposition. ", TheoremKind::Proposition),
    ("Prop. ", TheoremKind::Proposition),
    ("prop. ", TheoremKind::Proposition),
    // 補題
    ("Lemma. ", TheoremKind::Lemma),
    ("lemma. ", TheoremKind::Lemma),
    ("Lem. ", TheoremKind::Lemma),
    ("lem. ", TheoremKind::Lemma),
    // 系
    ("Corollary. ", TheoremKind::Corollary),
    ("corollary. ", TheoremKind::Corollary),
    ("Cor. ", TheoremKind::Corollary),
    ("cor. ", TheoremKind::Corollary),
    // 定義
    ("Definition. ", TheoremKind::Definition),
    ("definition. ", TheoremKind::Definition),
    ("Def. ", TheoremKind::Definition),
    ("def. ", TheoremKind::Definition),
    // 公理
    ("Axiom. ", TheoremKind::Axiom),
    ("axiom. ", TheoremKind::Axiom),
    ("Axm. ", TheoremKind::Axiom),
    ("axm. ", TheoremKind::Axiom),
];

fn parse_theorem_kind(
    src: &Source,
    rest_range: &InlineRange,
) -> ParseResult<TheoremKind, InlineRange> {
    let rest_range = rest_range.clone();
    for (mark, kind) in &MARK_TO_THEOREM_KIND {
        if rest_range.starts_with(src, mark) {
            return ParseResult {
                value: *kind,
                errors: vec![],
                warnings: vec![],
                rest_range: rest_range.consume(src, mark.len()),
            };
        }
    }

    ParseResult {
        value: TheoremKind::ParseError,
        errors: vec![],
        warnings: vec![],
        rest_range: rest_range.clone(),
    }
}

fn parse_proof_block_element(src: &Source, mut rest_range: BlockRange) -> ParseBlockElementResult {
    let parse_error = ParseBlockElementResult {
        value: BlockElement::ParseError,
        errors: vec![],
        warnings: vec![],
        rest_range: rest_range.clone(),
    };
    let mut errors = vec![];
    let mut warnings = vec![];

    let mut height_on_source = 1;
    if let Some(line) = rest_range.pop_front() {
        if !line.match_(src, "Proof.") && !line.match_(src, "proof.") {
            return parse_error;
        }
    } else {
        return parse_error;
    };

    let mut inner_range = lift_block_range(src, "  ", rest_range);
    errors.append(&mut inner_range.errors);
    warnings.append(&mut inner_range.warnings);
    rest_range = inner_range.rest_range;
    height_on_source += inner_range.value.len();

    let mut inner_result = parse_block_elements(src, inner_range.value);
    assert!(inner_result.rest_range.is_empty());
    errors.append(&mut inner_result.errors);
    warnings.append(&mut inner_result.warnings);
    height_on_source = inner_result
        .value
        .iter()
        .fold(height_on_source, |acc, inner_block_element| {
            acc + inner_block_element.height_on_source()
        });

    ParseBlockElementResult {
        value: BlockElement::Proof {
            content: inner_result.value,
            height_on_source,
        },
        errors,
        warnings,
        rest_range,
    }
}

fn parse_derivation_block_element(
    src: &Source,
    mut rest_range: BlockRange,
) -> ParseResult<Option<Derivation>, BlockRange> {
    let parse_error = ParseResult {
        value: None,
        errors: vec![],
        warnings: vec![],
        rest_range: rest_range.clone(),
    };
    let mut errors = vec![];
    let mut warnings = vec![];

    let mut premises_range_result = lift_block_range(src, "  ", rest_range);
    let mut height_on_source = premises_range_result.value.len();
    errors.append(&mut premises_range_result.errors);
    warnings.append(&mut premises_range_result.warnings);
    let mut premises: Vec<Derivation> = vec![];
    let mut premises_rest_range = premises_range_result.value;
    while !premises_rest_range.is_empty() {
        let mut premise_result = parse_derivation_block_element(src, premises_rest_range.clone());
        if let Some(premise) = premise_result.value {
            errors.append(&mut premise_result.errors);
            warnings.append(&mut premise_result.warnings);
            premises.push(premise);
            premises_rest_range = premise_result.rest_range;
        } else if let Some(line_range) = premises_rest_range.pop_front() {
            let height_on_source = 1;
            let mut inline_elements_result = parse_inline_elements(src, line_range);
            errors.append(&mut inline_elements_result.errors);
            warnings.append(&mut inline_elements_result.warnings);
            premises.push(Derivation::Leaf {
                inline_elements: inline_elements_result.value,
                height_on_source,
            });
        }
    }
    rest_range = premises_range_result.rest_range;

    let mut rule_name_result = if let Some(mut line_range) = rest_range.pop_front() {
        let mut count = 0;
        while line_range.starts_with(src, "-") {
            count += 1;
            line_range.next(src);
        }
        if count >= 3 {
            parse_inline_elements(src, line_range)
        } else {
            return parse_error;
        }
    } else {
        return parse_error;
    };
    height_on_source += 1;
    errors.append(&mut rule_name_result.errors);
    warnings.append(&mut rule_name_result.warnings);
    let rule_name = rule_name_result.value;

    let mut conclusion_range_result = lift_block_range(src, "  ", rest_range);
    height_on_source += conclusion_range_result.value.len();
    errors.append(&mut conclusion_range_result.errors);
    warnings.append(&mut conclusion_range_result.warnings);
    rest_range = conclusion_range_result.rest_range;
    let mut conclusion: Vec<InlineElement> = vec![];
    for conclusion_line_range in conclusion_range_result.value {
        let mut inline_elements_result = parse_inline_elements(src, conclusion_line_range);
        errors.append(&mut inline_elements_result.errors);
        warnings.append(&mut inline_elements_result.warnings);
        conclusion.append(&mut inline_elements_result.value);
    }

    ParseResult {
        value: Some(Derivation::InferenceRule {
            premises,
            conclusion,
            rule_name,
            height_on_source,
        }),
        errors,
        warnings,
        rest_range,
    }
}

fn parse_list_block_element(src: &Source, mut rest_range: BlockRange) -> ParseBlockElementResult {
    let mut errors = vec![];
    let mut warnings = vec![];

    let mut items = vec![];
    let mut height_on_source = 0;
    while let Some(line) = rest_range.pop_front() {
        height_on_source += 1;
        if !line.starts_with(src, "- ") {
            rest_range.push_front(line);
            break;
        }

        let mut top_line_result = parse_inline_elements(src, line.consume(src, 2));
        errors.append(&mut top_line_result.errors);
        warnings.append(&mut top_line_result.warnings);
        let top_line = top_line_result.value;

        let mut inner_range_result = lift_block_range(src, "  ", rest_range);
        height_on_source += inner_range_result.value.len();
        let inner_range = inner_range_result.value;
        errors.append(&mut inner_range_result.errors);
        warnings.append(&mut inner_range_result.warnings);
        rest_range = inner_range_result.rest_range;

        let mut block_elements_result = parse_block_elements(src, inner_range);
        errors.append(&mut block_elements_result.errors);
        warnings.append(&mut block_elements_result.warnings);
        let blocks = block_elements_result.value;

        items.push(ListItem { top_line, blocks });
    }

    ParseBlockElementResult {
        value: if items.is_empty() {
            BlockElement::ParseError
        } else {
            BlockElement::List {
                mark_kind: ListMarkKind::Bullet,
                items,
                height_on_source,
            }
        },
        errors,
        warnings,
        rest_range,
    }
}

fn parse_blockquote_element(src: &Source, mut rest_range: BlockRange) -> ParseBlockElementResult {
    let parse_error = ParseBlockElementResult {
        value: BlockElement::ParseError,
        errors: vec![],
        warnings: vec![],
        rest_range: rest_range.clone(),
    };
    let mut errors = vec![];
    let mut warnings = vec![];

    if let Some(line) = rest_range.pop_front() {
        if line.starts_with(src, "> ") {
            let mut inner_range = BlockRange::new();
            inner_range.push_back(line.consume(src, 2));
            loop {
                if let Some(line) = rest_range.pop_front() {
                    if line.starts_with(src, "> ") {
                        inner_range.push_back(line.consume(src, 2));
                    } else {
                        rest_range.push_front(line);
                        break;
                    }
                } else {
                    break;
                }
            }
            let height_on_source = inner_range.len();

            let mut inner_result = parse_block_elements(src, inner_range);
            let inner = inner_result.value;
            assert!(inner_result.rest_range.is_empty());
            errors.append(&mut inner_result.errors);
            warnings.append(&mut inner_result.warnings);

            ParseBlockElementResult {
                value: BlockElement::Blockquote {
                    inner,
                    height_on_source,
                },
                errors,
                warnings,
                rest_range,
            }
        } else {
            parse_error
        }
    } else {
        parse_error
    }
}

fn parse_paragraph(src: &Source, mut rest_range: BlockRange) -> ParseBlockElementResult {
    let mut inline_elements = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];

    let mut height_on_source = 0;
    let head_line = rest_range.front().cloned();
    height_on_source += 1;

    // ソース終端もしくは別のブロック要素の始まりまで Inline 要素をパースする
    while !is_paragraph_end(src, &rest_range, &head_line) {
        let rest_line_range = rest_range.pop_front().expect("can not be empty");
        height_on_source += 1;
        let mut inline_elements_result = parse_inline_elements(src, rest_line_range);
        assert!(inline_elements_result.rest_range.is_empty());
        errors.append(&mut inline_elements_result.errors);
        warnings.append(&mut inline_elements_result.warnings);
        inline_elements.append(&mut &mut inline_elements_result.value);
    }

    // 段落の終わりとは以下のどれか
    // - ソース終端
    // - 別のブロック要素の始まり
    //   - 見出しブロックの始まり
    //   - 段落の区切り行（空行）
    //   - コードブロックの始まり
    //   - 数式ブロックの始まり
    //   - 定理ブロックの始まり
    //   - 証明ブロックの始まり
    //   - リストブロックの始まり
    //   - 引用ブロックの始まり
    // ただしコードブロックの終端マーク ("```") もしくは 数式ブロックの終端マーク ("$$") が1行目に出現した場合は、それは段落の終わりではない。
    fn is_paragraph_end(
        src: &Source,
        rest_range: &BlockRange,
        head_line: &Option<InlineRange>,
    ) -> bool {
        if parse_derivation_block_element(src, rest_range.clone())
            .value
            .is_some()
        {
            return true;
        }
        if let Some(line_range) = rest_range.front() {
            line_range.starts_with(src, "#")
                || line_range.is_empty()
                || if let Some(head_line) = head_line {
                    if head_line != line_range {
                        line_range.starts_with(src, "```") || line_range.starts_with(src, "$$")
                    } else {
                        false
                    }
                } else {
                    false
                }
                || MARK_TO_THEOREM_KIND
                    .iter()
                    .any(|(mark, _)| line_range.starts_with(src, mark))
                || line_range.starts_with(src, "Proof.")
                || line_range.starts_with(src, "proof.")
                || line_range.starts_with(src, "- ")
                || line_range.starts_with(src, "> ")
        } else {
            true
        }
    }

    ParseBlockElementResult {
        value: BlockElement::Paragraph {
            content: inline_elements,
            height_on_source,
        },
        errors,
        warnings,
        rest_range,
    }
}

fn lift_block_range(
    src: &Source,
    prefix: &str,
    range: BlockRange,
) -> ParseResult<BlockRange, BlockRange> {
    let mut rest_range = range;
    let mut lifted_range = BlockRange::new();
    loop {
        if let Some(line) = rest_range.pop_front() {
            if line.starts_with(src, prefix) {
                lifted_range.push_back(line.consume(src, prefix.len()));
            } else {
                rest_range.push_front(line);
                break;
            }
        } else {
            break;
        }
    }
    ParseResult {
        value: lifted_range,
        errors: vec![],
        warnings: vec![],
        rest_range,
    }
}

fn parse_inline_elements(
    src: &Source,
    mut rest_range: InlineRange,
) -> ParseResult<Vec<InlineElement>, InlineRange> {
    let mut inline_elements = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];
    while !rest_range.is_empty() {
        let mut result = parse_inline_element(src, rest_range);
        inline_elements.push(result.value);
        errors.append(&mut result.errors);
        warnings.append(&mut result.warnings);
        rest_range = result.rest_range;
    }
    ParseResult {
        value: inline_elements,
        errors,
        warnings,
        rest_range,
    }
}

fn parse_inline_element(src: &Source, mut rest_range: InlineRange) -> ParseInlineElementResult {
    let result = parse_inline_math_element(src, &rest_range);
    if !result.value.is_parse_error() {
        return result;
    }

    let result = parse_inline_code_element(src, &rest_range);
    if !result.value.is_parse_error() {
        return result;
    }

    let result = parse_inline_small_caps_element(src, &rest_range);
    if !result.value.is_parse_error() {
        return result;
    }

    let mut length = 0;
    let start_cursor = rest_range.cursor.clone();
    let errors = vec![];
    let warnings = vec![];
    while !rest_range.is_empty() {
        if !parse_inline_math_element(src, &rest_range)
            .value
            .is_parse_error()
            || !parse_inline_code_element(src, &rest_range)
                .value
                .is_parse_error()
            || !parse_inline_small_caps_element(src, &rest_range)
                .value
                .is_parse_error()
        {
            let inline_range = InlineRange {
                cursor: start_cursor,
                length,
            };
            return ParseInlineElementResult {
                value: InlineElement::Text {
                    range: inline_range,
                    width_on_source: length,
                },
                errors,
                warnings,
                rest_range,
            };
        }
        rest_range.next(src);
        length += 1;
    }

    ParseInlineElementResult {
        value: InlineElement::Text {
            range: InlineRange {
                cursor: start_cursor,
                length,
            },
            width_on_source: length,
        },
        errors,
        warnings,
        rest_range,
    }
}

fn parse_inline_math_element(src: &Source, rest_range: &InlineRange) -> ParseInlineElementResult {
    parse_surrounded_inline_element(
        src,
        "$",
        |content_range, width_on_source| InlineElement::Math {
            range: content_range,
            width_on_source,
        },
        rest_range,
    )
}

fn parse_inline_code_element(src: &Source, rest_range: &InlineRange) -> ParseInlineElementResult {
    parse_surrounded_inline_element(
        src,
        "`",
        |content_range, width_on_source| InlineElement::Code {
            range: content_range,
            width_on_source,
        },
        rest_range,
    )
}

fn parse_inline_small_caps_element(
    src: &Source,
    rest_range: &InlineRange,
) -> ParseInlineElementResult {
    parse_surrounded_inline_element(
        src,
        "%",
        |content_range, width_on_source| InlineElement::SmallCaps {
            range: content_range,
            width_on_source,
        },
        rest_range,
    )
}

fn parse_surrounded_inline_element(
    src: &Source,
    mark: &str,
    make_inline_element: fn(InlineRange, usize) -> InlineElement,
    rest_range: &InlineRange,
) -> ParseInlineElementResult {
    let mut rest_range = rest_range.clone();
    let errors = vec![];
    let warnings = vec![];
    let mut width_on_source = 0;
    if rest_range.starts_with(src, mark) {
        rest_range = rest_range.consume(src, mark.len());
        let content_start_cursor = rest_range.cursor.clone();
        width_on_source += mark.len();
        loop {
            if rest_range.is_empty() {
                return ParseInlineElementResult {
                    value: InlineElement::ParseError,
                    errors,
                    warnings,
                    rest_range,
                };
            }
            if rest_range.starts_with(src, mark) {
                rest_range = rest_range.consume(src, mark.len());
                width_on_source += mark.len();
                let inline_content_range = InlineRange {
                    cursor: content_start_cursor,
                    length: width_on_source - mark.len() * 2,
                };
                let inline_element = make_inline_element(inline_content_range, width_on_source);
                return ParseInlineElementResult {
                    value: inline_element,
                    errors,
                    warnings,
                    rest_range,
                };
            } else {
                rest_range.next(src);
                width_on_source += 1;
            }
        }
    } else {
        ParseInlineElementResult {
            value: InlineElement::ParseError,
            errors,
            warnings,
            rest_range,
        }
    }
}
