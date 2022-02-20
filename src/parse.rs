use crate::{document::*, util};

pub fn parse(src: &Vec<Vec<char>>, mut range: Range) -> Document {
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.column, 0);
    assert_eq!(range.end.line, src.len());
    assert_eq!(range.end.column, 0);
    let whole_range = range.clone();
    eprintln!("parse document");

    let mut block_elements = vec![];
    while range.start != range.end {
        assert!(range.start.is_head_of_line(src));
        // 空行は無視する
        loop {
            if range.start == range.end {
                break;
            }
            if !range.start.is_tail_of_line(src) {
                break;
            }
            range.start.move_to_next_line();
        }

        if range.start == range.end {
            break;
        }
        let (block_element, rest_range) = parse_block_element(src, range);
        block_elements.push(block_element);
        range = rest_range;
    }
    Document {
        range: whole_range,
        block_elements,
    }
}

fn parse_block_element(src: &Vec<Vec<char>>, range: Range) -> (BlockElement, Range) {
    assert!(range.start != range.end); // ソースの終端でない
    assert!(!range.start.is_tail_of_line(src)); // 空行ではない

    if check_at(src, '#', &range.start) {
        parse_heading(src, range)
    } else {
        parse_paragraph(src, range)
    }
}

fn parse_heading(src: &Vec<Vec<char>>, mut range: Range) -> (BlockElement, Range) {
    assert!(range.start != range.end); // ソースの終端でない
    assert!(!range.start.is_tail_of_line(src)); // 空行ではない
    assert!(check_at(src, '#', &range.start)); // '#' で始まる

    eprintln!("  parse heading");

    let mut level_count = 0;
    while check_at(src, '#', &range.start) {
        range.start.move_to_next_char();
        level_count += 1;
    }

    let mut span_elements = vec![];
    while !range.start.is_tail_of_line(src) {
        let (span_element, rest_range) = parse_span_element(src, range);
        span_elements.push(span_element);
        range = rest_range;
    }
    range.start.move_to_next_line();
    (
        BlockElement::Heading {
            level: level_count,
            content: span_elements,
        },
        range,
    )
}

fn parse_paragraph(src: &Vec<Vec<char>>, mut range: Range) -> (BlockElement, Range) {
    assert!(range.start != range.end); // ソースの終端でない
    assert!(!range.start.is_tail_of_line(src)); // 空行ではない

    eprintln!("  parse paragraph");
    let mut span_elements = vec![];
    loop {
        let (span_element, rest_range) = parse_span_element(src, range);
        span_elements.push(span_element);
        range = rest_range;

        if range.start.is_head_of_line(src) {
            let is_another_paragraph = range.start.is_tail_of_line(src); // 空行である
            let is_heading = check_at(src, '#', &range.start);
            if is_another_paragraph || is_heading {
                break;
            }
        }
    }
    (
        BlockElement::Paragraph {
            content: span_elements,
        },
        range,
    )
}

fn parse_span_element(src: &Vec<Vec<char>>, mut range: Range) -> (SpanElement, Range) {
    let mut span_range = Range {
        start: range.start.clone(),
        end: range.start.clone(),
    };
    // 現状は一行はすべて text span element になる
    while !range.start.is_tail_of_line(src) {
        range.start.move_to_next_char();
    }
    span_range.end = range.start.clone();
    range.start.move_to_next_line();
    (SpanElement::Text { range: span_range }, range)
}

fn pick_char(src: &Vec<Vec<char>>, loc: &Location) -> Option<char> {
    if let Some(line) = src.iter().nth(loc.line) {
        if let Some(c) = line.iter().nth(loc.column) {
            Some(*c)
        } else {
            None
        }
    } else {
        None
    }
}

fn check_at(src: &Vec<Vec<char>>, expected: char, loc: &Location) -> bool {
    if let Some(actual) = pick_char(src, loc) {
        expected == actual
    } else {
        false
    }
}
