use super::*;

use std::collections::HashMap;

// single line vs multi line
// initial whole range, initial buffer, initial text, updated buffer, updated text

const SINGLE_LINE_TEXT: &'static str = "hoge\n";
const MULTI_LINE_TEXT: &'static str = "hoge\nfuga\npiyo";

#[test]
fn single_line_initial_whole_range_test() {
    let src = Source::from_text(SINGLE_LINE_TEXT.to_string());
    let src_range = src.whole_range();
    assert_eq!(
        src_range,
        vec![InlineRange {
            cursor: Cursor {
                node_id: src.start_node_id,
                pos_on_node: 0,
            },
            length: 4,
        }]
    );
}

#[test]
fn single_line_initial_buffer_test() {
    let src = Source::from_text(SINGLE_LINE_TEXT.to_string());
    assert_eq!(
        src.buffers,
        vec![Buffer {
            value: SINGLE_LINE_TEXT.chars().collect(),
        }]
    );
}

#[test]
fn single_line_initial_text_test() {
    let src = Source::from_text(SINGLE_LINE_TEXT.to_string());
    assert_eq!(SINGLE_LINE_TEXT, &src.to_text());
}

#[test]
fn single_line_updated_buffer_test() {
    let text = SINGLE_LINE_TEXT.to_string();
    let mut src = Source::from_text(text);

    // update: hoge\n -> hiuge\n
    src.update(vec![UpdateInfo {
        start: (0, 1),
        end: (0, 2),
        text: "iu".to_string(),
    }]);
    assert_eq!(
        vec![
            Buffer {
                value: "hoge\n".chars().collect(),
            },
            Buffer {
                value: "iu".chars().collect(),
            }
        ],
        src.buffers
    );
    assert_eq!(
        vec![
            (
                0,
                Node {
                    buffer_id: 0,
                    start_pos_on_buffer: 0,
                    length: 1,
                }
            ),
            (
                1,
                Node {
                    buffer_id: 1,
                    start_pos_on_buffer: 0,
                    length: 2,
                }
            ),
            (
                2,
                Node {
                    buffer_id: 0,
                    start_pos_on_buffer: 2,
                    length: 3
                }
            )
        ]
        .into_iter()
        .collect::<HashMap<_, _>>(),
        src.nodes,
    );

    // update: hiuge\n => fuugepiyo\n
    // update: h|iu|ge\n => h|iu|ge|piyo|\n
    src.update(vec![
        /*UpdateInfo {
            start: (0, 0),
            end: (0, 2),
            text: "fu".to_string(),
        }, */
        UpdateInfo {
            start: (0, 5),
            end: (0, 5),
            text: "piyo".to_string(),
        },
    ]);
    assert_eq!(
        vec![
            Buffer {
                value: "hoge\n".chars().collect(),
            },
            Buffer {
                value: "iu".chars().collect(),
            },
            /* Buffer {
                value: "fu".chars().collect()
            }, */
            Buffer {
                value: "piyo".chars().collect()
            }
        ],
        src.buffers
    );
    // buffers: ["hoge\n", "iu", "piyo"]
    // update: h|iu|ge\n => h|iu|ge|piyo|\n
    assert_eq!(
        vec![
            (
                0,
                Node {
                    buffer_id: 0,
                    start_pos_on_buffer: 0,
                    length: 1,
                }
            ),
            (
                1,
                Node {
                    buffer_id: 1,
                    start_pos_on_buffer: 0,
                    length: 2,
                }
            ),
            (
                2,
                Node {
                    buffer_id: 0,
                    start_pos_on_buffer: 2,
                    length: 2
                }
            ),
            (
                3,
                Node {
                    buffer_id: 2,
                    start_pos_on_buffer: 0,
                    length: 4
                }
            ),
            (
                4,
                Node {
                    buffer_id: 0,
                    start_pos_on_buffer: 4,
                    length: 1
                }
            )
        ]
        .into_iter()
        .collect::<HashMap<_, _>>(),
        src.nodes,
    );
}

