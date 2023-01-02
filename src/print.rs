use crate::document::*;
use crate::katex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Printer {
    template_path: PathBuf,
    output_path: PathBuf,
}

impl Printer {
    pub fn setup() -> Printer {
        // 環境変数から設定パスを取得
        let bokuteki_config_path_string = std::env::var("BOKUTEKI_CONFIG_PATH")
            .expect("env variable `$BOKUTEKI_CONFIG_PATH` is not defined.");
        let config_path = PathBuf::from(&bokuteki_config_path_string);
        let template_path = config_path.join("template");
        let output_path = PathBuf::from("./output");

        // 出力ディレクトリをクリーン
        if output_path.exists() {
            fs::remove_dir_all(&output_path).expect("failed to clean output directory..");
        }
        fs::create_dir(&output_path).expect("failed to create output directory..");

        assert!(template_path.is_dir());
        assert!(output_path.is_dir());

        // 共通ファイルを配置
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

        Printer {
            template_path,
            output_path,
        }
    }

    pub fn print(&self, src: &Vec<Vec<char>>, src_path: &Path, block_elements: Vec<BlockElement>) {
        // 出力する内容を構築
        let template_content = fs::read_to_string(self.template_path.join("template.html"))
            .expect("failed to read template.html");
        let relative_to_root = calc_relative_to_root(src_path);
        let body_content: String = print_block_elements(src, block_elements, 4, true);
        let css_path = relative_to_root.join("bokuteki.css");
        let js_path = relative_to_root.join("bokuteki.js");
        let html_content = template_content
            .replace("{body-string}", &body_content)
            .replace("{bokuteki-css-path}", &css_path.display().to_string())
            .replace("{bokuteki-js-path}", &js_path.display().to_string());

        // 出力する先を構築
        let mut html_path = self.output_path.join(src_path);
        html_path.set_extension("html");
        fs::create_dir_all(html_path.parent().unwrap()).unwrap();

        // 出力
        use std::io::Write;
        let mut html_file = fs::File::create(html_path).expect("faild to create html file");
        html_file
            .write_all(html_content.as_bytes())
            .expect("failed to write out html content..");
    }
}

// import に指定されたファイルパスからプロジェクトルートへの相対パスを得る
// e.g., "foo/bar/baz.bok" から "../../" を得る
fn calc_relative_to_root(filepath: &Path) -> PathBuf {
    let depth = filepath.components().collect::<Vec<_>>().len() - 1;
    if depth == 0 {
        return PathBuf::from("./");
    }
    let mut result = PathBuf::new();
    for _ in 0..depth {
        result.push("..");
    }
    result
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
            let content = verbatim_block_content(src, &lines);
            katex::render(content, true)
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
        BlockElement::Derivation(derivation) => print_derivation(src, derivation),
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

fn print_derivation(src: &Vec<Vec<char>>, derivation: Derivation) -> String {
    let (katex_src, inner_elements) = print_derivation_impl(src, derivation, vec![]);
    let content = katex::render(katex_src, true);
    inner_elements
        .into_iter()
        .enumerate()
        .fold(content, |acc, (count, inner)| {
            let mark = format!("bokuteki inner element {}", count);
            acc.as_str().replacen(&mark, &inner, 2)
        })
}

fn print_derivation_impl(
    src: &Vec<Vec<char>>,
    derivation: Derivation,
    mut inner_elements: Vec<String>,
) -> (String, Vec<String>) {
    fn make_fresh_mark(count: usize) -> String {
        format!("\\text{{bokuteki inner element {}}}", count)
    }

    match derivation {
        Derivation::InferenceRule {
            premises,
            conclusion,
            rule_name,
        } => {
            let mut premise_katex_srcs = vec![];
            for premise in premises {
                let (premise_katex_src, inner_elements_) =
                    print_derivation_impl(src, premise, inner_elements);
                premise_katex_srcs.push(premise_katex_src);
                inner_elements = inner_elements_;
            }
            let premises_katex_src = premise_katex_srcs
                .into_iter()
                .collect::<Vec<_>>()
                .join("\\ \\ ");

            let conclusion_mark = make_fresh_mark(inner_elements.len());
            let conclusion_katex_src = conclusion_mark;
            inner_elements.push(print_inline_elements(src, conclusion, 0));

            if rule_name.is_empty() {
                let katex_src =
                    format!("\\dfrac{{{premises_katex_src}}}{{{conclusion_katex_src}}}");

                (katex_src, inner_elements)
            } else {
                let rule_name_mark = make_fresh_mark(inner_elements.len());
                let rule_name_katex_src = rule_name_mark;
                inner_elements.push(print_inline_elements(src, rule_name, 0));
                let katex_src =
                    format!("\\dfrac{{{premises_katex_src}}}{{{conclusion_katex_src}}} {rule_name_katex_src}");
                (katex_src, inner_elements)
            }
        }
        Derivation::Leaf(inline_elements) => {
            let inner_element = print_inline_elements(src, inline_elements, 0);
            let katex_src = make_fresh_mark(inner_elements.len());
            inner_elements.push(inner_element);
            (katex_src, inner_elements)
        }
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
        InlineElement::Link {
            text,
            mut url_range,
        } => {
            let text = text
                .into_iter()
                .map(|inline_element| print_inline_element(src, inline_element))
                .collect::<Vec<_>>()
                .join("");
            let mut url = String::new();
            while !url_range.is_empty() {
                let c = pick_char(src, &url_range).unwrap();
                url.push(c);
                url_range.move_to_next_char();
            }
            format!("<a href=\"{}\">{}</a>", url, text)
        }
        InlineElement::Math { mut range } => {
            let mut result = String::new();
            while !range.is_empty() {
                let c: char = pick_char(src, &range).unwrap();
                result.push(c);
                range.move_to_next_char();
            }
            katex::render(result, false)
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
