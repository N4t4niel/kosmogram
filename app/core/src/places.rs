use crate::model::Place;
use flate2::read::GzDecoder;
use serde::Serialize;
use std::io::Read;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

static RAW: &[u8] = include_bytes!("../data/places.tsv.gz");

struct Entry {
    name: String,
    region: String,
    country: String,
    lat: f64,
    lon: f64,
    tz: String,
    pop: u64,
    key: String,
    alias: String,
    region_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Found {
    #[serde(flatten)]
    pub place: Place,
    pub population: u64,
}

/// Lowercase, no diacritics: "České Budějovice" -> "ceske budejovice".
pub fn normalize(s: &str) -> String {
    s.nfd()
        .filter(|c| !('\u{300}'..='\u{36f}').contains(c))
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'ł' => 'l',
            'ø' => 'o',
            'đ' => 'd',
            'ß' => 's',
            '-' | '–' => ' ',
            c => c,
        })
        .collect()
}

fn entries() -> &'static Vec<Entry> {
    static E: OnceLock<Vec<Entry>> = OnceLock::new();
    E.get_or_init(|| {
        let mut text = String::new();
        GzDecoder::new(RAW).read_to_string(&mut text).expect("data míst");
        text.lines()
            .filter_map(|l| {
                let p: Vec<&str> = l.split('\t').collect();
                if p.len() < 8 {
                    return None;
                }
                Some(Entry {
                    key: normalize(p[0]),
                    alias: normalize(p[1]),
                    region_key: normalize(p[3]),
                    name: p[0].into(),
                    region: p[3].into(),
                    country: p[2].into(),
                    lat: p[4].parse().ok()?,
                    lon: p[5].parse().ok()?,
                    tz: p[6].into(),
                    pop: p[7].parse().unwrap_or(0),
                })
            })
            .collect()
    })
}

fn weight(e: &Entry) -> f64 {
    let bonus = match e.country.as_str() {
        "Česko" => 60.0,
        "Slovensko" => 25.0,
        _ => 1.0,
    };
    (e.pop as f64 + 50.0) * bonus
}

/// Lower class = better match.
fn class(e: &Entry, q: &str, tokens: &[&str]) -> Option<u8> {
    if e.key == q || e.alias.split('|').any(|a| a == q) {
        return Some(0);
    }
    if e.key.starts_with(q) {
        return Some(1);
    }
    if e.alias.split('|').any(|a| a.starts_with(q)) {
        return Some(2);
    }
    if e.key.split(' ').any(|w| w.starts_with(q)) {
        return Some(3);
    }
    if tokens.len() > 1 && tokens.iter().all(|t| e.key.contains(t) || e.region_key.contains(t)) {
        return Some(4);
    }
    if q.len() >= 4 && e.key.contains(q) {
        return Some(5);
    }
    None
}

pub fn search(query: &str, limit: usize) -> Vec<Found> {
    let q = normalize(query.trim());
    if q.chars().count() < 2 {
        return vec![];
    }
    let tokens: Vec<&str> = q.split_whitespace().collect();
    let mut hits: Vec<(u8, f64, &Entry)> = entries().iter().filter_map(|e| class(e, &q, &tokens).map(|c| (c, -weight(e), e))).collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    hits.into_iter()
        .take(limit)
        .map(|(_, _, e)| Found {
            place: Place { name: e.name.clone(), region: e.region.clone(), country: e.country.clone(), lat: e.lat, lon: e.lon, tz: e.tz.clone() },
            population: e.pop,
        })
        .collect()
}

/// Forces the data to load, so the first keystroke is not the slow one.
pub fn preload() {
    let _ = entries();
}
