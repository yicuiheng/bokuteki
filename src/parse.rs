use crate::document::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;

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

pub fn parse_document(
    src: &Vec<Vec<char>>,
    src_range: BlockRange,
) -> ParseResult<Document, BlockRange> {
    let mut rest_range = src_range;
    let mut preamble = HashMap::new();
    let mut imports = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];

    // 空行は無視する
    loop {
        if let Some(top_line_range) = rest_range.front() {
            if starts_with(src, "%", *top_line_range) {
                let mut preamble_result = parse_preamble(src, *top_line_range);
                let v = preamble_result.value;
                preamble.insert(v.0, v.1);
                errors.append(&mut preamble_result.errors);
                warnings.append(&mut preamble_result.warnings);
                rest_range.pop_front();
            } else {
                break;
            }
        } else {
            return ParseResult {
                value: Document {
                    preamble,
                    block_elements: vec![],
                    imports: vec![],
                },
                errors,
                warnings,
                rest_range,
            };
        }
    }

    // 空行は無視する
    loop {
        if let Some(top_line_range) = rest_range.front() {
            if starts_with(src, IMPORT_KEYWORD, top_line_range.clone()) {
                let mut import_result = parse_import(src, top_line_range.clone());
                imports.push(import_result.value);
                errors.append(&mut import_result.errors);
                warnings.append(&mut import_result.warnings);
                rest_range.pop_front();
            } else {
                break;
            }
        } else {
            return ParseResult {
                value: Document {
                    preamble,
                    block_elements: vec![],
                    imports,
                },
                errors,
                warnings,
                rest_range,
            };
        }
    }

    let mut block_elements_result = parse_block_elements(src, rest_range);
    errors.append(&mut block_elements_result.errors);
    warnings.append(&mut block_elements_result.warnings);
    let rest_range = block_elements_result.rest_range;
    ParseResult {
        value: Document {
            preamble,
            block_elements: block_elements_result.value,
            imports,
        },
        errors,
        warnings,
        rest_range,
    }
}

fn parse_preamble(
    src: &Vec<Vec<char>>,
    inline_range: InlineRange,
) -> ParseResult<(String, String), InlineRange> {
    assert!(starts_with(src, "%", inline_range));
    let mut key = String::new();
    let mut value = String::new();
    let mut rest_range = inline_range.consume("%".len());

    // key のパース
    loop {
        match pick_char(src, &rest_range) {
            Some(c) if c.is_ascii_whitespace() => {
                rest_range.move_to_next_char();
                break;
            }
            Some(c) => {
                key.push(c);
                rest_range.move_to_next_char();
            }
            None => {
                return ParseResult {
                    value: (key, String::new()),
                    errors: vec![format!(
                        "at {}:{}: expected preamble value",
                        rest_range.line, rest_range.start_column
                    )],
                    warnings: vec![],
                    rest_range,
                }
            }
        }
    }

    // value のパース
    loop {
        match pick_char(src, &rest_range) {
            Some(c) => {
                value.push(c);
                rest_range.move_to_next_char();
            }
            None => {
                return ParseResult {
                    value: (key, value),
                    errors: vec![],
                    warnings: vec![],
                    rest_range,
                }
            }
        }
    }
}

const IMPORT_KEYWORD: &str = "import";
fn parse_import(
    src: &Vec<Vec<char>>,
    inline_range: InlineRange,
) -> ParseResult<PathBuf, InlineRange> {
    assert!(starts_with(src, IMPORT_KEYWORD, inline_range));
    let mut path = PathBuf::new();
    let mut rest_range = inline_range.consume(IMPORT_KEYWORD.len());

    // シングルクォートが来るまで空白を読み飛ばす
    // シングルクォートや空白以外が来たり終端が来たらエラーにする
    loop {
        match pick_char(src, &rest_range) {
            Some(c) if c.is_ascii_whitespace() => {
                rest_range.move_to_next_char();
            }
            Some('\'') => {
                rest_range.move_to_next_char();
                break;
            }
            Some(c) => {
                return ParseResult {
                    value: path,
                    errors: vec![format!(
                        "at {}:{}: expected single quote ('), but actual is '{}'.",
                        rest_range.line, rest_range.start_column, c
                    )],
                    warnings: vec![],
                    rest_range,
                }
            }
            None => {
                return ParseResult {
                    value: path,
                    errors: vec![format!(
                        "at {}:{}: expected single quote (').",
                        rest_range.line, rest_range.start_column
                    )],
                    warnings: vec![],
                    rest_range,
                }
            }
        }
    }

    // シングルクォートが来るまでインポートのパスを読み取る
    // '/' もしくは '\' が来たら push する
    // 上記以外でローマアルファベット、数字、ハイフン ('-')、アンダースコア ('_') が来たらパス文字列に追加する
    // 上記以外もしくは終端が来たらエラーにする
    let mut buf = String::new();
    loop {
        match pick_char(src, &rest_range) {
            Some('\'') => {
                rest_range.move_to_next_char();
                path.push(buf);
                break;
            }
            Some('/') | Some('\\') => {
                rest_range.move_to_next_char();
                path.push(buf);
                buf = String::new();
            }
            Some(c) if c.is_ascii_alphanumeric() || c == '-' || c == '_' => {
                rest_range.move_to_next_char();
                buf.push(c);
            }
            Some(c) => {
                return ParseResult {
                    value: PathBuf::new(),
                    errors: vec![format!(
                        "at {}:{}: '{}' is invalid character as imported path.",
                        rest_range.line, rest_range.start_column, c
                    )],
                    warnings: vec![],
                    rest_range,
                }
            }
            None => {
                return ParseResult {
                    value: PathBuf::new(),
                    errors: vec![format!(
                        "at {}:{}: expected single quote (').",
                        rest_range.line, rest_range.start_column
                    )],
                    warnings: vec![],
                    rest_range,
                }
            }
        }
    }

    // セミコロンが来るまで空白を読み飛ばす
    // セミコロンや空白以外が来たり終端が来たらエラーにする
    loop {
        match pick_char(src, &rest_range) {
            Some(c) if c.is_ascii_whitespace() => {
                rest_range.move_to_next_char();
            }
            Some(';') => {
                rest_range.move_to_next_char();
                break;
            }
            Some(c) => {
                return ParseResult {
                    value: path,
                    errors: vec![format!(
                        "at {}:{}: expected semicolon (';'), but actual is '{}'.",
                        rest_range.line, rest_range.start_column, c
                    )],
                    warnings: vec![],
                    rest_range,
                }
            }
            None => {
                return ParseResult {
                    value: path,
                    errors: vec![format!(
                        "at {}:{}: expected semicolon (';').",
                        rest_range.line, rest_range.start_column
                    )],
                    warnings: vec![],
                    rest_range,
                }
            }
        }
    }

    // 残りは空白のみであることを確認する
    // 空白以外が来たらエラーにする
    loop {
        match pick_char(src, &rest_range) {
            Some(c) if c.is_ascii_whitespace() => {
                rest_range.move_to_next_char();
            }
            None => {
                break;
            }
            Some(c) => {
                return ParseResult {
                    value: path,
                    errors: vec![format!(
                        "at {}:{}: unexpected character '{}'.",
                        rest_range.line, rest_range.start_column, c
                    )],
                    warnings: vec![],
                    rest_range,
                }
            }
        }
    }

    ParseResult {
        value: path,
        errors: vec![],
        warnings: vec![],
        rest_range,
    }
}

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
                    rest_range,
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

