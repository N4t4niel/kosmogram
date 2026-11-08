use astro_core::ephem::*;
#[test]
fn sun_and_moon_match_astrolog() {
    let jd = julday(1985, 7, 14, 13.5);
    let (sun, moon) = (position(jd, 0).unwrap(), position(jd, 1).unwrap());
    assert!((sun.lon - (90.0 + 22.0 + 2.0 / 60.0)).abs() < 0.02);
    assert!((moon.lon - (60.0 + 11.0 + 40.0 / 60.0)).abs() < 0.02);
}

#[test]
fn dates_outside_the_data_are_refused_with_a_message() {
    assert!(position(julday(1790, 1, 1, 0.0), 0).unwrap_err().contains("1800"));
    assert!(position(julday(2410, 1, 1, 0.0), 2).unwrap_err().contains("1800"));
    assert!(position(julday(1799, 12, 31, 0.0), 15).is_ok());
    assert!(position(julday(2400, 1, 1, 0.0), 1).is_ok());
}

#[test]
fn julian_days_of_known_dates() {
    assert_eq!(julday(2000, 1, 1, 12.0), 2451545.0);
    assert_eq!(julday(1999, 1, 1, 0.0), 2451179.5);
    assert_eq!(julday(1987, 6, 19, 12.0), 2446966.0);
    assert_eq!(julday(1900, 1, 1, 0.0), 2415020.5);
}

/// Where a wheel label sits, as zodiac longitude, with the ascendant on the left.
fn label_longitude(svg: &str, text: &str, asc: f64) -> f64 {
    let end = svg.find(&format!(">{text}</text>")).expect("popisek chybí");
    let tag = &svg[svg[..end].rfind("<text").unwrap()..end];
    let number = |key: &str| -> f64 {
        let from = tag.find(key).unwrap() + key.len();
        tag[from..from + tag[from..].find('"').unwrap()].parse().unwrap()
    };
    let (x, y) = (number("x=\"") - 400.0, 400.0 - number("y=\""));
    (y.atan2(x).to_degrees() + asc + 180.0).rem_euclid(360.0)
}

#[test]
fn wheel_labels_stand_on_the_real_angles_in_every_house_system() {
    use astro_core::chart::cast;
    use astro_core::data::{ObjKind, OBJECTS};
    use astro_core::ephem::julday;
    use astro_core::model::Settings;
    use astro_core::wheel::render;
    for system in ['P', 'K', 'E', 'W', 'R', 'C', 'O', 'B'] {
        let s = Settings { house_system: system, ..Default::default() };
        let c = cast(julday(2026, 3, 20, 8.0), Some((48.15, 17.1167)), &s).unwrap();
        let lon = |kind: ObjKind| c.bodies.iter().find(|b| OBJECTS[b.index].kind == kind).unwrap().lon;
        let (asc, mc) = (lon(ObjKind::Asc), lon(ObjKind::Mc));
        let svg = render(&c, None, &[]);
        for (text, angle) in [("ASC", asc), ("MC", mc)] {
            let off = (label_longitude(&svg, text, asc) - angle + 540.0).rem_euclid(360.0) - 180.0;
            assert!(off.abs() < 6.0, "{system}: {text} je o {off}° mimo");
        }
    }
}

#[test]
fn angle_labels_keep_clear_of_the_planet_glyphs() {
    use astro_core::chart::cast;
    use astro_core::ephem::julday;
    use astro_core::model::Settings;
    use astro_core::wheel::render;
    let c = cast(julday(2150, 3, 20, 8.0), Some((48.15, 17.1167)), &Settings { house_system: 'W', ..Default::default() }).unwrap();
    let svg = render(&c, None, &[]);
    let center = |text: &str| {
        let end = svg.find(&format!(">{text}</text>")).unwrap();
        let tag = &svg[svg[..end].rfind("<text").unwrap()..end];
        let number = |key: &str| -> f64 {
            let from = tag.find(key).unwrap() + key.len();
            tag[from..from + tag[from..].find('"').unwrap()].parse().unwrap()
        };
        (number("x=\"") - 400.0).hypot(number("y=\"") - 400.0)
    };
    for text in ["ASC", "MC"] {
        assert!(center(text) > 330.0, "{text} je v pásu symbolů planet");
    }
}
