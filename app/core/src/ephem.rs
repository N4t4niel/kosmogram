//! Geocentric apparent positions from JPL's DE440 and Horizons series packed into `data/ephem.bin`.
//!
//! The file is read in place: nothing is unpacked, nothing is written to disk and there is no shared
//! mutable state, so any number of threads can calculate at once.
use crate::earth::{cross, delta_t, dot, unit, Frame, Vec3};
use std::sync::OnceLock;

/// Our own numbering of the bodies.
pub type Id = i32;
pub const NODE: Id = 10;
pub const CHIRON: Id = 15;

static RAW: &[u8] = include_bytes!("../data/ephem.bin");


const C_KM_DAY: f64 = 299_792.458 * 86400.0;
/// Earth to Moon mass ratio of DE440.
const EMRAT: f64 = 81.3005682214972154;
/// Schwarzschild radius of the Sun in km.
const SUN_RS_KM: f64 = 2.95325008;
const OUT_OF_RANGE: &str = "Výpočty fungují pro roky 1800 až 2399.";
/// Most terms any series keeps.
const MAX_TERMS: usize = 14;

const SUN: usize = 9;
const MOON: usize = 10;
const EMB: usize = 2;
const SERIES_CHIRON: usize = 11;

struct Series {
    start: f64,
    days: f64,
    segments: usize,
    terms: usize,
    step: f64,
    data: &'static [u8],
    widths: &'static [u8],
    segment_bits: usize,
}

fn u32_at(at: usize) -> u32 {
    u32::from_le_bytes(RAW[at..at + 4].try_into().unwrap())
}

fn f64_at(at: usize) -> f64 {
    f64::from_le_bytes(RAW[at..at + 8].try_into().unwrap())
}

fn all_series() -> &'static [Series] {
    static SERIES: OnceLock<Vec<Series>> = OnceLock::new();
    SERIES.get_or_init(|| {
        let count = u32_at(0) as usize;
        let mut at = 4;
        (0..count)
            .map(|_| {
                let (segments, terms, offset) = (u32_at(at + 4) as usize, u32_at(at + 8) as usize, u32_at(at + 12) as usize);
                let widths = &RAW[at + 40..at + 40 + 3 * terms];
                let series = Series {
                    start: f64_at(at + 16),
                    days: f64_at(at + 24),
                    segments,
                    terms,
                    step: f64_at(at + 32),
                    data: &RAW[offset..],
                    widths,
                    segment_bits: widths.iter().map(|&w| w as usize).sum(),
                };
                at += 40 + 3 * terms;
                series
            })
            .collect()
    })
}

/// A value of `width` bits at bit position `at`, sign extended; the widths never exceed 40 bits.
#[inline]
fn signed_bits(data: &[u8], at: usize, width: u32) -> i64 {
    if width == 0 {
        return 0;
    }
    let mut bytes = [0u8; 8];
    let first = at / 8;
    let end = (first + 8).min(data.len());
    bytes[..end - first].copy_from_slice(&data[first..end]);
    let raw = u64::from_le_bytes(bytes) >> (at % 8);
    ((raw << (64 - width)) as i64) >> (64 - width)
}

impl Series {
    /// Position in km and, on request, velocity in km per day.
    fn eval(&self, jd: f64, want_velocity: bool) -> Result<(Vec3, Vec3), String> {
        let position = (jd - self.start) / self.days;
        if !(0.0..self.segments as f64).contains(&position) {
            return Err(OUT_OF_RANGE.into());
        }
        let segment = position as usize;
        let x = 2.0 * (position - segment as f64) - 1.0;
        let mut t = [0.0; MAX_TERMS];
        let mut dt = [0.0; MAX_TERMS];
        t[0] = 1.0;
        t[1] = x;
        dt[1] = 1.0;
        for k in 2..self.terms {
            t[k] = 2.0 * x * t[k - 1] - t[k - 2];
            dt[k] = 2.0 * t[k - 1] + 2.0 * x * dt[k - 1] - dt[k - 2];
        }
        let mut bit = segment * self.segment_bits;
        let (mut pos, mut vel) = ([0.0; 3], [0.0; 3]);
        for axis in 0..3 {
            for k in 0..self.terms {
                let width = self.widths[axis * self.terms + k] as u32;
                let c = signed_bits(self.data, bit, width) as f64;
                bit += width as usize;
                pos[axis] += c * t[k];
                if want_velocity {
                    vel[axis] += c * dt[k];
                }
            }
            pos[axis] *= self.step;
            vel[axis] *= self.step * 2.0 / self.days;
        }
        Ok((pos, vel))
    }
}

fn series(index: usize, jd: f64) -> Result<Vec3, String> {
    Ok(all_series()[index].eval(jd, false)?.0)
}

/// Barycentric position and velocity of the Earth, which sits opposite the Moon around their centre.
fn earth(jd: f64) -> Result<(Vec3, Vec3), String> {
    let (emb, emb_v) = all_series()[EMB].eval(jd, true)?;
    let (moon, moon_v) = all_series()[MOON].eval(jd, true)?;
    let mut p = [0.0; 3];
    let mut v = [0.0; 3];
    for i in 0..3 {
        p[i] = emb[i] - moon[i] / EMRAT;
        v[i] = emb_v[i] - moon_v[i] / EMRAT;
    }
    Ok((p, v))
}

