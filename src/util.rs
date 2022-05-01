use crate::source::{range::InlineRange, Source};
use log::debug;

#[allow(dead_code)]
pub fn debug_at(src: &Source, range: &InlineRange, msg: &str) {
    debug!("{} | {}", msg, range.to_string(src));
}
