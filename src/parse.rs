use crate::document::*;
use std::collections::VecDeque;

type Error = String; // TODO: マシなエラー型をつける
type Warning = String; // TODO: マシな警告型をつける

pub struct ParseResult<V, R> {
    pub value: V,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
    pub rest_range: R,
}

type ParseBlockElementResult = ParseResult<BlockElement, BlockRange>;
type ParseInlineElementResult = ParseResult<InlineElement, InlineRange>;

pub fn parse_block_elements(
    src: &Vec<Vec<char>>,
    src_range: BlockRange,
) -> ParseResult<Vec<BlockElement>, BlockRange> {
    let mut rest_range = src_range;
    let mut block_elements = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];
    loop {
        // 空行は無視する
        loop {
            if let Some(top_line_range) = rest_range.front() {
                if top_line_range.is_empty() {
                    rest_range.pop_front();
                } else {
                    break;
                }
            } else {
                return ParseResult {
                    value: block_elements,
                    errors,
                    warnings,
                    rest_range: rest_range,
                };
            }
        }

        // ここでは `rest_range` はブロック要素から始まっているはず
        let mut result = parse_block_element(src, rest_range);
        block_elements.push(result.value);
        errors.append(&mut result.errors);
        warnings.append(&mut result.warnings);
        rest_range = result.rest_range;
    }
}

