use crate::{document, parse, print::Printer};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

pub fn build(src_path: PathBuf) {
    let printer = Printer::setup();

    let mut q = VecDeque::new();
    let root_path = src_path.parent().unwrap().to_path_buf();
    q.push_back(PathBuf::from(src_path.file_name().unwrap()));
    while let Some(import_path) = q.pop_front() {
        if let Ok(content) = fs::read_to_string(root_path.join(&import_path)) {
            let imports = compile(&printer, &import_path, content);
            let import_base_path = import_path.parent().unwrap();
            q.append(
                &mut imports
                    .into_iter()
                    .map(|child| {
                        let mut path = import_base_path.join(child);
                        path.set_extension("bok");
                        path
                    })
                    .collect(),
            );
        } else {
            eprintln!("[error] input file not found: {}", import_path.display());
        }
    }
}

fn compile(printer: &Printer, src_path: &Path, content: String) -> Vec<PathBuf> {
    let content: Vec<Vec<char>> = content.lines().map(|line| line.chars().collect()).collect();
    let src_block_range = document::src_block_range(&content);

    let result = parse::parse_document(&content, src_block_range);
    if result.errors.is_empty() {
        printer.print(&content, src_path, result.value.block_elements);
    }

    for error in &result.errors {
        eprintln!("[error] {}: {}", src_path.display(), error);
    }
    for warning in &result.warnings {
        eprintln!("[warning] {}: {}", src_path.display(), warning);
    }

    result.value.imports
}
