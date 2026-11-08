//! An independent check: JPL's own apparent places, computed by neither program. This covers the
//! first months of 1800, where the Swiss Ephemeris file is unusable, and the far future alike.
//!
//! Horizons measures from the IAU 1976 equinox, whose precession is known to run 0.3″ per century
//! fast, while we use the current IAU model. That moves every body alike, so each one is compared
//! after removing the offset of the Sun at the same moment; the offset itself is checked separately.
use astro_core::ephem;
use std::collections::BTreeMap;

const LAST_NEAR_SUN: f64 = 1.5;

fn signed(a: f64, b: f64) -> f64 {
    ((a - b + 540.0).rem_euclid(360.0) - 180.0) * 3600.0
}

#[test]
fn positions_match_jpl_horizons() {
    let rows: Vec<Vec<f64>> = include_str!("fixtures/horizons.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split(' ').map(|x| x.parse().unwrap()).collect())
        .collect();
    let mut sun = BTreeMap::new();
    for r in rows.iter().filter(|r| r[0] == 0.0) {
        let p = ephem::position_tt(r[1], 0).unwrap();
        sun.insert(r[1].to_bits(), (signed(p.lon, r[2]), r[2], r[3]));
    }
    // body -> worst longitude, worst latitude, worst near the Sun, count
    let mut stats: BTreeMap<i32, (f64, f64, f64, usize)> = BTreeMap::new();
    for r in &rows {
        let (body, jd) = (r[0] as i32, r[1]);
        let (offset, sun_lon, sun_lat) = sun[&jd.to_bits()];
        let years = (jd - 2451545.0) / 36525.0;
        if body == 0 {
            // 0.3″ per century away from J2000, in the direction of the IAU 1976 error.
            assert!((offset.abs() - 0.3 * years.abs()).abs() < 0.2, "posun {offset}″ v {years}");
            continue;
        }
        let p = ephem::position_tt(jd, body).unwrap();
        let dl = (signed(p.lon, r[2]) - offset).abs() * r[3].to_radians().cos();
        let db = (p.lat - r[3]).abs() * 3600.0;
        let from_sun = ((r[2] - sun_lon).to_radians().cos() * (r[3] - sun_lat).to_radians().cos()).clamp(-1.0, 1.0).acos().to_degrees();
        let e = stats.entry(body).or_default();
        if from_sun < LAST_NEAR_SUN {
            e.2 = e.2.max(dl).max(db);
        } else {
            e.0 = e.0.max(dl);
            e.1 = e.1.max(db);
        }
        e.3 += 1;
    }
    println!("těleso  délka ″  šířka ″  u Slunce ″  počet");
    for (body, (dl, db, near, n)) in &stats {
        println!("{body:6} {dl:8.3} {db:8.3} {near:11.3} {n:6}");
        // The Moon reacts first to any error in the place of the Earth; 0.04″ of it is DE440 against JPL's DE441.
        assert!(*dl < 0.06 && *db < 0.1 && *near < 3.0, "{body}: {dl}″ {db}″ {near}″");
    }
    assert_eq!(stats.len(), 10);
}
