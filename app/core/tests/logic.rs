use astro_core::model::*;
use astro_core::places::{normalize, search};
use astro_core::timeconv::*;
use astro_core::views;

fn off(date: &str, time: &str, tz: &str) -> i32 {
    local_to_moment(parse_date(date).unwrap().and_time(parse_time(time).unwrap()), tz, None).unwrap().offset_min
}

#[test]
fn czech_time_history() {
    assert_eq!(off("1985-07-14", "14:30", "Europe/Prague"), 120, "letní čas");
    assert_eq!(off("1985-01-14", "14:30", "Europe/Prague"), 60);
    assert_eq!(off("1975-07-14", "14:30", "Europe/Prague"), 60, "v 70. letech do 1979 letní čas nebyl");
    assert_eq!(off("1943-07-14", "14:30", "Europe/Prague"), 120, "válečný letní čas");
}

/// Offset of a Czech wall clock and whether it needed a note.
fn prague(date: &str, time: &str) -> (i32, bool) {
    let m = local_to_moment(parse_date(date).unwrap().and_time(parse_time(time).unwrap()), "Europe/Prague", None).unwrap();
    (m.offset_min, m.note.is_some())
}

#[test]
fn czechoslovak_law_wins_over_the_iana_history() {
    // 181/1946 Zb.: summer time ended on 7 October at three, not on the 6th.
    assert_eq!(prague("1946-10-06", "12:00"), (120, false));
    assert_eq!(prague("1946-10-06", "02:30"), (120, false));
    assert_eq!(prague("1946-10-07", "02:30"), (120, true));
    assert_eq!(prague("1946-10-07", "03:30"), (60, false));
    // 54/1949 Zb.: it began on 10 April at two, not on the 9th.
    assert_eq!(prague("1949-04-09", "12:00"), (60, false));
    assert_eq!(prague("1949-04-10", "01:30"), (60, false));
    assert!(prague("1949-04-10", "02:30").1);
    assert_eq!(prague("1949-04-10", "12:00"), (120, false));
    // 36/1979 Zb.: clocks moved at midnight.
    assert!(prague("1979-04-01", "00:30").1);
    assert_eq!(prague("1979-04-01", "01:30"), (120, false));
    assert_eq!(prague("1979-09-30", "00:30"), (120, true));
    assert_eq!(prague("1979-09-30", "02:30"), (60, false));
    // 31/1980 Zb. moved the start to 6 April, which IANA has right apart from the hour.
    assert_eq!(prague("1980-03-30", "12:00"), (60, false));
    assert_eq!(prague("1980-04-06", "01:30"), (120, false));
    assert_eq!(prague("1982-09-26", "02:30"), (60, false));
    // The neighbours and the days around stay as they were.
    assert_eq!(prague("1946-10-08", "12:00"), (60, false));
    assert_eq!(prague("1983-03-27", "01:30"), (60, false));
    assert_eq!(off("1979-04-01", "01:30", "Europe/Bratislava"), 120);
    assert_eq!(off("1979-04-01", "01:30", "Europe/Vienna"), 60);
}

#[test]
fn utc_back_to_a_czech_clock_follows_the_law() {
    for (date, time, offset) in [("1946-10-06", "12:00", 120), ("1949-04-09", "12:00", 60), ("1979-04-01", "01:30", 120)] {
        let local = parse_date(date).unwrap().and_time(parse_time(time).unwrap());
        let m = local_to_moment(local, "Europe/Prague", None).unwrap();
        assert_eq!(m.offset_min, offset);
        assert_eq!(jd_to_local(m.jd_ut, "Europe/Prague"), local);
    }
}

#[test]
fn before_standard_time_the_clock_follows_the_place() {
    let born = |date: &str, lon: f64| {
        let place = Place { lat: 49.2, lon, tz: "Europe/Prague".into(), ..Place::default() };
        person_moment(&Person { date: date.into(), time: "12:00".into(), time_known: true, place, ..Person::default() }).unwrap()
    };
    let brno = born("1880-05-01", 16.6);
    assert_eq!(brno.offset_min, 66, "Brno, ne pražský čas");
    assert!(brno.note.is_some());
    assert_eq!(born("1880-05-01", 14.4167).offset_min, 57);
    assert_eq!(born("1891-10-02", 16.6).offset_min, 60);
    assert!(born("1900-10-02", 16.6).note.is_none());
}

