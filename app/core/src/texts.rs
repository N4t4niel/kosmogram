//! Hand-written interpretations from `data/vyklady`, built into the program.
//! A line is `part | part | text`, see `data/vyklady/PRAVIDLA.md` for the keys.
//! A missing key means the caller falls back to the sentences composed in `interpret`.
use std::collections::HashMap;
use std::sync::OnceLock;

const ALL: &str = include_str!(concat!(env!("OUT_DIR"), "/vyklady.txt"));

fn table() -> &'static HashMap<&'static str, &'static str> {
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| ALL.lines().filter_map(split).collect())
}

fn split(line: &'static str) -> Option<(&'static str, &'static str)> {
    let line = line.trim().trim_start_matches('\u{feff}');
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    line.rsplit_once(" | ")
}

pub fn get(key: &str) -> Option<&'static str> {
    table().get(key).copied()
}

pub fn sign(body: &str, sign: &str) -> Option<&'static str> {
    get(&format!("{body} | {sign}"))
}

pub fn house(body: &str, house: usize) -> Option<&'static str> {
    get(&format!("{body} | {house}. dům"))
}

pub fn aspect(a: &str, b: &str, aspect: &str) -> Option<&'static str> {
    get(&format!("{a} | {b} | {aspect}"))
}

/// Every key in the data files, for checking them.
pub fn keys() -> Vec<&'static str> {
    table().keys().copied().collect()
}