/// Barycentric position of a body. Planets are the centres of mass of their systems, as in JPL's files.
fn barycentric(body: Id, jd: f64) -> Result<Vec3, String> {
    match body {
        0 => series(SUN, jd),
        1 => {
            let (emb, moon) = (series(EMB, jd)?, series(MOON, jd)?);
            Ok([emb[0] + moon[0], emb[1] + moon[1], emb[2] + moon[2]])
        }
        2 => series(0, jd),
        3 => series(1, jd),
        4..=9 => series(body as usize - 1, jd),
        CHIRON => series(SERIES_CHIRON, jd),
        _ => Err(format!("Neznámé těleso {body}")),
    }
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Ecliptic longitude and latitude of date in degrees, as seen from the Earth at `jd` (TT).
fn apparent(body: Id, jd: f64) -> Result<(f64, f64), String> {
    let (earth_p, earth_v) = earth(jd)?;
    let sun_now = series(SUN, jd)?;

    let mut tau = 0.0;
    let mut target = barycentric(body, jd)?;
    for _ in 0..3 {
        let d = sub(target, earth_p);
        tau = dot(d, d).sqrt() / C_KM_DAY;
        target = barycentric(body, jd - tau)?;
    }
    let mut p = unit(sub(target, earth_p));

    // The Sun bends the light of everything but itself; the Moon is too near for it to matter.
    if body != 0 && body != 1 {
        let sun = series(SUN, jd - tau)?;
        let q = unit(sub(target, sun));
        let from_sun = sub(earth_p, sun_now);
        let em = dot(from_sun, from_sun).sqrt();
        let e = unit(from_sun);
        let sum = [q[0] + e[0], q[1] + e[1], q[2] + e[2]];
        // Behind the Sun's disc the bending fades out instead of growing without limit.
        const DISC_EDGE: f64 = 2.2e-5;
        let w = SUN_RS_KM / em / dot(q, sum).max(DISC_EDGE);
        let bend = cross(p, cross(e, q));
        for i in 0..3 {
            p[i] += w * bend[i];
        }
        p = unit(p);
    }

    // Annual aberration.
    let beta = [earth_v[0] / C_KM_DAY, earth_v[1] / C_KM_DAY, earth_v[2] / C_KM_DAY];
    let gamma_inv = (1.0 - dot(beta, beta)).sqrt();
    let pv = dot(p, beta);
    let w = 1.0 + pv / (1.0 + gamma_inv);
    let moved = [p[0] * gamma_inv + w * beta[0], p[1] * gamma_inv + w * beta[1], p[2] * gamma_inv + w * beta[2]];

    let (lon, lat) = Frame::at(jd).to_ecliptic(moved);
    Ok((lon.to_degrees().rem_euclid(360.0), lat.to_degrees()))
}

/// Mean ascending node of the Moon, Chapront-Touzé & Chapront 1991 as given by Meeus, of date.
fn mean_node(jd: f64) -> f64 {
    let t = (jd - crate::earth::J2000) / 36525.0;
    let mean = 125.0445479 + t * (-1934.1362891 + t * (0.0020754 + t * (1.0 / 467441.0 - t / 60616000.0)));
    (mean + Frame::at(jd).dpsi.to_degrees()).rem_euclid(360.0)
}

fn longitude_tt(body: Id, jd: f64) -> Result<f64, String> {
    if body == NODE { Ok(mean_node(jd)) } else { Ok(apparent(body, jd)?.0) }
}

/// Apparent place in degrees, speed in degrees per day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub lon: f64,
    pub lat: f64,
    pub speed: f64,
}

/// Position for a moment given in terrestrial time.
pub fn position_tt(jd: f64, body: Id) -> Result<Position, String> {
    const H: f64 = 0.01;
    let (lon, lat) = if body == NODE { (mean_node(jd), 0.0) } else { apparent(body, jd)? };
    let before = longitude_tt(body, jd - H)?;
    let after = longitude_tt(body, jd + H)?;
    let d = (after - before + 540.0).rem_euclid(360.0) - 180.0;
    Ok(Position { lon, lat, speed: d / (2.0 * H) })
}

/// Position for a moment given in universal time.
pub fn position(jd_ut: f64, body: Id) -> Result<Position, String> {
    position_tt(jd_ut + delta_t(jd_ut), body)
}

/// Longitude alone costs a third of a full position; searches call this thousands of times.
pub fn longitude(jd_ut: f64, body: Id) -> Result<f64, String> {
    longitude_tt(body, jd_ut + delta_t(jd_ut))
}

/// Julian day of a Gregorian calendar date, `hour` in decimal hours.
pub fn julday(year: i32, month: u32, day: u32, hour: f64) -> f64 {
    let (y, m) = if month > 2 { (year as i64, month as i64) } else { (year as i64 - 1, month as i64 + 12) };
    let a = y.div_euclid(100);
    let b = 2 - a + a.div_euclid(4);
    (365.25 * (y + 4716) as f64).floor() + (30.6001 * (m + 1) as f64).floor() + day as f64 + b as f64 - 1524.5 + hour / 24.0
}


