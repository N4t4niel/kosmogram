use crate::chart::{min_diff, Body, Chart};
use crate::data::*;
use crate::model::Settings;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Aspect {
    pub a: usize,
    pub b: usize,
    pub a_name: &'static str,
    pub b_name: &'static str,
    pub a_glyph: String,
    pub b_glyph: String,
    pub aspect: &'static str,
    pub aspect_name: &'static str,
    pub aspect_index: usize,
    pub glyph: String,
    pub color: &'static str,
    /// Absolute distance from the exact aspect in degrees.
    pub orb: f64,
    pub max_orb: f64,
    pub applying: bool,
    pub power: f64,
}

pub fn aspect_count(s: &Settings) -> usize {
    if s.minor_aspects { ASPECTS.len() } else { MAJOR_ASPECTS }
}

/// Astrolog GetOrb.
pub fn get_orb(a: usize, b: usize, asp: usize) -> f64 {
    let (x, y) = (&OBJECTS[a], &OBJECTS[b]);
    ASPECTS[asp].orb.min(x.orb_cap).min(y.orb_cap) + x.orb_add + y.orb_add
}

/// `x` is the moving body; with `both_move` false, `y` is treated as a fixed natal point.
fn find(x: &Body, y: &Body, n_asp: usize, both_move: bool) -> Option<Aspect> {
    let sep = min_diff(x.lon, y.lon).abs();
    for k in 0..n_asp {
        let d = &ASPECTS[k];
        let diff = sep - d.angle;
        let orb = get_orb(x.index, y.index, k);
        if diff.abs() < orb {
            let vy = if both_move { y.speed } else { 0.0 };
            let signed = diff * (x.speed - vy).signum() * min_diff(x.lon, y.lon).signum();
            let power = d.influence * (OBJECTS[x.index].influence + OBJECTS[y.index].influence) / 2.0 * (1.0 - diff.abs() / orb);
            return Some(Aspect {
                a: x.index,
                b: y.index,
                a_name: x.name,
                b_name: y.name,
                a_glyph: x.glyph.clone(),
                b_glyph: y.glyph.clone(),
                aspect: d.id,
                aspect_name: d.name,
                aspect_index: k,
                glyph: d.glyph.to_string(),
                color: d.color,
                orb: diff.abs(),
                max_orb: orb,
                applying: signed < 0.0,
                power,
            });
        }
    }
    None
}

pub fn within(c: &Chart, s: &Settings) -> Vec<Aspect> {
    let n = aspect_count(s);
    let mut out = Vec::new();
    for i in 0..c.bodies.len() {
        for j in i + 1..c.bodies.len() {
            if let Some(a) = find(&c.bodies[i], &c.bodies[j], n, true) {
                out.push(a);
            }
        }
    }
    out.sort_by(|a, b| b.power.total_cmp(&a.power));
    out
}

pub fn between(a: &Chart, b: &Chart, s: &Settings, both_move: bool) -> Vec<Aspect> {
    let n = aspect_count(s);
    let mut out = Vec::new();
    for x in &a.bodies {
        for y in &b.bodies {
            if let Some(v) = find(x, y, n, both_move) {
                out.push(v);
            }
        }
    }
    out.sort_by(|p, q| q.power.total_cmp(&p.power));
    out
}
