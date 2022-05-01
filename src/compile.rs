use crate::{lsp::state::Cache, parse, print, source::Source};

use std::path::PathBuf;

pub fn compile(input_file: PathBuf) {
    let src = Source::open(input_file.as_path());
    compile_src(src);
}

pub fn compile_src(src: Source) -> (Vec<parse::Error>, Vec<parse::Warning>) {
    let mut dummy_cache = Cache::new();
    let result = parse::parse_document(&src, &mut dummy_cache);
    if !result.errors.is_empty() || !result.warnings.is_empty() {
        (result.errors, result.warnings)
    } else {
        print::print(&src, &dummy_cache, result.value);
        (vec![], vec![])
    }
}