#[test]
fn uncertain_periods_are_marked_and_certain_ones_are_not() {
    let note = |date: &str, tz: &str, known: bool, manual: Option<i32>| {
        let place = Place { lat: 49.2, lon: 16.6, tz: tz.into(), ..Place::default() };
        person_moment(&Person { date: date.into(), time: "12:00".into(), time_known: known, place, offset_min: manual, ..Person::default() }).unwrap().note
    };
    assert!(note("1895-05-01", "Europe/Prague", true, None).unwrap().contains("postupně"));
    assert!(note("1943-05-01", "Europe/Prague", true, None).unwrap().contains("války"));
    assert!(note("1941-05-01", "Europe/Bratislava", true, None).unwrap().contains("Slovensko"));
    assert!(note("1960-05-01", "Europe/Vienna", true, None).unwrap().contains("neověřovali"));
    for (date, tz) in [("1910-05-01", "Europe/Prague"), ("1947-05-01", "Europe/Prague"), ("1985-05-01", "Europe/Prague"), ("1985-05-01", "Europe/Vienna")] {
        assert_eq!(note(date, tz, true, None), None, "{date} {tz}");
    }
    assert_eq!(note("1943-05-01", "Europe/Prague", true, Some(120)), None, "ruční posun");
    assert_eq!(note("1943-05-01", "Europe/Prague", false, None), None, "čas neznámý");
}

#[test]
fn solar_return_finds_the_sun_on_its_birth_position() {
    let place = Place { name: "Brno".into(), lat: 49.2, lon: 16.6, tz: "Europe/Prague".into(), ..Place::default() };
    let p = Person { date: "1985-07-14".into(), time: "14:30".into(), time_known: true, place: place.clone(), ..Person::default() };
    let s = Settings::default();
    let natal = views::radix(&p, &s).unwrap();
    for year in [1985, 1986, 2026, 2300] {
        let r = views::solar_return(&p, year, None, &s).unwrap();
        assert_eq!(r.rows[0].position, natal.rows[0].position, "{year}");
        assert!(r.info[0].contains(&format!(". 7. {year}")), "{}", r.info[0]);
    }
    let elsewhere = Place { name: "Tokio".into(), lat: 35.68, lon: 139.69, tz: "Asia/Tokyo".into(), ..Place::default() };
    let (a, b) = (views::solar_return(&p, 2026, None, &s).unwrap(), views::solar_return(&p, 2026, Some(&elsewhere), &s).unwrap());
    assert_eq!(a.rows[0].position, b.rows[0].position, "okamžik nezávisí na místě");
    assert_ne!(a.cusps[0].position, b.cusps[0].position, "domy ano");
    assert!(b.info[0].contains("UTC+9") && b.info[0].contains("Tokio"), "{}", b.info[0]);
    assert!(views::solar_return(&p, 1984, None, &s).is_err());
    let unknown = Person { time_known: false, ..p };
    assert!(views::solar_return(&unknown, 2026, None, &s).unwrap().cusps.is_empty());
}

#[test]
fn changeover_gets_a_note() {
    let gap = parse_date("2024-03-31").unwrap().and_time(parse_time("02:30").unwrap());
    assert!(local_to_moment(gap, "Europe/Prague", None).unwrap().note.is_some());
    let twice = parse_date("2024-10-27").unwrap().and_time(parse_time("02:30").unwrap());
    assert!(local_to_moment(twice, "Europe/Prague", None).unwrap().note.is_some());
}

#[test]
fn search_ignores_diacritics_and_prefers_czech() {
    assert_eq!(normalize("České Budějovice"), "ceske budejovice");
    assert_eq!(search("ceske bud", 5)[0].place.name, "České Budějovice");
    assert_eq!(search("praha", 5)[0].place.name, "Praha");
    assert_eq!(search("Prague", 5)[0].place.name, "Praha");
    assert_eq!(search("vienna", 5)[0].place.name, "Vídeň");
    assert_eq!(search("br", 8)[0].place.name, "Brno");
    assert!(search("horni lidec", 5).iter().any(|f| f.place.name == "Horní Lideč"));
    assert!(search("nova ves melnik", 5).iter().any(|f| f.place.name == "Nová Ves" && f.place.region.contains("Mělník")));
    assert!(search("hallstatt", 5).iter().any(|f| f.place.country == "Rakousko"), "malá obec v sousední zemi");
    assert_eq!(search("drazdany", 5)[0].place.name, "Drážďany");
    assert_eq!(search("dresden", 5)[0].place.name, "Drážďany");
    let brno = &search("brno", 1)[0].place;
    assert!(brno.lon > 16.0 && brno.lat > 49.0, "východní délka je kladná");
}