fn parse_block_element(src: &Vec<Vec<char>>, rest_range: BlockRange) -> ParseBlockElementResult {
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

fn parse_heading_block_element(
    src: &Vec<Vec<char>>,
    rest_range: &BlockRange,
) -> ParseBlockElementResult {
    let parse_error = ParseBlockElementResult {
        value: BlockElement::ParseError,
        errors: vec![],
        warnings: vec![],
        rest_range: rest_range.clone(),
    };

    let mut rest_range = rest_range.clone();
    if let Some(mut line_rest_range) = rest_range.pop_front() {
        if starts_with(src, "#", line_rest_range) {
            let mut level = 0;
            while starts_with(src, "#", line_rest_range) {
                line_rest_range.move_to_next_char();
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

fn parse_code_block_element(
    src: &Vec<Vec<char>>,
    rest_range: &BlockRange,
) -> ParseBlockElementResult {
    fn check_start_line(src: &Vec<Vec<char>>, line: InlineRange) -> Option<()> {
        if starts_with(src, "```", line.clone()) {
            Some(())
        } else {
            None
        }
    }
    fn check_end_line(src: &Vec<Vec<char>>, line: InlineRange) -> Option<()> {
        if match_(src, "```", line.clone()) {
            Some(())
        } else {
            None
        }
    }
    fn make_code_block(range: BlockRange, _: (), _: ()) -> BlockElement {
        BlockElement::Code { lines: range }
    }
    parse_surrounded_block_element(
        src,
        rest_range,
        check_start_line,
        check_end_line,
        make_code_block,
    )
}

fn parse_math_block_element(
    src: &Vec<Vec<char>>,
    rest_range: &BlockRange,
) -> ParseBlockElementResult {
    fn check_start_line(src: &Vec<Vec<char>>, line: InlineRange) -> Option<()> {
        if match_(src, "$$", line.clone()) {
            Some(())
        } else {
            None
        }
    }
    fn check_end_line(src: &Vec<Vec<char>>, line: InlineRange) -> Option<()> {
        if match_(src, "$$", line.clone()) {
            Some(())
        } else {
            None
        }
    }
    fn make_math_block(range: BlockRange, _: (), _: ()) -> BlockElement {
        BlockElement::Math { lines: range }
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
    src: &Vec<Vec<char>>,
    rest_range: &BlockRange,
    check_start_line: fn(src: &Vec<Vec<char>>, InlineRange) -> Option<T>,
    check_end_line: fn(src: &Vec<Vec<char>>, InlineRange) -> Option<U>,
    make_func: fn(BlockRange, T, U) -> BlockElement,
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

    if let Some(line) = rest_range.pop_front() {
        if let Some(t) = check_start_line(src, line) {
            let mut content_lines = VecDeque::new();
            loop {
                if let Some(line) = rest_range.pop_front() {
                    if let Some(u) = check_end_line(src, line) {
                        return ParseBlockElementResult {
                            value: make_func(content_lines, t, u),
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
    src: &Vec<Vec<char>>,
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
    if inner_range.value.is_empty() {
        return parse_error;
    }
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
    src: &Vec<Vec<char>>,
    rest_range: &InlineRange,
) -> ParseResult<TheoremKind, InlineRange> {
    for (mark, kind) in &MARK_TO_THEOREM_KIND {
        if starts_with(src, mark, rest_range.clone()) {
            return ParseResult {
                value: *kind,
                errors: vec![],
                warnings: vec![],
                rest_range: rest_range.consume(mark.len()),
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

fn parse_proof_block_element(
    src: &Vec<Vec<char>>,
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

    if let Some(line) = rest_range.pop_front() {
        if !match_(src, "Proof.", line) && !match_(src, "proof.", line) {
            return parse_error;
        }
    } else {
        return parse_error;
    };

    let mut inner_range = lift_block_range(src, "  ", rest_range);
    errors.append(&mut inner_range.errors);
    warnings.append(&mut inner_range.warnings);
    rest_range = inner_range.rest_range;

    let mut inner_result = parse_block_elements(src, inner_range.value);
    assert!(inner_result.rest_range.is_empty());
    errors.append(&mut inner_result.errors);
    warnings.append(&mut inner_result.warnings);

    ParseBlockElementResult {
        value: BlockElement::Proof {
            content: inner_result.value,
        },
        errors,
        warnings,
        rest_range,
    }
}

fn parse_list_block_element(
    src: &Vec<Vec<char>>,
    mut rest_range: BlockRange,
) -> ParseBlockElementResult {
    let mut errors = vec![];
    let mut warnings = vec![];

    let mut items = vec![];
    while let Some(line) = rest_range.pop_front() {
        if !starts_with(src, "- ", line) {
            rest_range.push_front(line);
            break;
        }

        let mut top_line_result = parse_inline_elements(src, line.consume(2));
        errors.append(&mut top_line_result.errors);
        warnings.append(&mut top_line_result.warnings);
        let top_line = top_line_result.value;

        let mut inner_range_result = lift_block_range(src, "  ", rest_range);
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
            }
        },
        errors,
        warnings,
        rest_range,
    }
}

fn parse_blockquote_element(
    src: &Vec<Vec<char>>,
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

    if let Some(line) = rest_range.pop_front() {
        if starts_with(src, "> ", line) {
            let mut inner_range = BlockRange::new();
            inner_range.push_back(line.consume(2));
            loop {
                if let Some(line) = rest_range.pop_front() {
                    if starts_with(src, "> ", line) {
                        inner_range.push_back(line.consume(2));
                    } else {
                        rest_range.push_front(line);
                        break;
                    }
                } else {
                    break;
                }
            }

            let mut inner_result = parse_block_elements(src, inner_range);
            let inner = inner_result.value;
            assert!(inner_result.rest_range.is_empty());
            errors.append(&mut inner_result.errors);
            warnings.append(&mut inner_result.warnings);

            ParseBlockElementResult {
                value: BlockElement::Blockquote { inner },
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

fn parse_paragraph(src: &Vec<Vec<char>>, mut rest_range: BlockRange) -> ParseBlockElementResult {
    let mut inline_elements = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];

    // ソース終端もしくは別のブロック要素の始まりまで Inline 要素をパースする
    while !is_paragraph_end(src, &rest_range) {
        let rest_line_range = rest_range.pop_front().expect("can not be empty");
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
    fn is_paragraph_end(src: &Vec<Vec<char>>, rest_range: &BlockRange) -> bool {
        if let Some(line_range) = rest_range.front() {
            starts_with(src, "#", *line_range)
                || line_range.is_empty()
                || starts_with(src, "```", *line_range)
                || starts_with(src, "$$", *line_range)
                || MARK_TO_THEOREM_KIND
                    .iter()
                    .any(|(mark, _)| starts_with(src, mark, *line_range))
                || starts_with(src, "Proof.", *line_range)
                || starts_with(src, "proof.", *line_range)
                || starts_with(src, "- ", *line_range)
                || starts_with(src, "> ", *line_range)
        } else {
            true
        }
    }

    ParseBlockElementResult {
        value: BlockElement::Paragraph {
            content: inline_elements,
        },
        errors,
        warnings,
        rest_range,
    }
}

fn lift_block_range(
    src: &Vec<Vec<char>>,
    prefix: &str,
    range: BlockRange,
) -> ParseResult<BlockRange, BlockRange> {
    let mut rest_range = range;
    let mut lifted_range = BlockRange::new();
    loop {
        if let Some(line) = rest_range.pop_front() {
            if starts_with(src, prefix, line) {
                lifted_range.push_back(line.consume(prefix.len()));
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
    src: &Vec<Vec<char>>,
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

fn parse_inline_element(
    src: &Vec<Vec<char>>,
    mut rest_range: InlineRange,
) -> ParseInlineElementResult {
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

    let start_column = rest_range.start_column;
    let errors = vec![];
    let warnings = vec![];
    while !rest_range.is_empty() {
        if !parse_inline_math_element(src, &rest_range)
            .value
            .is_parse_error()
            || !parse_inline_code_element(src, &rest_range)
                .value
                .is_parse_error()
        {
            let inline_range = InlineRange {
                line: rest_range.line,
                start_column,
                end_column: rest_range.start_column,
            };
            return ParseInlineElementResult {
                value: InlineElement::Text {
                    range: inline_range,
                },
                errors,
                warnings,
                rest_range,
            };
        }
        rest_range.move_to_next_char();
    }

    ParseInlineElementResult {
        value: InlineElement::Text {
            range: InlineRange {
                line: rest_range.line,
                start_column,
                end_column: rest_range.start_column,
            },
        },
        errors,
        warnings,
        rest_range,
    }
}

fn parse_inline_math_element(
    src: &Vec<Vec<char>>,
    rest_range: &InlineRange,
) -> ParseInlineElementResult {
    let mut rest_range = *rest_range;
    let errors = vec![];
    let warnings = vec![];
    if let Some('$') = pick_char(src, &rest_range) {
        rest_range.move_to_next_char();
        let inline_math_start_column = rest_range.start_column;
        loop {
            match pick_char(src, &rest_range) {
                Some('$') => {
                    let inline_math_range = InlineRange {
                        line: rest_range.line,
                        start_column: inline_math_start_column,
                        end_column: rest_range.start_column,
                    };
                    rest_range.move_to_next_char();
                    return ParseInlineElementResult {
                        value: InlineElement::Math {
                            range: inline_math_range,
                        },
                        errors,
                        warnings,
                        rest_range,
                    };
                }
                Some(_) => {
                    rest_range.move_to_next_char();
                }
                None => {
                    return ParseInlineElementResult {
                        value: InlineElement::ParseError,
                        errors,
                        warnings,
                        rest_range,
                    };
                }
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

fn parse_inline_code_element(
    src: &Vec<Vec<char>>,
    rest_range: &InlineRange,
) -> ParseInlineElementResult {
    let mut rest_range = *rest_range;
    let errors = vec![];
    let warnings = vec![];
    if let Some('`') = pick_char(src, &rest_range) {
        rest_range.move_to_next_char();
        let inline_code_start_column = rest_range.start_column;
        loop {
            match pick_char(src, &rest_range) {
                Some('`') => {
                    let inline_code_range = InlineRange {
                        line: rest_range.line,
                        start_column: inline_code_start_column,
                        end_column: rest_range.start_column,
                    };
                    rest_range.move_to_next_char();
                    return ParseInlineElementResult {
                        value: InlineElement::Code {
                            range: inline_code_range,
                        },
                        errors,
                        warnings,
                        rest_range,
                    };
                }
                Some(_) => {
                    rest_range.move_to_next_char();
                }
                None => {
                    return ParseInlineElementResult {
                        value: InlineElement::ParseError,
                        errors,
                        warnings,
                        rest_range,
                    };
                }
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

fn parse_inline_small_caps_element(
    src: &Vec<Vec<char>>,
    rest_range: &InlineRange,
) -> ParseInlineElementResult {
    let mut rest_range = *rest_range;
    let errors = vec![];
    let warnings = vec![];
    if let Some('%') = pick_char(src, &rest_range) {
        rest_range.move_to_next_char();
        let inline_start_column = rest_range.start_column;
        loop {
            match pick_char(src, &rest_range) {
                Some('%') => {
                    let inline_range = InlineRange {
                        line: rest_range.line,
                        start_column: inline_start_column,
                        end_column: rest_range.start_column,
                    };
                    rest_range.move_to_next_char();
                    return ParseInlineElementResult {
                        value: InlineElement::SmallCaps {
                            range: inline_range,
                        },
                        errors,
                        warnings,
                        rest_range,
                    };
                }
                Some(_) => {
                    rest_range.move_to_next_char();
                }
                None => {
                    return ParseInlineElementResult {
                        value: InlineElement::ParseError,
                        errors,
                        warnings,
                        rest_range,
                    };
                }
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

fn pick_char(src: &Vec<Vec<char>>, range: &InlineRange) -> Option<char> {
    if let Some(line) = src.iter().nth(range.line) {
        if let Some(c) = line.iter().nth(range.start_column) {
            Some(*c)
        } else {
            None
        }
    } else {
        None
    }
}

fn check_at(src: &Vec<Vec<char>>, expected: char, range: &InlineRange) -> bool {
    if let Some(actual) = pick_char(src, range) {
        expected == actual
    } else {
        false
    }
}

fn starts_with(src: &Vec<Vec<char>>, expected: &str, mut range: InlineRange) -> bool {
    expected.chars().all(|c| {
        let res = check_at(src, c, &range);
        if range.is_empty() {
            return false;
        }
        range.move_to_next_char();
        res
    })
}

fn match_(src: &Vec<Vec<char>>, expected: &str, range: InlineRange) -> bool {
    expected.len() == range.end_column - range.start_column && starts_with(src, expected, range)
}
