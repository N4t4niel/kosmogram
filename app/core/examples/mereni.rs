//! Měří dobu hledání tranzitů a progresí: cargo run --release -p astro-core --example mereni
use astro_core::chart::cast;
use astro_core::model::Settings;
use astro_core::search::{find_hits, Mode};
use astro_core::ephem::julday;
use std::time::Instant;

fn main() {
    let place = (50.088, 14.4208);
    let s = Settings::default();
    let natal = cast(julday(1985, 7, 14, 12.5), Some(place), &s).unwrap();
    let (y, m) = (julday(2026, 1, 1, 0.0), julday(2026, 2, 1, 0.0));
    let e = julday(2027, 1, 1, 0.0);
    for (label, to, mode) in [("tranzity měsíc", m, Mode::Transit), ("tranzity rok", e, Mode::Transit), ("progrese rok", e, Mode::Progression)] {
        let mut times = Vec::new();
        let mut n = 0;
        for _ in 0..7 {
            let t = Instant::now();
            n = find_hits(&natal, place, y, to, mode, &s).unwrap().len();
            times.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        times.sort_by(f64::total_cmp);
        println!("{label}: {n} událostí, medián {:.1} ms, nejrychlejší {:.1} ms", times[3], times[0]);
    }
}
