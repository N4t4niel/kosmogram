use crate::data::*;
use crate::model::{Person, Settings};
use crate::{ephem, houses};
use crate::timeconv::{person_moment, Moment};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Body {
    pub index: usize,
    pub id: &'static str,
    pub name: &'static str,
    pub glyph: String,
    pub lon: f64,
    pub lat: f64,
    pub speed: f64,
    pub retro: bool,
    pub sign: usize,
    pub sign_name: &'static str,
    pub position: String,
    pub house: Option<usize>,
    pub dignity: Option<char>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Chart {
    pub bodies: Vec<Body>,
    pub cusps: Option<[f64; 12]>,
    pub jd_ut: f64,
}

pub fn norm360(x: f64) -> f64 {
    let r = x.rem_euclid(360.0);
    if r >= 360.0 { 0.0 } else { r }
}

/// Signed shortest difference a - b in (-180, 180].
pub fn min_diff(a: f64, b: f64) -> f64 {
    let d = norm360(a - b);
    if d > 180.0 { d - 360.0 } else { d }
}

pub fn sign_of(lon: f64) -> usize {
    ((norm360(lon) / 30.0) as usize).min(11)
}

pub fn format_pos(lon: f64) -> String {
    let l = norm360(lon);
    let s = sign_of(l);
    let minutes = ((l - s as f64 * 30.0) * 60.0).floor() as i32;
    format!("{}°{:02}′ {}", minutes / 60, minutes % 60, SIGNS[s])
}

pub fn dignity(obj: &Obj, sign: usize) -> Option<char> {
    if obj.rulers.contains(&sign) {
        return Some('R');
    }
    if obj.rulers.iter().any(|r| (r + 6) % 12 == sign) {
        return Some('d');
    }
    match obj.exalt {
        Some(e) if e == sign => Some('X'),
        Some(e) if (e + 6) % 12 == sign => Some('f'),
        _ => None,
    }
}

pub fn house_of(lon: f64, cusps: &[f64; 12]) -> usize {
    for i in 0..12 {
        let a = cusps[i];
        let span = norm360(cusps[(i + 1) % 12] - a);
        if norm360(lon - a) < span {
            return i + 1;
        }
    }
    1
}

pub fn make_body(index: usize, lon: f64, lat: f64, speed: f64, cusps: Option<&[f64; 12]>) -> Body {
    let obj = &OBJECTS[index];
    let sign = sign_of(lon);
    let is_angle = matches!(obj.kind, ObjKind::Asc | ObjKind::Mc);
    Body {
        index,
        id: obj.id,
        name: obj.name,
        glyph: obj.glyph.to_string(),
        lon: norm360(lon),
        lat,
        speed,
        retro: !is_angle && speed < 0.0,
        sign,
        sign_name: SIGNS[sign],
        position: format_pos(lon),
        house: cusps.map(|c| house_of(lon, c)),
        dignity: dignity(obj, sign),
    }
}

pub fn active_objects(s: &Settings, with_angles: bool) -> Vec<usize> {
    (0..OBJECTS.len())
        .filter(|&i| (s.chiron || i != CHIRON) && (with_angles || !matches!(OBJECTS[i].kind, ObjKind::Asc | ObjKind::Mc)))
        .collect()
}

/// Casts a chart; `place` is (lat, lon), and `None` means no houses or angles.
pub fn cast(jd: f64, place: Option<(f64, f64)>, s: &Settings) -> Result<Chart, String> {
    cast_with(jd, place, s, |t, lat, lon| houses::houses(t, lat, lon, s.house_system as u8))
}

/// Degrees the progressed Sun has moved since birth.
pub fn solar_arc(natal_jd: f64, jd: f64) -> Result<f64, String> {
    Ok(norm360(ephem::longitude(jd, 0)? - ephem::longitude(natal_jd, 0)?))
}

/// The progressed chart. By default it is cast whole for the progressed moment, like Astrolog, so the
/// angles turn once a year. With `solar_arc_angles` the planets stay, and the midheaven moves by the
/// solar arc with the other cusps recomputed from it.
pub fn progressed(natal_jd: f64, target_jd: f64, place: Option<(f64, f64)>, s: &Settings) -> Result<Chart, String> {
    let jd = progressed_jd(natal_jd, target_jd);
    if !s.solar_arc_angles {
        return cast(jd, place, s);
    }
    cast_with(jd, place, s, |t, lat, lon| houses::houses_by_arc(natal_jd, t, solar_arc(natal_jd, t)?, lat, lon, s.house_system as u8))
}

fn cast_with(
    jd: f64,
    place: Option<(f64, f64)>,
    s: &Settings,
    houses_for: impl Fn(f64, f64, f64) -> Result<([f64; 12], f64, f64), String>,
) -> Result<Chart, String> {
    let mut cusps = None;
    let mut angles = None;
    let mut angle_speed = (0.0, 0.0);
    if let Some((lat, lon)) = place {
        let (c, asc, mc) = houses_for(jd, lat, lon)?;
        cusps = Some(c);
        angles = Some((asc, mc));
        // Aspects to the angles apply or separate by this speed.
        const DT: f64 = 1.0 / 1440.0;
        let (_, asc2, mc2) = houses_for(jd + DT, lat, lon)?;
        angle_speed = (min_diff(asc2, asc) / DT, min_diff(mc2, mc) / DT);
    }
    let mut bodies = Vec::new();
    for i in active_objects(s, angles.is_some()) {
        let b = match OBJECTS[i].kind {
            ObjKind::Body(se) => {
                let p = ephem::position(jd, se)?;
                make_body(i, p.lon, p.lat, p.speed, cusps.as_ref())
            }
            ObjKind::Asc => make_body(i, angles.unwrap().0, 0.0, angle_speed.0, cusps.as_ref()),
            ObjKind::Mc => make_body(i, angles.unwrap().1, 0.0, angle_speed.1, cusps.as_ref()),
        };
        bodies.push(b);
    }
    Ok(Chart { bodies, cusps, jd_ut: jd })
}

pub fn natal(p: &Person, s: &Settings) -> Result<(Chart, Moment), String> {
    let m = person_moment(p)?;
    let place = p.time_known.then_some((p.place.lat, p.place.lon));
    Ok((cast(m.jd_ut, place, s)?, m))
}

/// Secondary progression: one day after birth stands for one year of life.
pub fn progressed_jd(natal_jd: f64, target_jd: f64) -> f64 {
    natal_jd + (target_jd - natal_jd) / DAYS_IN_YEAR
}

/// Midpoint along the shorter arc.
pub fn midpoint(a: f64, b: f64) -> f64 {
    norm360(a + min_diff(b, a) / 2.0)
}

fn angle_cusp(index: usize) -> Option<usize> {
    match OBJECTS[index].kind {
        ObjKind::Asc => Some(0),
        ObjKind::Mc => Some(9),
        _ => None,
    }
}

pub fn composite(a: &Chart, b: &Chart) -> Chart {
    let cusps = match (&a.cusps, &b.cusps) {
        (Some(x), Some(y)) => {
            let mut c = [0.0; 12];
            for i in 0..12 {
                c[i] = midpoint(x[i], y[i]);
            }
            // Astrolog: far-apart cusps can land 180° off, so line every cusp up with the MC.
            let mc = c[9];
            for i in 0..12 {
                if min_diff(mc, c[i] - (i as f64 + 3.0) * 30.0).abs() > 90.0 {
                    c[i] = norm360(c[i] + 180.0);
                }
            }
            Some(c)
        }
        _ => None,
    };
    let bodies = a
        .bodies
        .iter()
        .filter_map(|x| {
            let y = b.bodies.iter().find(|y| y.index == x.index)?;
            let mut lon = midpoint(x.lon, y.lon);
            if let (Some(c), Some(cusp)) = (&cusps, angle_cusp(x.index)) {
                if min_diff(c[cusp], lon).abs() > 90.0 {
                    lon = norm360(lon + 180.0);
                }
            }
            Some(make_body(x.index, lon, (x.lat + y.lat) / 2.0, (x.speed + y.speed) / 2.0, cusps.as_ref()))
        })
        .collect();
    Chart { bodies, cusps, jd_ut: (a.jd_ut + b.jd_ut) / 2.0 }
}