fn person(time_known: bool) -> Person {
    let place = search("brno", 1)[0].place.clone();
    Person { id: "1".into(), name: "Jana".into(), date: "1985-07-14".into(), time: "14:30".into(), time_known, place, offset_min: None, note: String::new(), sex: String::new(), group: String::new() }
}

#[test]
fn sky_uses_the_time_zone_of_the_place() {
    let s = Settings::default();
    let praha = search("praha", 1)[0].place.clone();
    let r = views::sky(&praha, "2026-07-01", "12:00", &s).unwrap();
    assert!(r.info.iter().any(|t| t.contains("UTC+2")), "{:?}", r.info);
    assert_eq!(r.cusps.len(), 12);
    let tokyo = search("tokio", 1)[0].place.clone();
    assert!(views::sky(&tokyo, "2026-07-01", "12:00", &s).unwrap().info.iter().any(|t| t.contains("UTC+9")));
    assert!(views::sky(&praha, "1500-07-01", "12:00", &s).is_err());
}

#[test]
fn views_work_with_and_without_birth_time() {
    let s = Settings::default();
    for known in [true, false] {
        let p = person(known);
        let r = views::radix(&p, &s).unwrap();
        assert!(r.svg.starts_with("<svg"));
        assert_eq!(r.cusps.is_empty(), !known);
        assert_eq!(r.rows.iter().any(|b| b.name == "Ascendent"), known);
        views::transits(&p, "2026-11-04", "12:00", &s).unwrap();
        views::progressions(&p, "2026-11-04", &s).unwrap();
        assert!(!views::events(&p, 2026, 11, false, &s).unwrap().is_empty());
        views::events(&p, 2026, 0, true, &s).unwrap();
        views::relation(&p, &person(true), &s).unwrap();
    }
}

#[test]
fn moon_phase_says_how_the_moon_really_looked() {
    // Full moon 13 August 1992, 10:27 UTC. Three days later the Moon was waning.
    let mut p = person(true);
    p.date = "1992-08-16".into();
    p.time = "13:20".into();
    let r = views::radix(&p, &Settings::default()).unwrap();
    let title = r.sections[0].items.iter().map(|i| i.title.clone()).find(|t| t.starts_with("Fáze Měsíce")).unwrap();
    println!("{title}");
    assert!(title.contains("fáze úplňku") && title.contains("po úplňku") && title.contains("ubývá"), "{title}");
}

#[test]
fn all_interpretations_are_complete_sentences() {
    let mut s = Settings::default();
    s.minor_aspects = true;
    s.chiron = true;
    let r = views::radix(&person(true), &s).unwrap();
    for sec in &r.sections {
        for it in &sec.items {
            assert!(it.text.ends_with('.') && !it.text.contains("{}") && !it.text.contains("  "), "{}", it.text);
        }
    }
}


mod vyklady {
    use astro_core::aspects::Aspect;
    use astro_core::data::*;
    use astro_core::interpret::{active_moving, natal_aspects, Section};
    use astro_core::texts;