#[test]
fn single_line_updated_text_test() {
    let text = SINGLE_LINE_TEXT.to_string();
    let mut src = Source::from_text(text);

    // update: hoge -> hige
    src.update(vec![UpdateInfo {
        start: (0, 1),
        end: (0, 2),
        text: "i".to_string(),
    }]);
    assert_eq!("hige\n".to_string(), src.to_text());

    // update: hige => fugepiyo
    src.update(vec![
        UpdateInfo {
            start: (0, 0),
            end: (0, 2),
            text: "fu".to_string(),
        },
        UpdateInfo {
            start: (0, 4),
            end: (0, 4),
            text: "piyo".to_string(),
        },
    ]);
    assert_eq!("fugepiyo\n".to_string(), src.to_text());
}

#[test]
fn multi_line_initial_whole_range_test() {
    let text = MULTI_LINE_TEXT.to_string();
    let src = Source::from_text(text.clone());
    let src_range = src.whole_range();
    assert_eq!(
        src_range,
        vec![
            InlineRange {
                cursor: Cursor {
                    node_id: src.start_node_id,
                    pos_on_node: 0,
                },
                length: 4,
            },
            InlineRange {
                cursor: Cursor {
                    node_id: src.start_node_id,
                    pos_on_node: 5,
                },
                length: 4,
            },
            InlineRange {
                cursor: Cursor {
                    node_id: src.start_node_id,
                    pos_on_node: 10
                },
                length: 4,
            },
        ]
    );
}

#[test]
fn multi_line_initial_buffer_test() {
    let src = Source::from_text(MULTI_LINE_TEXT.to_string());
    assert_eq!(
        src.buffers,
        vec![Buffer {
            value: "hoge\nfuga\npiyo\n".chars().collect(),
        }]
    );
}

#[test]
fn multi_line_updated_buffer_test() {
    todo!()
}

#[test]
fn multi_line_updated_text_buffer() {
    let text = MULTI_LINE_TEXT.to_string();
    let mut src = Source::from_text(text);

    // update: [hoge, fuga, piyo] -> [hoge, figa, piyo]
    src.update(vec![UpdateInfo {
        start: (1, 1),
        end: (1, 2),
        text: "i".to_string(),
    }]);
    assert_eq!("hoge\nfiga\npiyo\n".to_string(), src.to_text());

    // update: [hoge, figa, piyo] => [hoge, hogapiyo, piyo]
    src.update(vec![
        /* UpdateInfo {
            start: (1, 0),
            end: (1, 2),
            text: "ho".to_string(),
        }, */
        UpdateInfo {
            start: (1, 4),
            end: (1, 4),
            text: "piyo".to_string(),
        },
    ]);
    assert_eq!("hoge\nfigapiyo\npiyo\n".to_string(), src.to_text());

    // update: [hoge, figa, piyo] => [hoge, hogapiyo, piyo]
    src.update(vec![UpdateInfo {
        start: (1, 0),
        end: (1, 2),
        text: "ho".to_string(),
    }]);
    assert_eq!("hoge\nhogapiyo\npiyo\n".to_string(), src.to_text());

    // update: [hoge, hogapiyo, piyo] => [nege, hogapiyo, piyo]
    src.update(vec![UpdateInfo {
        start: (0, 0),
        end: (0, 2),
        text: "ne".to_string(),
    }]);
    assert_eq!("nege\nhogapiyo\npiyo\n".to_string(), src.to_text());

    // update: [nege, hogapiyo, piyo] => [nege, hogapiyo, hoyo]
    src.update(vec![UpdateInfo {
        start: (2, 0),
        end: (2, 2),
        text: "ho".to_string(),
    }]);
    assert_eq!("nege\nhogapiyo\nhoyo\n".to_string(), src.to_text());
}

#[test]
fn cursor_test() {
    let src = Source::from_text(MULTI_LINE_TEXT.to_string());
    let mut cursor: Cursor = src.whole_range().get(1).unwrap().clone().cursor;
    assert_eq!(cursor.node_id, 0);
    assert_eq!(cursor.pos_on_node, 5);

    assert_eq!(Some('f'), cursor.peek(&src));
    assert_eq!(Some('f'), cursor.next(&src));
    assert_eq!(Some('u'), cursor.next(&src));

    assert!(cursor.starts_with(&src, "ga"));
    assert!(cursor.starts_with(&src, "ga\npiyo")); // ノードをまたぐ場合

    assert_eq!((1, 2), cursor.calc_point_on_source(&src));
}
