use crate::{
    document::Document,
    parse,
    source::{range::InlineRange, Source, UpdateInfo},
};
use lsp_types::Url;
use std::collections::HashMap;

pub struct State {
    pub units: HashMap<Url, StateUnit>,
}

pub type Cache = HashMap<InlineRange, String>;

pub struct StateUnit {
    src: Source,
    document: Document,
    inline_elements_cache: Cache,
}

impl Default for State {
    fn default() -> Self {
        Self {
            units: vec![].into_iter().collect(),
        }
    }
}

impl State {
    pub fn add_source(&mut self, url: Url, text: String) -> (Vec<String>, Vec<String>) {
        let src = Source::from_text(text);
        let mut cache = HashMap::new();
        let document_result = parse::parse_document(&src, &mut cache);
        self.units.insert(
            url,
            StateUnit {
                src,
                document: document_result.value,
                inline_elements_cache: cache,
            },
        );
        (document_result.errors, document_result.warnings)
    }

    pub fn update(&mut self, url: Url, infos: Vec<UpdateInfo>) -> (Vec<String>, Vec<String>) {
        let mut unit = self.units.get_mut(&url).expect("unknown url");
        unit.src.update(infos.clone()); // TODO
        unit.inline_elements_cache.retain(|ref range, _| {
            infos
                .iter()
                .any(|info| is_exclusive(&unit.src, &range, &info))
        });
        unit.document
            .update(&unit.src, &mut unit.inline_elements_cache)
    }
}

type Point = (u32, u32);

fn is_exclusive(src: &Source, range: &InlineRange, info: &UpdateInfo) -> bool {
    assert!(is_former(info.start, info.end));

    let (update_start, update_end) = range.calc_range_on_source(src);
    assert!(is_former(update_start, update_end));

    !is_former(update_start, info.end) || is_former(info.start, update_end)
}

fn is_former(a: Point, b: Point) -> bool {
    a.0 < b.0 || (a.0 == b.0 && a.1 <= b.1)
}