    fn names() -> Vec<&'static str> {
        OBJECTS.iter().map(|o| o.name).collect()
    }

    /// Mercury and Venus never leave the Sun far enough for these aspects.
    fn impossible(a: &str, b: &str, aspect: &str) -> bool {
        match (a, b) {
            ("Slunce", "Merkur") => !["Konjunkce", "Polosextil"].contains(&aspect),
            ("Slunce", "Venuše") => !["Konjunkce", "Polosextil", "Polokvadrát"].contains(&aspect),
            _ => false,
        }
    }

    #[test]
    fn keys_name_real_things() {
        let bodies = names();
        let houses: Vec<String> = (1..=12).map(|h| format!("{h}. dům")).collect();
        let aspects: Vec<&str> = ASPECTS.iter().map(|a| a.name).collect();
        for key in texts::keys() {
            let p: Vec<&str> = key.split(" | ").collect();
            if !bodies.contains(&p[0]) {
                continue;
            }
            let ok = match p[..] {
                [a, "Zpětný pohyb"] => bodies.contains(&a),
                [a, b] => bodies.contains(&a) && (SIGNS.contains(&b) || houses.iter().any(|h| h == b)),
                [a, b, c] => {
                    let order = |n: &str| bodies.iter().position(|x| *x == n);
                    order(a).is_some() && order(b).is_some() && order(a) < order(b) && aspects.contains(&c) && !impossible(a, b, c)
                }
                _ => false,
            };
            assert!(ok, "chybný klíč: {key}");
        }
    }

    #[test]
    fn texts_are_clean_sentences() {
        for key in texts::keys() {
            let t = texts::get(key).unwrap();
            assert!(t.ends_with('.'), "{key}");
            assert!(t.starts_with('{') || t.chars().next().unwrap().is_uppercase(), "{key}");
            assert!(!t.contains("  ") && !t.contains(" ,") && !t.contains('—'), "{key}");
        }
    }

    #[test]
    fn sun_is_complete() {
        for s in SIGNS {
            assert!(texts::sign("Slunce", s).is_some(), "{s}");
        }
        for h in 1..=12 {
            assert!(texts::house("Slunce", h).is_some(), "{h}");
        }
        for b in &names()[1..] {
            for a in &ASPECTS {
                let found = texts::aspect("Slunce", b, a.name).is_some();
                assert_eq!(found, !impossible("Slunce", b, a.name), "{b} {}", a.name);
            }
        }
    }

    fn aspect(a: usize, b: usize, k: usize, orb: f64) -> Aspect {
        Aspect {
            a,
            b,
            a_name: OBJECTS[a].name,
            b_name: OBJECTS[b].name,
            a_glyph: String::new(),
            b_glyph: String::new(),
            aspect: ASPECTS[k].id,
            aspect_name: ASPECTS[k].name,
            aspect_index: k,
            glyph: String::new(),
            color: "",
            orb,
            max_orb: 8.0,
            applying: false,
            power: 0.0,
        }
    }

    fn only(s: Section) -> String {
        s.items[0].text.clone()
    }

    #[test]
    fn aspect_uses_own_text_or_falls_back() {
        let own = texts::aspect("Slunce", "Saturn", "Kvadratura").unwrap();
        let close = only(natal_aspects(&[aspect(0, 6, 2, 1.0)]));
        assert!(close.starts_with(own) && close.contains("těsný"), "{close}");
        let wide = only(natal_aspects(&[aspect(0, 6, 2, 6.0)]));
        assert!(wide.contains("volný"), "{wide}");
        let old = only(natal_aspects(&[aspect(12, 13, 0, 3.0)]));
        assert!(old.starts_with("Vnější projev"), "{old}");
    }

    #[test]
    fn progressed_axes_follow_the_method() {
        use astro_core::interpret::progression_family;
        use astro_core::model::Settings;
        let days = progression_family(&Settings::default());
        let years = progression_family(&Settings { solar_arc_angles: true, ..Default::default() });
        let text = |mover, family| only(active_moving(&[aspect(mover, 0, 0, 0.5)], "", family));
        assert!(text(12, days).starts_with(texts::get("Progrese denní | Ascendent | Slunce | Konjunkce").unwrap()));
        assert!(text(12, years).starts_with(texts::get("Progrese | Ascendent | Slunce | Konjunkce").unwrap()));
        assert_eq!(text(4, days), text(4, years));
    }

    #[test]
    fn fast_axes_have_every_major_aspect() {
        for axis in ["Ascendent", "Medium coeli"] {
            for b in names() {
                for a in ["Konjunkce", "Opozice", "Kvadratura", "Trigon", "Sextil"] {
                    let key = format!("Progrese denní | {axis} | {b} | {a}");
                    assert!(texts::get(&key).is_some(), "{key}");
                }
            }
        }
    }

    #[test]
    fn preposition_before_sibilant() {
        let t = only(active_moving(&[aspect(1, 10, 0, 3.0)], "", "Bez textu"));
        assert!(t.contains("se starými ranami"), "{t}");
    }
}

mod vyklady_jinde {
    use astro_core::chart::cast;
    use astro_core::interpret::{placements, placements_of, Kind};
    use astro_core::model::Settings;

