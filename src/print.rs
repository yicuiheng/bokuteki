use crate::document::*;

pub fn print(src: &Vec<Vec<char>>, doc: Document) {
    let body_string: String = doc
        .block_elements
        .into_iter()
        .map(|block_element| print_block_element(src, block_element))
        .collect::<Vec<_>>()
        .join("\n");
    println!(
        r#"
<!DOCTYPE html>
<html>

<head>
  <meta charset="utf-8">
</head>

<body>
{}
</body>

</html>
"#,
        body_string
    );
}

fn print_block_element(src: &Vec<Vec<char>>, block_element: BlockElement) -> String {
    match block_element {
        BlockElement::Heading { level, content } => {
            format!(
                "<h{}>{}</h{}>",
                level,
                content
                    .into_iter()
                    .map(|span_element| print_span_element(src, span_element))
                    .collect::<Vec<_>>()
                    .join(""),
                level
            )
        }
        BlockElement::Paragraph { content } => {
            format!(
                "<p>{}</p>",
                content
                    .into_iter()
                    .map(|span_element| print_span_element(src, span_element))
                    .collect::<Vec<_>>()
                    .join("")
            )
        }
    }
}

fn print_span_element(src: &Vec<Vec<char>>, span_element: SpanElement) -> String {
    match span_element {
        SpanElement::Text { mut range } => {
            let mut result = String::new();
            while range.start != range.end {
                let c: char = pick_char(src, &range.start).unwrap();
                result.push(c);
                range.start.move_to_next_char();
            }
            result
        }
        _ => unimplemented!(),
    }
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
