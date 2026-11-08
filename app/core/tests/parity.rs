//! Compares results with reference output of the original Astrolog 8.00 (tests/fixtures).
use astro_core::aspects;
use astro_core::chart::*;
use astro_core::model::*;
use astro_core::search::*;
use astro_core::ephem::julday;

const LAT: f64 = 49.2;
const LON: f64 = 16.6;

fn s() -> Settings {
    Settings::default()
}
fn jd_a() -> f64 {
    julday(1985, 7, 14, 13.5)
}
fn jd_b() -> f64 {
    julday(1990, 3, 2, 5.25)
}
fn chart_a() -> Chart {
    cast(jd_a(), Some((LAT, LON)), &s()).unwrap()
}
fn chart_b() -> Chart {
    cast(jd_b(), Some((50.0 + 5.0 / 60.0, 14.0 + 25.0 / 60.0)), &s()).unwrap()
}
fn fixture(n: &str) -> String {
    std::fs::read_to_string(format!("{}/tests/fixtures/{n}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

const SIGN3: [&str; 12] = ["Ari", "Tau", "Gem", "Can", "Leo", "Vir", "Lib", "Sco", "Sag", "Cap", "Aqu", "Pis"];
const KEYS: [(&str, usize); 13] = [
    ("Sun", 0), ("Moon", 1), ("Merc", 2), ("Venu", 3), ("Mars", 4), ("Jupi", 5), ("Satu", 6),
    ("Uran", 7), ("Nept", 8), ("Plut", 9), ("Nort", 11), ("Asce", 12), ("Midh", 13),
];

/// Parses rows like `Sun : 22Can02'25"` into longitude and house.
fn positions(text: &str) -> Vec<(usize, f64, usize)> {
    let mut out = vec![];
    for l in text.lines() {
        let Some((k, rest)) = l.split_once(':') else { continue };
        let Some(&(_, idx)) = KEYS.iter().find(|x| x.0 == k.trim()) else { continue };
        let r = rest.trim_start();
        let (deg, tail) = r.split_at(r.find(|c: char| c.is_alphabetic()).unwrap());
        let sign = SIGN3.iter().position(|x| tail.starts_with(x)).unwrap();
        let mut parts = tail[3..].split('\'');
        let min: f64 = parts.next().unwrap().parse().unwrap();
        let sec: f64 = parts.next().unwrap().split('"').next().unwrap().parse().unwrap();
        let hs = l.split('[').nth(1).unwrap().trim_start();
        let house: usize = hs.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap();
        out.push((idx, sign as f64 * 30.0 + deg.trim().parse::<f64>().unwrap() + min / 60.0 + sec / 3600.0, house));
    }
    out
}

fn assert_positions(c: &Chart, text: &str, tol: f64, label: &str) {
    let rows = positions(text);
    assert_eq!(rows.len(), 13, "{label}");
    for (i, lon, house) in rows {
        let b = c.bodies.iter().find(|b| b.index == i).unwrap();
        let d = min_diff(b.lon, lon).abs();
        println!("{label} {}: {:.1}\"", b.name, d * 3600.0);
        assert!(d < tol, "{label} {}: {} vs {} ({:.1}\")", b.name, b.lon, lon, d * 3600.0);
        assert_eq!(b.house, Some(house), "{label} dům {}", b.name);
    }
}

#[test]
fn radix_positions_and_houses() {
    assert_positions(&chart_a(), &fixture("radix.txt"), 2.0 / 3600.0, "radix");
}

#[test]
fn progressed_positions() {
    let target = julday(2026, 11, 4, 8.0);
    let c = cast(progressed_jd(jd_a(), target), Some((LAT, LON)), &s()).unwrap();
    assert_positions(&c, &fixture("prog.txt"), 5.0 / 3600.0, "progrese");
}

#[test]
fn composite_positions() {
    assert_positions(&composite(&chart_a(), &chart_b()), &fixture("composite.txt"), 3.0 / 3600.0, "kompozit");
}

fn en(n: &str) -> &str {
    match n {
        "Slunce" => "Sun",
        "Měsíc" => "Moon",
        "Merkur" => "Merc",
        "Venuše" => "Venu",
        "Mars" => "Mars",
        "Jupiter" => "Jupi",
        "Saturn" => "Satu",
        "Uran" => "Uran",
        "Neptun" => "Nept",
        "Pluto" => "Plut",
        "Severní uzel" => "Nort",
        "Ascendent" => "Asce",
        "Medium coeli" => "Midh",
        x => x,
    }
}
fn asp3(n: &str) -> &str {
    match n {
        "Konjunkce" => "Con",
        "Opozice" => "Opp",
        "Kvadratura" => "Squ",
        "Trigon" => "Tri",
        "Sextil" => "Sex",
        x => x,
    }
}

/// Rows like `Sun (Can) Con (Can) Mars - orb: +1:04' - power: 17.30`; a row number comes first.
fn aspect_rows(text: &str) -> Vec<(String, String, String, f64)> {
    text.lines()
        .filter(|l| l.contains("orb:"))
        .map(|l| {
            let t: Vec<&str> = l.split_whitespace().filter(|w| *w != "with").collect();
            let p = l.rsplit("power:").next().unwrap().trim().parse().unwrap();
            let name = |w: &str| w.chars().take(4).collect::<String>();
            let aspect_at = t.iter().position(|w| ASP3.contains(w)).unwrap();
            (name(t[1]), t[aspect_at].to_string(), name(t[aspect_at + 2]), p)
        })
        .collect()
}
const ASP3: [&str; 5] = ["Con", "Opp", "Squ", "Tri", "Sex"];

fn compare_aspects(mine: &[aspects::Aspect], text: &str) {
    let rows = aspect_rows(text);
    let mut got: Vec<_> = mine.iter().map(|a| (en(a.a_name).to_string(), asp3(a.aspect_name).to_string(), en(a.b_name).to_string(), a.power)).collect();
    assert_eq!(got.len(), rows.len(), "počet aspektů");
    for r in &rows {
        let i = got.iter().position(|g| g.0 == r.0 && g.1 == r.1 && g.2 == r.2).unwrap_or_else(|| panic!("chybí {r:?}"));
        let g = got.remove(i);
        assert!((g.3 - r.3).abs() < 0.05, "síla {r:?} vs {g:?}");
    }
}

#[test]
fn natal_aspects() {
    compare_aspects(&aspects::within(&chart_a(), &s()), &fixture("aspects.txt"));
}

#[test]
fn synastry_aspects() {
    compare_aspects(&aspects::between(&chart_a(), &chart_b(), &s(), true), &fixture("synastry.txt"));
}

/// Fixture times are in zone 8W, hence the +8 hours.
fn parse_hits(text: &str) -> Vec<(f64, String, String, String)> {
    text.lines()
        .filter(|l| l.contains(" natal "))
        .map(|l| {
            let t: Vec<&str> = l.split_whitespace().collect();
            let pos = t.iter().position(|w| *w == "trans" || *w == "progr").unwrap();
            let date: String = t[..pos - 1].concat();
            let mut d = date.split('/');
            let (m, day, y): (u32, u32, i32) = (d.next().unwrap().parse().unwrap(), d.next().unwrap().parse().unwrap(), d.next().unwrap().parse().unwrap());
            let clock = t[pos - 1];
            let (hm, ap) = clock.split_at(clock.len() - 2);
            let (h, min) = hm.split_once(':').unwrap();
            let hour = h.parse::<i32>().unwrap() % 12 + if ap == "pm" { 12 } else { 0 };
            let jd = julday(y, m, day, hour as f64 + min.parse::<f64>().unwrap() / 60.0 + 8.0);
            let natal = t.iter().position(|w| *w == "natal").unwrap();
            let sn: String = t[natal + 2].chars().take(4).collect();
            let mv: String = t[pos + 1].chars().take(4).collect();
            let (mv, sn) = (fix_two_word(&mv), fix_two_word(&sn));
            (jd, mv, t[natal - 1].to_string(), sn)
        })
        .collect()
}
fn fix_two_word(s: &str) -> String {
    if s == "Nort" { "Nort".into() } else { s.into() }
}

#[test]
fn yearly_transits_match() {
    let c = chart_a();
    let (start, end) = (julday(2026, 1, 1, 8.0), julday(2027, 1, 1, 8.0));
    let hits = find_hits(&c, (LAT, LON), start, end, Mode::Transit, &s()).unwrap();
    let mut mine: Vec<_> = hits.iter().map(|h| (h.jd, en(h.mover_name).to_string(), asp3(h.aspect_name).to_string(), en(h.natal_name).to_string())).collect();
    let refs = parse_hits(&fixture("transits2026.txt"));
    let mut missing = vec![];
    let mut worst: f64 = 0.0;
    for r in &refs {
        match mine.iter().position(|g| g.1 == r.1 && g.2 == r.2 && g.3 == r.3 && (g.0 - r.0).abs() < 0.25) {
            Some(i) => {
                worst = worst.max((mine[i].0 - r.0).abs() * 1440.0);
                mine.remove(i);
            }
            None => missing.push(r.clone()),
        }
    }
    println!("reference {}, chybí {}, navíc {}, nejhorší rozdíl {worst:.1} min", refs.len(), missing.len(), mine.len());
    println!("chybí: {:?}", &missing[..missing.len().min(8)]);
    println!("navíc: {:?}", &mine[..mine.len().min(8)]);
    assert!(missing.is_empty() && mine.is_empty());
    // Astrolog interpolates linearly inside its search segments, which drifts near planetary
    // stations (worst case 23 minutes); this code bisects on real positions.
    assert!(worst < 30.0, "{worst}");
}