    #[test]
    fn each_kind_of_chart_has_its_own_texts() {
        let s = Settings::default();
        let c = cast(2_446_000.5, Some((50.0, 15.0)), &s).unwrap();
        let text = |kind| placements_of(&c, kind).items[0].text.clone();
        let (radix, solar, composite) = (placements(&c).items[0].text.clone(), text(Kind::Solar), text(Kind::Composite));
        assert!(radix != solar && radix != composite && solar != composite, "{radix} / {solar} / {composite}");
        assert!(!solar.contains("se projevují") && !composite.contains("se projevují"), "{solar} / {composite}");
    }
}

mod solarni_oblouk {
    use astro_core::chart::*;
    use astro_core::model::*;
    use astro_core::search::{find_hits, Mode};

    const PLACE: (f64, f64) = (50.08, 14.42);
    const NATAL: f64 = 2438834.5 + 11.0 / 24.0;
    const AGE_60: f64 = NATAL + 60.5 * 365.24219;

    fn arc_settings() -> Settings {
        Settings { solar_arc_angles: true, ..Default::default() }
    }

    fn body(c: &Chart, name: &str) -> f64 {
        c.bodies.iter().find(|b| b.name == name).unwrap().lon
    }

    /// Ascendant from the midheaven by the textbook formula, independent of the program's code.
    fn asc_of(mc: f64, lat: f64, eps: f64) -> f64 {
        let (m, e, p) = (mc.to_radians(), eps.to_radians(), lat.to_radians());
        let ra = (m.sin() * e.cos()).atan2(m.cos());
        ra.cos().atan2(-(ra.sin() * e.cos() + p.tan() * e.sin())).to_degrees().rem_euclid(360.0)
    }

    #[test]
    fn default_stays_astrolog() {
        let s = Settings::default();
        let a = progressed(NATAL, AGE_60, Some(PLACE), &s).unwrap();
        let b = cast(progressed_jd(NATAL, AGE_60), Some(PLACE), &s).unwrap();
        assert_eq!(a.cusps, b.cusps);
    }

    #[test]
    fn angles_follow_the_sun_and_planets_stay() {
        let n = cast(NATAL, Some(PLACE), &Settings::default()).unwrap();
        let d = progressed(NATAL, AGE_60, Some(PLACE), &Settings::default()).unwrap();
        let a = progressed(NATAL, AGE_60, Some(PLACE), &arc_settings()).unwrap();
        let arc = min_diff(body(&a, "Slunce"), body(&n, "Slunce"));
        assert!((arc - 59.0).abs() < 3.0, "oblouk {arc}");
        assert!(min_diff(body(&a, "Medium coeli"), norm360(body(&n, "Medium coeli") + arc)).abs() < 1e-6);
        let asc = asc_of(body(&a, "Medium coeli"), PLACE.0, 23.44);
        assert!(min_diff(body(&a, "Ascendent"), asc).abs() < 0.01, "{} {asc}", body(&a, "Ascendent"));
        for name in ["Slunce", "Měsíc", "Pluto"] {
            assert_eq!(body(&a, name), body(&d, name), "planety se nemění");
        }
        let c = a.cusps.unwrap();
        assert_eq!((c[0], c[9]), (body(&a, "Ascendent"), body(&a, "Medium coeli")));
        assert!(a.bodies.iter().find(|b| b.name == "Medium coeli").unwrap().speed.abs() < 1.2);
    }

    #[test]
    fn search_agrees_with_the_chart_and_finds_few_hits() {
        let n = cast(NATAL, Some(PLACE), &Settings::default()).unwrap();
        let from = NATAL + 60.0 * 365.24219;
        let s = arc_settings();
        let hits = find_hits(&n, PLACE, from, from + 3652.4, Mode::Progression, &s).unwrap();
        let axes: Vec<_> = hits.iter().filter(|h| h.mover_name == "Medium coeli").collect();
        assert!(!axes.is_empty() && axes.len() < 40, "zásahů {}", axes.len());
        let h = axes[0];
        let c = progressed(NATAL, h.jd, Some(PLACE), &s).unwrap();
        let aspect = astro_core::data::ASPECTS[h.aspect].angle;
        let natal_lon = n.bodies.iter().find(|b| b.index == h.natal).unwrap().lon;
        assert!((min_diff(body(&c, "Medium coeli"), natal_lon).abs() - aspect).abs() < 1e-3);
    }
}

