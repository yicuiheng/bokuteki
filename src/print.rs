use crate::document::*;
use std::collections::HashMap;
use std::{env, fs, path::Path};

pub fn print(src: &Vec<Vec<char>>, doc: Vec<BlockElement>) {
    let bokuteki_config_path_string = env::var("BOKUTEKI_CONFIG_PATH")
        .expect("env variable `$BOKUTEKI_CONFIG_PATH` is not defined.");
    let config_path = Path::new(&bokuteki_config_path_string);
    let template_path = config_path.join("template");
    let output_path = Path::new("./output");
    if output_path.exists() {
        fs::remove_dir_all(output_path).expect("failed to clean output directory..");
    }
    fs::create_dir(output_path).expect("failed to create output directory..");
    assert!(template_path.is_dir());
    assert!(output_path.is_dir());

    fs::copy(
        template_path.join("bokuteki.css"),
        output_path.join("bokuteki.css"),
    )
    .expect("failed to copy bokuteki.css");
    fs::copy(
        template_path.join("bokuteki.js"),
        output_path.join("bokuteki.js"),
    )
    .expect("failed to copy bokuteki.js");

    let template_string = fs::read_to_string(template_path.join("index.template.html"))
        .expect("failed to read index.template.html");

    let body_string: String = print_block_elements(src, doc, 4, true);

    let html_string = template_string.replace("{body-string}", &body_string);
    let mut html_file =
        fs::File::create(output_path.join("index.html")).expect("faild to create index.html");
    use std::io::Write;
    write!(html_file, "{}", html_string).expect("failed to write out html content..")
}

fn print_block_elements(
    src: &Vec<Vec<char>>,
    block_elements: Vec<BlockElement>,
    indent_depth: usize,
    needs_margin: bool,
) -> String {
    block_elements
        .into_iter()
        .map(|block_element| print_block_element(src, block_element, indent_depth, needs_margin))
        .collect::<Vec<_>>()
        .join("\n")
}

fn print_html_tag(
    tag_name: &str,
    attributes: HashMap<&str, &str>,
    inner: String,
    indent_depth: usize,
) -> String {
    let indent: String = std::iter::repeat(" ").take(indent_depth).collect();
    let mut attributes_str = String::new();
    for (attr_name, value) in attributes {
        attributes_str += &format!(r#" {}="{}""#, attr_name, value);
    }
    format!(
        r#"{indent}<{tag_name}{attributes_str}>
{inner}
{indent}</{tag_name}>"#
    )
}

fn print_block_element(
    src: &Vec<Vec<char>>,
    block_element: BlockElement,
    indent_depth: usize,
    needs_margin: bool,
) -> String {
    let mut attributes: HashMap<_, _> = if needs_margin {
        vec![("class", "block")].into_iter().collect()
    } else {
        HashMap::new()
    };
    match block_element {
        BlockElement::Heading { level, content } => {
            let content = print_inline_elements(src, content, indent_depth + 4);
            let tag_name = format!("h{}", level);
            print_html_tag(&tag_name, attributes, content, indent_depth)
        }
        BlockElement::Paragraph { content } => {
            let content = print_inline_elements(src, content, indent_depth + 4);
            print_html_tag("p", attributes, content, indent_depth)
        }
        BlockElement::Code { lines } => {
            let indent: String = std::iter::repeat(" ").take(indent_depth).collect();
            let inner = verbatim_block_content(src, &lines);
            let attributes = attributes
                .into_iter()
                .map(|(name, value)| format!(r#"{}="{}""#, name, value))
                .collect::<Vec<_>>()
                .join(" ");
            format!(
                r#"{indent}<pre {attributes}>
{inner}</pre>"#
            )
        }
        BlockElement::Math { lines } => {
            let indent: String = std::iter::repeat(" ").take(indent_depth).collect();
            let content = verbatim_block_content(src, &lines);
            format!(
                r#"{indent}$$
{content}
{indent}$$"#
            )
        }
        BlockElement::Theorem {
            kind: _kind,
            title,
            content,
        } => {
            let title = print_inline_elements(src, title, 0);
            let content = print_block_elements(src, content, indent_depth + 4, false);
            attributes.insert("class", "math-theorem");
            attributes.insert("data-title", &title);
            print_html_tag("div", attributes, content, indent_depth)
        }
        BlockElement::Proof { content } => {
            let content = print_block_elements(src, content, indent_depth + 4, false);
            attributes.insert("class", "math-proof");
            print_html_tag("div", attributes, content, indent_depth)
        }
        BlockElement::List {
            mark_kind: _mark_kind,
            items,
        } => {
            let items = items
                .into_iter()
                .map(|item| {
                    let top_line = print_inline_elements(src, item.top_line, indent_depth + 8);
                    let blocks = print_block_elements(src, item.blocks, indent_depth + 8, false);
                    let content = if blocks.is_empty() {
                        top_line
                    } else {
                        format!("{}\n{}", top_line, blocks)
                    };
                    print_html_tag("li", HashMap::new(), content, indent_depth + 4)
                })
                .collect::<Vec<_>>()
                .join("\n");

            print_html_tag("ul", attributes, items, indent_depth)
        }
        BlockElement::Blockquote { inner } => {
            let inner = print_block_elements(src, inner, indent_depth + 4, true);
            print_html_tag("blockquote", attributes, inner, indent_depth)
        }
        BlockElement::ParseError => "parse error..".to_string(),
    }
}

fn print_inline_elements(
    src: &Vec<Vec<char>>,
    inline_elements: Vec<InlineElement>,
    indent_depth: usize,
) -> String {
    let indent: String = std::iter::repeat(" ").take(indent_depth).collect();
    let line = inline_elements
        .into_iter()
        .map(|inline_element| print_inline_element(src, inline_element))
        .collect::<Vec<_>>()
        .join("");
    format!("{indent}{line}")
}

fn print_inline_element(src: &Vec<Vec<char>>, inline_element: InlineElement) -> String {
    match inline_element {
        InlineElement::Text { mut range } => {
            let mut result = String::new();
            while !range.is_empty() {
                let c: char = pick_char(src, &range).unwrap();
                result.push(c);
                range.move_to_next_char();
            }
            result
        }
        InlineElement::Math { mut range } => {
            let mut result = String::new();
            while !range.is_empty() {
                let c: char = pick_char(src, &range).unwrap();
                result.push(c);
                range.move_to_next_char();
            }
            format!("${}$", result)
        }
        InlineElement::Code { mut range } => {
            let mut result = String::new();
            while !range.is_empty() {
                let c: char = pick_char(src, &range).unwrap();
                result.push(c);
                range.move_to_next_char();
            }
            format!("<code>{}</code>", result)
        }
        InlineElement::SmallCaps { mut range } => {
            let mut result = String::new();
            while !range.is_empty() {
                let c: char = pick_char(src, &range).unwrap();
                result.push(c);
                range.move_to_next_char();
            }
            format!(r#"<span class="small-caps">{}</span>"#, result)
        }
        _ => unimplemented!(),
    }
}

fn verbatim_block_content(src: &Vec<Vec<char>>, range: &BlockRange) -> String {
    range
        .iter()
        .map(|line_range: &InlineRange| verbatim_inline_content(src, line_range))
        .collect::<Vec<_>>()
        .join("\n")
}

fn verbatim_inline_content(src: &Vec<Vec<char>>, range: &InlineRange) -> String {
    src[range.line][range.start_column..range.end_column]
        .into_iter()
        .collect()
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
