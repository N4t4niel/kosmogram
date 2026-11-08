//! Applying/separating labels and synastry house placement, checked against Astrolog output.
use astro_core::aspects::*;
use astro_core::chart::*;
use astro_core::model::*;
use astro_core::ephem::julday;
use astro_core::views;

fn fixture(n: &str) -> String {
    std::fs::read_to_string(format!("{}/tests/fixtures/{n}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}
fn en(n: &str) -> String {
    let e = match n {
        "Slunce" => "Sun",
        "Měsíc" => "Moon",
        "Merkur" => "Merc",
        "Venuše" => "Venu",
        "Jupiter" => "Jupi",
        "Saturn" => "Satu",
        "Uran" => "Uran",
        "Neptun" => "Nept",
        "Pluto" => "Plut",
        "Severní uzel" => "Nort",
        "Ascendent" => "Asce",
        "Medium coeli" => "Midh",
        x => x,
    };
    e.to_string()
}

#[test]
fn applying_matches_astrolog() {
    let c = cast(julday(1985, 7, 14, 13.5), Some((49.2, 16.6)), &Settings::default()).unwrap();
    let list = within(&c, &Settings::default());
    let text = fixture("aspects_aa.txt");
    let mut checked = 0;
    for l in text.lines().filter(|l| l.contains("orb:")) {
        let t: Vec<&str> = l.split_whitespace().collect();
        let a: String = t[1].chars().take(4).collect();
        let open = t.iter().position(|w| w.starts_with('(') || w.starts_with('[')).unwrap();
        let b: String = t[open + 3].chars().take(4).collect();
        let orb = l.split("orb:").nth(1).unwrap().trim_start();
        let applying = orb.starts_with('a');
        let mine = list.iter().find(|x| en(x.a_name) == a && en(x.b_name) == b).unwrap_or_else(|| panic!("{a} {b}"));
        assert_eq!(mine.applying, applying, "{l}");
        checked += 1;
    }
    assert!(checked > 15);
}

#[test]
fn transit_applying_depends_on_the_transiting_body_only() {
    let s = Settings::default();
    let mut natal = cast(julday(1985, 7, 14, 13.5), None, &s).unwrap();
    natal.bodies.truncate(1);
    natal.bodies[0] = make_body(0, 0.0, 0.0, 1.0, None);
    let mut moving = natal.clone();
    moving.bodies[0] = make_body(5, 359.0, 0.0, 0.2, None);
    assert!(between(&moving, &natal, &s, false)[0].applying, "Jupiter před bodem se blíží");
    moving.bodies[0] = make_body(5, 1.0, 0.0, 0.2, None);
    assert!(!between(&moving, &natal, &s, false)[0].applying, "Jupiter za bodem se vzdaluje");
    moving.bodies[0] = make_body(5, 1.0, 0.0, -0.2, None);
    assert!(between(&moving, &natal, &s, false)[0].applying, "zpětný Jupiter za bodem se blíží");
}

fn person(name: &str, date: &str, time: &str, lat: f64, lon: f64) -> Person {
    Person {
        id: name.into(),
        name: name.into(),
        date: date.into(),
        time: time.into(),
        time_known: true,
        sex: String::new(),
        place: Place { name: "x".into(), region: String::new(), country: String::new(), lat, lon, tz: "Europe/Prague".into() },
        offset_min: Some(60),
        note: String::new(),
    }
}

#[test]
fn synastry_table_uses_the_hosts_houses() {
    let a = person("A", "1985-07-14", "14:30", 49.2, 16.6);
    let b = person("B", "1990-03-02", "06:15", 50.0 + 5.0 / 60.0, 14.0 + 25.0 / 60.0);
    let r = views::relation(&a, &b, &Settings::default()).unwrap();
    let text = fixture("synastry_houses.txt");
    let mut checked = 0;
    for row in &r.synastry.rows {
        let key = en(row.name);
        let l = text.lines().find(|l| l.split(':').next().unwrap().trim() == key).unwrap_or_else(|| panic!("{key}"));
        let hs = l.split('[').nth(1).unwrap().trim_start();
        let house: usize = hs.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap();
        assert_eq!(row.house, Some(house), "{key}");
        checked += 1;
    }
    assert_eq!(checked, 11, "bez Asc a MC druhé osoby");
}
