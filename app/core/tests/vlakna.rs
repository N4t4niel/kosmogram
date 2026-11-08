//! Calculations must not depend on which thread runs them or on what other threads are doing.
use astro_core::chart::{cast, Chart};
use astro_core::model::Settings;
use astro_core::search::{find_hits, find_hits_serial, Hit, Mode};
use astro_core::ephem::{julday, position};
use astro_core::houses::houses;

const PLACES: [(f64, f64); 3] = [(50.088, 14.4208), (49.195, 16.608), (-33.87, 151.21)];

fn settings() -> Vec<Settings> {
    let d = Settings::default();
    vec![
        d.clone(),
        Settings { chiron: true, minor_aspects: true, ..d.clone() },
        Settings { house_system: 'K', chiron: true, ..d.clone() },
        Settings { house_system: 'R', ..d },
    ]
}

/// The raw bits of jd plus every serialised field: equal text means bit-equal results.
fn exact(hits: &[Hit]) -> Vec<String> {
    hits.iter().map(|h| format!("{} {}", h.jd.to_bits(), serde_json::to_string(h).unwrap())).collect()
}

fn natal(y: i32, place: (f64, f64), s: &Settings) -> Chart {
    cast(julday(y, 7, 14, 12.5), Some(place), s).unwrap()
}

#[test]
fn parallel_search_equals_serial_to_the_last_bit() {
    let mut checked = 0;
    for (i, s) in settings().iter().enumerate() {
        for (j, &place) in PLACES.iter().enumerate() {
            for (k, year) in [1950, 1985, 2003].into_iter().enumerate() {
                let n = natal(year, place, s);
                let from = julday(2020 + (i + j + k) as i32, 1, 1, 0.0);
                for (mode, days) in [(Mode::Transit, 366.0), (Mode::Transit, 31.0), (Mode::Progression, 366.0)] {
                    let a = find_hits(&n, place, from, from + days, mode, s).unwrap();
                    let b = find_hits_serial(&n, place, from, from + days, mode, s).unwrap();
                    assert_eq!(exact(&a), exact(&b), "{year} {place:?} {mode:?} {days}");
                    checked += a.len();
                }
            }
        }
    }
    println!("porovnáno událostí: {checked}");
    assert!(checked > 10_000, "jen {checked} událostí");
}

#[test]
fn search_results_repeat_exactly() {
    let s = &settings()[1];
    let place = PLACES[0];
    let n = natal(1985, place, s);
    let from = julday(2026, 1, 1, 0.0);
    let first = exact(&find_hits(&n, place, from, from + 365.0, Mode::Transit, s).unwrap());
    for _ in 0..20 {
        assert_eq!(first, exact(&find_hits(&n, place, from, from + 365.0, Mode::Transit, s).unwrap()));
    }
}

/// Each thread walks its own dates, so any state shared between threads would corrupt the others.
#[test]
fn twelve_threads_at_once_match_single_thread() {
    fn work(seed: usize) -> Vec<u64> {
        let mut out = Vec::new();
        for i in 0..3000 {
            let jd = 2_379_000.0 + ((seed * 7919 + i * 104_729) % 200_000) as f64 + i as f64 / 7.0;
            let body = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15][i % 12];
            let p = position(jd, body).unwrap();
            out.extend([p.lon, p.lat, p.speed].iter().map(|v| v.to_bits()));
            let (c, asc, mc) = houses(jd, 50.0 - seed as f64, 14.0 + seed as f64, b"PKRCOBEW"[i % 8]).unwrap();
            out.extend(c.iter().chain([&asc, &mc]).map(|v| v.to_bits()));
        }
        out
    }
    // Every thread starts fresh, also in the reference run, so a warm cache cannot hide a difference.
    let expected: Vec<_> = (0..12).map(|t| std::thread::spawn(move || work(t)).join().unwrap()).collect();
    for round in 0..3 {
        let got: Vec<_> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..12).map(|t| sc.spawn(move || work(t))).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        assert!(got == expected, "kolo {round} se liší");
    }
}