#[test]
fn czech_names_decline() {
    use astro_core::names::{decline, looks_female};
    let cases = |w: &str, f| decline(w, f).join(" ");
    assert_eq!(cases("Jana", true), "Jana Jany Janě Janu Janě Janou");
    assert_eq!(cases("Věra", true), "Věra Věry Věře Věru Věře Věrou");
    assert_eq!(cases("Hanka", true), "Hanka Hanky Hance Hanku Hance Hankou");
    assert_eq!(cases("Olga", true), "Olga Olgy Olze Olgu Olze Olgou");
    assert_eq!(cases("Tereza", true), "Tereza Terezy Tereze Terezu Tereze Terezou");
    assert_eq!(cases("Marie", true), "Marie Marie Marii Marii Marii Marií");
    assert_eq!(cases("Karin", true), "Karin Karin Karin Karin Karin Karin");
    assert_eq!(cases("Petr", false), "Petr Petra Petrovi Petra Petrovi Petrem");
    assert_eq!(cases("Tomáš", false), "Tomáš Tomáše Tomášovi Tomáše Tomášovi Tomášem");
    assert_eq!(cases("Marek", false), "Marek Marka Markovi Marka Markovi Markem");
    assert_eq!(cases("Zdeněk", false), "Zdeněk Zdeňka Zdeňkovi Zdeňka Zdeňkovi Zdeňkem");
    assert_eq!(cases("Pavel", false), "Pavel Pavla Pavlovi Pavla Pavlovi Pavlem");
    assert_eq!(cases("Nataniel", false), "Nataniel Nataniela Natanielovi Nataniela Natanielovi Natanielem");
    assert_eq!(cases("Jiří", false), "Jiří Jiřího Jiřímu Jiřího Jiřím Jiřím");
    assert_eq!(cases("Honza", false), "Honza Honzy Honzovi Honzu Honzovi Honzou");
    assert!(looks_female("Jana") && looks_female("Máma") && looks_female("Marie") && looks_female("Karin"));
    assert!(!looks_female("Honza") && !looks_female("Táta") && !looks_female("Petr"));
}

#[test]
fn templates_fill_names_and_gender() {
    use astro_core::names::{fill, name_of};
    let (a, b) = (name_of("Petr Novák", ""), name_of("Jana Nováková", ""));
    assert_eq!(fill("Slunce {1g} svítí na {2a}, {2:on sám|ona sama} to cítí.", &a, &b), "Slunce Petra svítí na Janu, ona sama to cítí.");
    assert_eq!(fill("{p:Oba|Obě} {1:něj|ni}", &b, &name_of("Eva", "")), "Obě ni");
    assert_eq!(fill("{p:Oba|Obě}", &a, &b), "Oba");
    assert_eq!(fill("{x} zůstane", &a, &b), "{x} zůstane");
    let (v, s) = (name_of("Věra", ""), name_of("Simona", ""));
    assert_eq!(fill("v {1l} a s {2i}", &v, &s), "ve Věře a se Simonou");
    assert_eq!(fill("S {2i} a v {1l}", &v, &a), "S Petrem a ve Věře");
    assert_eq!(fill("k {2d}", &v, &name_of("Kateřina", "")), "ke Kateřině");
    assert_eq!(fill("v {1l}", &a, &b), "v Petrovi");
}

#[test]
fn every_relationship_template_fills() {
    use astro_core::names::{fill, name_of};
    let (a, b) = (name_of("Jana", ""), name_of("Věra", ""));
    for key in astro_core::texts::keys() {
        let text = astro_core::texts::get(key).unwrap();
        let out = fill(text, &a, &b);
        assert!(!out.contains('{') && !out.contains('}'), "{key}: {out}");
        if !key.starts_with("Synastrie") && !key.starts_with("Složený") {
            assert!(!text.contains('{'), "{key}");
        }
    }
}

/// Settings saved by version 1.0 have no font sizes; they load with the normal size.
#[test]
fn old_settings_load_with_normal_font_size() {
    let s: Settings = serde_json::from_str(r#"{"house_system":"P","chiron":true,"minor_aspects":false,"solar_arc_angles":false,"dark":true}"#).unwrap();
    assert_eq!((s.font_scale, s.print_scale), (100, 100));
    assert!(s.chiron && s.dark);
    assert_eq!(Settings::default().font_scale, 100);
}
