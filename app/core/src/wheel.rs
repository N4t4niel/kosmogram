//! Chart wheel as an SVG string. Glyphs use the Astronomicon font loaded by the UI.
use crate::chart::{min_diff, norm360, Body, Chart};
use crate::data::*;
use std::fmt::Write;

const C: f64 = 400.0;
const OUTER: f64 = 394.0;
const SIGN_IN: f64 = 350.0;
const ELEMENT_FILL: [&str; 4] = ["var(--w-fire)", "var(--w-earth)", "var(--w-air)", "var(--w-water)"];
const INK: &str = "var(--w-ink)";
const SECOND: &str = "var(--w-second)";
const MIN_GAP: f64 = 9.0;

pub struct Line {
    pub from: f64,
    pub to: f64,
    pub color: &'static str,
    pub weight: f64,
}

fn at(lon: f64, r: f64, left: f64) -> (f64, f64) {
    let th = (180.0 + lon - left).to_radians();
    (C + r * th.cos(), C - r * th.sin())
}

fn esc(c: &str) -> String {
    c.replace('&', "&amp;").replace('<', "&lt;").replace('"', "&quot;")
}

fn radial(o: &mut String, lon: f64, r1: f64, r2: f64, left: f64, stroke: &str, w: f64) {
    let (a, b) = (at(lon, r1, left), at(lon, r2, left));
    let _ = write!(o, r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{stroke}" stroke-width="{w}"/>"#, a.0, a.1, b.0, b.1);
}

fn glyph(o: &mut String, p: (f64, f64), g: &str, size: f64, fill: &str, class: &str) {
    let _ = write!(o, r#"<text x="{:.1}" y="{:.1}" class="{class}" font-size="{size}" fill="{fill}">{}</text>"#, p.0, p.1, esc(g));
}

fn label(o: &mut String, p: (f64, f64), t: &str, size: f64, fill: &str) {
    let _ = write!(o, r#"<text x="{:.1}" y="{:.1}" class="t" font-size="{size}" fill="{fill}">{}</text>"#, p.0, p.1, esc(t));
}

fn circle(o: &mut String, r: f64, fill: &str) {
    let _ = write!(o, r##"<circle cx="400" cy="400" r="{r}" fill="{fill}" stroke="var(--w-line)" stroke-width="1"/>"##);
}

/// Spreads glyphs apart while keeping their order around the circle.
fn spread(bodies: &[&Body]) -> Vec<(usize, f64)> {
    let mut v: Vec<(usize, f64)> = bodies.iter().enumerate().map(|(i, b)| (i, b.lon)).collect();
    v.sort_by(|a, b| a.1.total_cmp(&b.1));
    let n = v.len();
    for _ in 0..200 {
        let mut moved = false;
        for i in 0..n {
            let j = (i + 1) % n;
            let gap = min_diff(v[j].1, v[i].1);
            if n > 1 && gap < MIN_GAP {
                let push = (MIN_GAP - gap) / 2.0 + 0.01;
                v[i].1 = norm360(v[i].1 - push);
                v[j].1 = norm360(v[j].1 + push);
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    v
}

fn planets(o: &mut String, c: &Chart, r_glyph: f64, r_deg: f64, r_tick: f64, left: f64, color: &str) {
    let bodies: Vec<&Body> = c.bodies.iter().filter(|b| !matches!(OBJECTS[b.index].kind, ObjKind::Asc | ObjKind::Mc)).collect();
    for (i, shown) in spread(&bodies) {
        let b = bodies[i];
        radial(o, b.lon, r_tick, r_tick - 8.0, left, color, 1.5);
        let (a, z) = (at(b.lon, r_tick - 8.0, left), at(shown, r_glyph + 15.0, left));
        let _ = write!(o, r##"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="var(--w-leader)" stroke-width="0.7"/>"##, a.0, a.1, z.0, z.1);
        glyph(o, at(shown, r_glyph, left), &b.glyph, 38.0, color, "g h");
        let deg = (b.lon % 30.0).floor() as i32;
        let text = if b.retro { format!("{deg}°R") } else { format!("{deg}°") };
        label(o, at(shown, r_deg, left), &text, 14.0, color);
    }
}

pub fn render(inner: &Chart, outer: Option<&Chart>, lines: &[Line]) -> String {
    let angle = |kind: ObjKind| inner.bodies.iter().find(|b| OBJECTS[b.index].kind == kind).map(|b| b.lon);
    let left = angle(ObjKind::Asc).unwrap_or(0.0);
    let (r_inner_glyph, r_inner_deg, r_disc, r_house, r_inner_tick) = if outer.is_some() {
        (254.0, 221.0, 176.0, 193.0, 288.0)
    } else {
        (300.0, 258.0, 205.0, 223.0, SIGN_IN)
    };
    let mut o = String::with_capacity(16000);
    o.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 800" role="img" aria-label="Horoskop"><style>.g{font-family:Astronomicon;text-anchor:middle;dominant-baseline:central}.t{font-family:Segoe UI,sans-serif;text-anchor:middle;dominant-baseline:central}.t,.h{paint-order:stroke;stroke:var(--w-bg);stroke-width:4px}</style>"#);
    circle(&mut o, OUTER, "var(--w-bg)");
    for s in 0..12 {
        let start = s as f64 * 30.0;
        let (a, b) = (at(start, OUTER, left), at(start + 30.0, OUTER, left));
        let (c1, d) = (at(start + 30.0, SIGN_IN, left), at(start, SIGN_IN, left));
        let _ = write!(o, r##"<path d="M{:.1} {:.1}A{OUTER} {OUTER} 0 0 0 {:.1} {:.1}L{:.1} {:.1}A{SIGN_IN} {SIGN_IN} 0 0 1 {:.1} {:.1}Z" fill="{}" stroke="var(--w-line)" stroke-width="1"/>"##, a.0, a.1, b.0, b.1, c1.0, c1.1, d.0, d.1, ELEMENT_FILL[s % 4]);
        glyph(&mut o, at(start + 15.0, (OUTER + SIGN_IN) / 2.0, left), &SIGN_GLYPHS[s].to_string(), 38.0, INK, "g");
        for d in (0..30).step_by(5) {
            let len = if d % 10 == 0 { 8.0 } else { 4.0 };
            radial(&mut o, start + d as f64, SIGN_IN, SIGN_IN - len, left, "var(--w-line)", 0.8);
        }
    }
    if let Some(c) = outer {
        planets(&mut o, c, 322.0, 297.0, SIGN_IN, left, SECOND);
        let _ = write!(o, r##"<circle cx="400" cy="400" r="{r_inner_tick}" fill="none" stroke="var(--w-line)" stroke-width="1"/>"##);
    }
    if let Some(cusps) = &inner.cusps {
        for (i, &c) in cusps.iter().enumerate() {
            radial(&mut o, c, r_disc, r_inner_tick, left, "var(--w-cusp)", 0.8);
            let mid = c + min_diff(cusps[(i + 1) % 12], c).rem_euclid(360.0) / 2.0;
            label(&mut o, at(mid, r_house, left), &(i + 1).to_string(), 15.0, "var(--w-house)");
        }
        // The axes stand on the real angles: in equal and whole sign houses they miss the cusps.
        for (text, kind) in [("ASC", ObjKind::Asc), ("MC", ObjKind::Mc)] {
            let Some(lon) = angle(kind) else { continue };
            for end in [lon, lon + 180.0] {
                radial(&mut o, end, r_disc, r_inner_tick, left, INK, 1.8);
            }
            // Between the sign ring and the planet glyphs, where nothing else is drawn.
            label(&mut o, at(lon + 3.0, r_inner_tick - 12.0, left), text, 14.0, INK);
        }
    }
    planets(&mut o, inner, r_inner_glyph, r_inner_deg, r_inner_tick, left, INK);
    circle(&mut o, r_disc, "var(--w-disc)");
    for l in lines {
        let (a, b) = (at(l.from, r_disc, left), at(l.to, r_disc, left));
        let _ = write!(o, r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{:.2}" stroke-opacity="0.8"/>"#, a.0, a.1, b.0, b.1, l.color, l.weight);
    }
    o.push_str("</svg>");
    o
}
