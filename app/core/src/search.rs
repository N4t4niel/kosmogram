//! Finds the exact moments of transits and progressions to natal points.
use crate::chart::{min_diff, norm360, progressed_jd, sign_of, solar_arc, Chart};
use crate::data::*;
use crate::model::Settings;
use crate::{ephem, houses};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Hit {
    pub jd: f64,
    pub mover: usize,
    pub mover_name: &'static str,
    pub mover_glyph: String,
    pub aspect: usize,
    pub aspect_name: &'static str,
    pub aspect_glyph: String,
    pub color: &'static str,
    pub natal: usize,
    pub natal_name: &'static str,
    pub natal_glyph: String,
    pub retro: bool,
    /// Sign of the moving body at the moment of the hit.
    pub sign: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Transit,
    Progression,
}

struct Ctx<'a> {
    mode: Mode,
    natal_jd: f64,
    lat: f64,
    lon: f64,
    s: &'a Settings,
}

impl Ctx<'_> {
    fn at(&self, jd: f64) -> f64 {
        if self.mode == Mode::Progression { progressed_jd(self.natal_jd, jd) } else { jd }
    }

    fn lon(&self, obj: usize, jd: f64) -> Result<f64, String> {
        let t = self.at(jd);
        match OBJECTS[obj].kind {
            ObjKind::Body(body) => ephem::longitude(t, body),
            ObjKind::Asc => Ok(self.angles(t)?.0),
            ObjKind::Mc => Ok(self.angles(t)?.1),
        }
    }

    fn angles(&self, t: f64) -> Result<(f64, f64), String> {
        let system = self.s.house_system as u8;
        if self.mode == Mode::Progression && self.s.solar_arc_angles {
            return Ok(houses::angles_by_arc(self.natal_jd, t, solar_arc(self.natal_jd, t)?, self.lat, self.lon, system));
        }
        Ok(houses::angles(t, self.lat, self.lon, system))
    }

    fn retrograde(&self, obj: usize, jd: f64) -> Result<bool, String> {
        match OBJECTS[obj].kind {
            ObjKind::Body(body) => Ok(ephem::position(self.at(jd), body)?.speed < 0.0),
            _ => Ok(false),
        }
    }
}

/// Bodies that move. Astrolog's defaults skip the transiting Moon (too fast to matter), while
/// progressions add the Moon and angles, because the progressed Moon is the main timing tool.
pub fn movers(mode: Mode, s: &Settings, has_angles: bool) -> Vec<usize> {
    (0..OBJECTS.len())
        .filter(|&i| match OBJECTS[i].kind {
            ObjKind::Asc | ObjKind::Mc => mode == Mode::Progression && has_angles,
            _ if i == MOON => mode == Mode::Progression,
            _ if i == CHIRON => s.chiron,
            _ => true,
        })
        .collect()
}

/// Each moving body is searched on its own thread; the order of the result never depends on timing.
pub fn find_hits(natal: &Chart, place: (f64, f64), from_jd: f64, to_jd: f64, mode: Mode, s: &Settings) -> Result<Vec<Hit>, String> {
    let ctx = Ctx { mode, natal_jd: natal.jd_ut, lat: place.0, lon: place.1, s };
    let found: Vec<_> = std::thread::scope(|scope| {
        let workers: Vec<_> = movers(mode, s, natal.cusps.is_some())
            .into_iter()
            .map(|mover| {
                let ctx = &ctx;
                scope.spawn(move || scan(ctx, natal, mover, from_jd, to_jd))
            })
            .collect();
        workers.into_iter().map(|w| w.join().expect("vlákno výpočtu spadlo")).collect()
    });
    merge(found)
}

/// The same search on the calling thread, kept as the reference that tests compare `find_hits` with.
pub fn find_hits_serial(natal: &Chart, place: (f64, f64), from_jd: f64, to_jd: f64, mode: Mode, s: &Settings) -> Result<Vec<Hit>, String> {
    let ctx = Ctx { mode, natal_jd: natal.jd_ut, lat: place.0, lon: place.1, s };
    merge(movers(mode, s, natal.cusps.is_some()).into_iter().map(|m| scan(&ctx, natal, m, from_jd, to_jd)).collect())
}

fn merge(found: Vec<Result<Vec<Hit>, String>>) -> Result<Vec<Hit>, String> {
    let mut hits = Vec::new();
    for part in found {
        hits.extend(part?);
    }
    hits.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    Ok(hits)
}

fn scan(ctx: &Ctx, natal: &Chart, mover: usize, from_jd: f64, to_jd: f64) -> Result<Vec<Hit>, String> {
    let step = if ctx.mode == Mode::Progression { 2.0 } else { 0.25 };
    let n_asp = crate::aspects::aspect_count(ctx.s);
    let n_steps = ((to_jd - from_jd) / step).ceil() as usize;
    let mut hits = Vec::new();
    let mut prev = ctx.lon(mover, from_jd)?;
    for i in 1..=n_steps {
        let (t0, t1) = (from_jd + (i - 1) as f64 * step, (from_jd + i as f64 * step).min(to_jd));
        let cur = ctx.lon(mover, t1)?;
        for n in &natal.bodies {
            for k in 0..n_asp {
                let angle = ASPECTS[k].angle;
                let signs: &[f64] = if angle == 0.0 || angle == 180.0 { &[1.0] } else { &[1.0, -1.0] };
                for &sgn in signs {
                    let target = norm360(n.lon + sgn * angle);
                    let (g0, g1) = (min_diff(prev, target), min_diff(cur, target));
                    if (g0 < 0.0) != (g1 < 0.0) && (g0 - g1).abs() < 90.0 {
                        let jd = refine(ctx, mover, target, t0, t1, g0)?;
                        let l = ctx.lon(mover, jd)?;
                        let d = &ASPECTS[k];
                        hits.push(Hit {
                            jd,
                            mover,
                            mover_name: OBJECTS[mover].name,
                            mover_glyph: OBJECTS[mover].glyph.to_string(),
                            aspect: k,
                            aspect_name: d.name,
                            aspect_glyph: d.glyph.to_string(),
                            color: d.color,
                            natal: n.index,
                            natal_name: n.name,
                            natal_glyph: n.glyph.clone(),
                            retro: ctx.retrograde(mover, jd)?,
                            sign: SIGNS_LOC[sign_of(l)],
                        });
                    }
                }
            }
        }
        prev = cur;
    }
    Ok(hits)
}

fn refine(ctx: &Ctx, obj: usize, target: f64, mut a: f64, mut b: f64, ga: f64) -> Result<f64, String> {
    let neg = ga < 0.0;
    for _ in 0..22 {
        let m = (a + b) / 2.0;
        if (min_diff(ctx.lon(obj, m)?, target) < 0.0) == neg {
            a = m
        } else {
            b = m
        }
    }
    Ok((a + b) / 2.0)
}