fn parse_derivation_block_element(
    src: &Vec<Vec<char>>,
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
            let inline_elements = parse_inline_elements(src, line_range);
            premises.push(Derivation::Leaf(inline_elements.value));
        }
    }
    rest_range = premises_range_result.rest_range;

    let mut rule_name_result = if let Some(mut line_range) = rest_range.pop_front() {
        let mut count = 0;
        while check_at(src, '-', &line_range) {
            count += 1;
            line_range.move_to_next_char();
        }
        if count >= 3 {
            parse_inline_elements(src, line_range)
        } else {
            return parse_error;
        }
    } else {
        return parse_error;
    };
    errors.append(&mut rule_name_result.errors);
    warnings.append(&mut rule_name_result.warnings);
    let rule_name = rule_name_result.value;

    let mut conclusion_range_result = lift_block_range(src, "  ", rest_range);
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
        }),
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

    let head_line = rest_range.front().cloned();

    // ソース終端もしくは別のブロック要素の始まりまで Inline 要素をパースする
    while !is_paragraph_end(src, &rest_range, &head_line) {
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
    // ただしコードブロックの終端マーク ("```") もしくは 数式ブロックの終端マーク ("$$") が1行目に出現した場合は、それは段落の終わりではない。
    fn is_paragraph_end(
        src: &Vec<Vec<char>>,
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
            starts_with(src, "#", *line_range)
                || line_range.is_empty()
                || if let Some(head_line) = head_line {
                    if head_line != line_range {
                        starts_with(src, "```", *line_range) || starts_with(src, "$$", *line_range)
                    } else {
                        false
                    }
                } else {
                    false
                }
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
    let result = parse_inline_link_element(src, &rest_range);
    if !result.value.is_parse_error() {
        return result;
    }

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
        if !parse_inline_link_element(src, &rest_range)
            .value
            .is_parse_error()
            || !parse_inline_math_element(src, &rest_range)
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

fn parse_inline_link_element(
    src: &Vec<Vec<char>>,
    rest_range: &InlineRange,
) -> ParseInlineElementResult {
    let mut rest_range = rest_range.clone();
    let mut errors = vec![];
    let mut warnings = vec![];
    if let Some('[') = pick_char(src, &rest_range) {
        rest_range.move_to_next_char();
    } else {
        return ParseInlineElementResult {
            value: InlineElement::ParseError,
            errors,
            warnings,
            rest_range,
        };
    }

    let text_start_column = rest_range.start_column;
    loop {
        match pick_char(src, &rest_range) {
            Some(']') => {
                break;
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
                }
            }
        }
    }
    let text_end_column = rest_range.start_column;
    rest_range.move_to_next_char();
    let mut text_result = parse_inline_elements(
        src,
        InlineRange {
            line: rest_range.line,
            start_column: text_start_column,
            end_column: text_end_column,
        },
    );
    errors.append(&mut text_result.errors);
    warnings.append(&mut text_result.warnings);
    if !text_result.rest_range.is_empty() {
        return ParseInlineElementResult {
            value: InlineElement::ParseError,
            errors,
            warnings,
            rest_range,
        };
    }
    let text = text_result.value;

    if let Some('(') = pick_char(src, &rest_range) {
        rest_range.move_to_next_char();
    } else {
        return ParseInlineElementResult {
            value: InlineElement::ParseError,
            errors,
            warnings,
            rest_range,
        };
    }

    let url_start_column = rest_range.start_column;
    loop {
        match pick_char(src, &rest_range) {
            Some(')') => {
                break;
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
                }
            }
        }
    }
    let url_end_column = rest_range.start_column;
    rest_range.move_to_next_char();

    let url_range = InlineRange {
        line: rest_range.line,
        start_column: url_start_column,
        end_column: url_end_column,
    };

    ParseInlineElementResult {
        value: InlineElement::Link { text, url_range },
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
