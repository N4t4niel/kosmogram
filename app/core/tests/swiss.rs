//! Compares with Swiss Ephemeris, whose results stay here as plain numbers; the library itself is not
//! part of this project. Limits are in arcseconds, and the program shows positions to 60″.
use astro_core::{ephem, houses};

fn arcsec(a: f64, b: f64) -> f64 {
    ((a - b + 540.0).rem_euclid(360.0) - 180.0).abs() * 3600.0
}

#[test]
fn positions_match() {
    let (mut lon, mut lat) = ([0.0_f64; 16], [0.0_f64; 16]);
    for row in include_str!("fixtures/swiss_polohy.txt").lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<f64> = row.split(' ').map(|x| x.parse().unwrap()).collect();
        let body = f[0] as usize;
        let p = ephem::position_tt(f[1], f[0] as i32).unwrap();
        lon[body] = lon[body].max(arcsec(p.lon, f[2]));
        lat[body] = lat[body].max((p.lat - f[3]).abs() * 3600.0);
    }
    for body in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15] {
        println!("{body:3} {:8.4} {:8.4}", lon[body], lat[body]);
        // Swiss has an older orbit of Chiron and its own theory of the mean node.
        let limit = match body { 15 => 15.0, 10 => 1.0, _ => 0.1 };
        assert!(lon[body] < limit && lat[body] < limit, "{body}: {}″ {}″", lon[body], lat[body]);
    }
}

#[test]
fn houses_match_and_fail_in_the_same_places() {
    let mut rows = 0;
    for row in include_str!("fixtures/swiss_domy.txt").lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = row.split(' ').collect();
        let system = f[0].as_bytes()[0];
        let [jd, lat, lon]: [f64; 3] = [f[1].parse().unwrap(), f[2].parse().unwrap(), f[3].parse().unwrap()];
        let ours = houses::houses(jd, lat, lon, system);
        if f[4] == "chyba" {
            assert!(ours.is_err(), "{row}");
        } else {
            let (c, asc, mc) = ours.unwrap_or_else(|e| panic!("{row}: {e}"));
            let theirs: Vec<f64> = f[4..].iter().map(|x| x.parse().unwrap()).collect();
            // Polar cusps are harder to reach, and fixed to six decimals here, which is 0.004″.
            let limit = if lat.abs() > 65.0 { 5.0 } else { 0.2 };
            for (a, b) in c.iter().chain([&asc, &mc]).zip(&theirs) {
                assert!(arcsec(*a, *b) < limit, "{row}: {}″", arcsec(*a, *b));
            }
            let (fast_asc, fast_mc) = houses::angles(jd, lat, lon, system);
            assert!(arcsec(fast_asc, asc) < 1e-6 && arcsec(fast_mc, mc) < 1e-6, "rychlá cesta se liší");
        }
        rows += 1;
    }
    assert_eq!(rows, 4800);
}
