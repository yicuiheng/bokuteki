pub mod range;

#[cfg(test)]
mod spec;

use range::{BlockRange, InlineRange};
use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::path::Path;

// https://code.visualstudio.com/blogs/2018/03/23/text-buffer-reimplementation
pub struct Source {
    buffers: Vec<Buffer>,
    nodes: HashMap<usize, Node>,
    next_map: HashMap<usize, usize>,
    start_node_id: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Cursor {
    node_id: usize,
    pos_on_node: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UpdateInfo {
    pub start: (u32, u32),
    pub end: (u32, u32),
    pub text: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Buffer {
    value: Vec<char>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Node {
    buffer_id: usize,
    start_pos_on_buffer: usize,
    length: usize,
}

impl Source {
    pub fn open(input_filepath: &Path) -> Self {
        let src = fs::read_to_string(input_filepath).expect("failed to read file..");
        Self::from_text(src)
    }

    pub fn from_text(text: String) -> Self {
        let mut src = vec![];
        for line in text.lines() {
            src.append(&mut line.chars().collect());
            src.push('\n');
        }
        let src = src;
        let src_length = src.len();
        Source {
            buffers: vec![Buffer { value: src }],
            nodes: vec![(
                0,
                Node {
                    buffer_id: 0,
                    start_pos_on_buffer: 0,
                    length: src_length,
                },
            )]
            .into_iter()
            .collect(),
            next_map: HashMap::new(),
            start_node_id: 0,
        }
    }

    pub fn update(&mut self, infos: Vec<UpdateInfo>) {
        // 方針
        // 更新範囲が node をまたぐ場合とまたがない場合がある
        // - またがない場合:
        //   1つの node を 3つの node に分割することになる
        //   1. まずどの node を分割するのか調べる

        let mut line = 0;
        let mut chara = 0;

        let mut current_node_id = self.start_node_id;
        loop {
            let chars_in_node: Vec<char> = {
                let current_node = &self.nodes[&current_node_id];
                let buffer = &self.buffers[current_node.buffer_id];
                let start_idx = current_node.start_pos_on_buffer;
                let end_idx = current_node.start_pos_on_buffer + current_node.length;
                buffer.value[start_idx..end_idx].iter().cloned().collect()
            };
            let mut next_line = line;
            let mut next_chara = chara;
            for c in &chars_in_node {
                if *c == '\n' {
                    next_line += 1;
                    next_chara = 0;
                } else {
                    next_chara += 1;
                }
            }
            let next_node_id = self.next_map.get(&&current_node_id).cloned();

            for (idx, c) in chars_in_node.iter().enumerate() {
                for info in &infos {
                    if line == info.start.0 && chara == info.start.1 {
                        let new_buffer = Buffer {
                            value: info.text.chars().collect(),
                        };
                        let new_buffer_id = self.buffers.len();
                        self.buffers.push(new_buffer);
                        let new_node = Node {
                            buffer_id: new_buffer_id,
                            start_pos_on_buffer: 0,
                            length: info.text.len(),
                        };
                        let new_node_id = self.nodes.len();
                        self.nodes.insert(new_node_id, new_node);
                        let old_next = self.next_map.get(&current_node_id).cloned();

                        let mut current_node = self.nodes.get_mut(&current_node_id).unwrap();
                        let original_current_node_length = current_node.length;
                        current_node.length = idx;

                        let after_part_node = Node {
                            buffer_id: self.nodes[&current_node_id].buffer_id,
                            start_pos_on_buffer: self.nodes[&current_node_id].start_pos_on_buffer
                                + (info.end.1 as usize - info.start.1 as usize)
                                + idx,
                            length: original_current_node_length
                                - (info.end.1 as usize - info.start.1 as usize)
                                - idx,
                        };
                        let after_part_node_id = self.nodes.len();
                        self.nodes.insert(after_part_node_id, after_part_node);

                        self.next_map.insert(current_node_id, new_node_id);
                        self.next_map.insert(new_node_id, after_part_node_id);
                        if let Some(old_next) = old_next {
                            self.next_map.insert(after_part_node_id, old_next);
                        }
                    }
                }
                if *c == '\n' {
                    line += 1;
                    chara = 0;
                } else {
                    chara += 1;
                }
            }

            line = next_line;
            chara = next_chara;
            if let Some(next_node_id) = next_node_id {
                current_node_id = next_node_id;
            } else {
                break;
            }
        }
    }

    pub fn save(&self, output_path: &Path) {
        use std::io::{BufWriter, Write};
        let file = File::create(output_path).expect("failed to create output file..");
        let mut writer = BufWriter::new(file);
        let mut current_node_id = self.start_node_id;
        while {
            let current_node = &self.nodes[&current_node_id];
            let start_idx = current_node.start_pos_on_buffer;
            let end_idx = current_node.start_pos_on_buffer + current_node.length;
            let content: String = self.buffers[current_node.buffer_id].value[start_idx..end_idx]
                .into_iter()
                .collect();
            write!(writer, "{}", content).expect("failed to write to output file..");

            if let Some(next_node_id) = self.next_map.get(&current_node_id) {
                current_node_id = *next_node_id;
                true
            } else {
                false
            }
        } {}
        writer.flush().expect("failed to flush..");
    }

    #[cfg(test)]
    pub fn to_text(&self) -> String {
        let mut result = String::new();
        let mut current_node_id = self.start_node_id;
        while {
            let current_node = &self.nodes[&current_node_id];
            let start_idx = current_node.start_pos_on_buffer;
            let end_idx = current_node.start_pos_on_buffer + current_node.length;
            let content: String = self.buffers[current_node.buffer_id].value[start_idx..end_idx]
                .into_iter()
                .collect();
            result += content.as_str();

            if let Some(next_node_id) = self.next_map.get(&current_node_id) {
                current_node_id = *next_node_id;
                true
            } else {
                false
            }
        } {}
        result
    }

    // precondition: Source::open や Source::from_text で生成した直後である
    pub fn whole_range(&self) -> BlockRange {
        assert_eq!(self.buffers.len(), 1);
        let buffer = &self.buffers[0];
        assert_eq!(self.nodes.len(), 1);
        assert_eq!(self.start_node_id, 0);
        let mut block_range = VecDeque::new();
        let mut line_start_idx = 0;
        let mut chara = 0;
        for (idx, c) in buffer.value.iter().enumerate() {
            if *c == '\n' {
                block_range.push_back(InlineRange {
                    cursor: Cursor {
                        node_id: 0,
                        pos_on_node: line_start_idx,
                    },
                    length: chara,
                });
                line_start_idx = idx + 1;
                chara = 0;
            } else {
                chara += 1;
            }
        }
        block_range
    }
}

impl Cursor {
    pub fn peek(&self, src: &Source) -> Option<char> {
        let current_node = &src.nodes[&self.node_id];
        let buffer = &src.buffers[current_node.buffer_id];
        let idx = current_node.start_pos_on_buffer + self.pos_on_node;
        buffer.value.get(idx).cloned()
    }

    pub fn next(&mut self, src: &Source) -> Option<char> {
        let current_node = &src.nodes[&self.node_id];
        match self.peek(src) {
            Some(c) => {
                if self.pos_on_node + 1 < current_node.length {
                    self.pos_on_node += 1;
                    Some(c)
                } else if let Some(next_node_id) = src.next_map.get(&self.node_id) {
                    // next node
                    self.node_id = *next_node_id;
                    self.pos_on_node = 0;
                    Some(c)
                } else {
                    // end of Source
                    self.pos_on_node += 1;
                    Some(c)
                }
            }
            None => None,
        }
    }

    pub fn starts_with(&self, src: &Source, text: &str) -> bool {
        let mut cursor = self.clone();
        for expected_char in text.chars() {
            match (cursor.next(src), src.next_map.get(&cursor.node_id)) {
                (Some(actual_char), _) => {
                    if expected_char != actual_char {
                        return false;
                    }
                }
                (None, Some(next_node_id)) => {
                    cursor = Cursor {
                        node_id: *next_node_id,
                        pos_on_node: 0,
                    };
                    let actual_char = cursor.next(src).unwrap();
                    if expected_char != actual_char {
                        return false;
                    }
                }
                (None, None) => {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn calc_point_on_source(&self, src: &Source) -> (u32, u32) {
        let mut line = 0;
        let mut chara = 0;

        let mut current_node_id = src.start_node_id;
        while current_node_id != self.node_id {
            let current_node = &src.nodes[&current_node_id];
            let buffer = &src.buffers[current_node.buffer_id];
            let start_idx = current_node.start_pos_on_buffer;
            let end_idx = current_node.start_pos_on_buffer + current_node.length;
            for c in &buffer.value[start_idx..end_idx] {
                if *c == '\n' {
                    line += 1;
                    chara = 0;
                } else {
                    chara += 1;
                }
            }
            current_node_id = *src.next_map.get(&current_node_id).unwrap();
        }

        let node = &src.nodes[&self.node_id];
        let buffer = &src.buffers[node.buffer_id];
        let start_idx = node.start_pos_on_buffer;
        let end_idx = node.start_pos_on_buffer + self.pos_on_node;
        for c in &buffer.value[start_idx..end_idx] {
            if *c == '\n' {
                line += 1;
                chara = 0;
            } else {
                chara += 1;
            }
        }
        return (line, chara);
    }
}
