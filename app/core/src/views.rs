//! Ready-to-display results for the UI.
use crate::aspects::{between, within, Aspect};
use crate::chart::*;
use crate::data::*;
use crate::ephem;
use crate::interpret::{self, Kind, Section};
use crate::model::{Person, Place, Settings};
use crate::names;
use crate::search::{find_hits, Hit, Mode};
use crate::timeconv::{jd_to_local, local_to_moment, offset_label, parse_date, parse_time, Moment};
use crate::wheel::{render, Line};
use chrono::{Datelike, NaiveDate};
use serde::Serialize;

/// Orb limit for "what is active now" lists: transits fade quickly beyond it.
/// Transit dates and times are read and shown in the user's own zone, not the birth place's.
const APP_TZ: &str = "Europe/Prague";
const ACTIVE_TRANSIT_ORB: f64 = 2.0;
const ACTIVE_PROGRESSION_ORB: f64 = 1.0;
/// Astrolog -YjT: how long-lasting, and so how important, each transiting body is.
const TRANSIT_WEIGHT: [f64; 14] = [10.0, 4.0, 8.0, 9.0, 20.0, 30.0, 35.0, 40.0, 45.0, 50.0, 30.0, 30.0, 0.0, 0.0];

#[derive(Serialize)]
pub struct Row {
    pub glyph: String,
    pub name: &'static str,
    pub position: String,
    pub house: Option<usize>,
    pub retro: bool,
    pub dignity: &'static str,
}

#[derive(Serialize)]
pub struct Cusp {
    pub house: usize,
    pub position: String,
}

#[derive(Serialize)]
pub struct ChartPanel {
    pub title: String,
    pub svg: String,
    pub rows: Vec<Row>,
    pub cusps: Vec<Cusp>,
    pub aspects: Vec<Aspect>,
    pub sections: Vec<Section>,
    pub info: Vec<String>,
}

#[derive(Serialize)]
pub struct Event {
    pub date: String,
    pub time: String,
    pub title: String,
    pub text: String,
    pub hit: Hit,
}

fn dignity_name(d: Option<char>) -> &'static str {
    match d {
        Some('R') => "vládce",
        Some('d') => "vyhnanství",
        Some('X') => "povýšení",
        Some('f') => "pád",
        _ => "",
    }
}

/// Planets of `guest` placed in the houses of `host`; the guest's own angles do not apply there.
fn rows_in_houses_of(guest: &Chart, host: &Chart) -> Vec<Row> {
    let bodies = guest
        .bodies
        .iter()
        .filter(|b| !matches!(OBJECTS[b.index].kind, ObjKind::Asc | ObjKind::Mc))
        .map(|b| make_body(b.index, b.lon, b.lat, b.speed, host.cusps.as_ref()))
        .collect();
    rows(&Chart { bodies, cusps: host.cusps, jd_ut: guest.jd_ut })
}

fn rows(c: &Chart) -> Vec<Row> {
    c.bodies
        .iter()
        .map(|b| Row { glyph: b.glyph.clone(), name: b.name, position: b.position.clone(), house: b.house, retro: b.retro, dignity: dignity_name(b.dignity) })
        .collect()
}

fn cusps(c: &Chart) -> Vec<Cusp> {
    c.cusps.iter().flatten().enumerate().map(|(i, &l)| Cusp { house: i + 1, position: format_pos(l) }).collect()
}

fn line_list(list: &[Aspect], a: &Chart, b: &Chart) -> Vec<Line> {
    list.iter()
        .filter_map(|x| {
            let from = a.bodies.iter().find(|p| p.index == x.a)?.lon;
            let to = b.bodies.iter().find(|p| p.index == x.b)?.lon;
            Some(Line { from, to, color: x.color, weight: (2.2 * (1.0 - x.orb / x.max_orb)).max(0.5) })
        })
        .collect()
}

fn birth_info(p: &Person, m: &Moment) -> Vec<String> {
    let mut v = vec![format!("{}, {}", p.place.name, p.place.country)];
    let time = if p.time_known { format!("{} ({})", p.time, offset_label(m.offset_min)) } else { "čas neznámý, počítáno pro poledne".into() };
    v.push(format!("{}, {time}", cz_date(&p.date)));
    v.push(format!("{:.4}° s. š., {:.4}° v. d.", p.place.lat, p.place.lon));
    if let Some(n) = &m.note {
        v.push(n.clone());
    }
    if !p.time_known {
        v.push("Bez času narození nejsou k dispozici domy, Ascendent ani MC a poloha Měsíce může být až o 7° jiná.".into());
    }
    v
}

fn cz_date(iso: &str) -> String {
    parse_date(iso).map(|d| format!("{}. {}. {}", d.day(), d.month(), d.year())).unwrap_or_else(|_| iso.into())
}

pub fn radix(p: &Person, s: &Settings) -> Result<ChartPanel, String> {
    let (c, m) = natal(p, s)?;
    let asp = within(&c, s);
    let svg = render(&c, None, &line_list(&asp, &c, &c));
    let sections = vec![interpret::overview(&c), interpret::placements(&c), interpret::houses(&c), interpret::natal_aspects(&asp)];
    Ok(ChartPanel { title: format!("Horoskop: {}", p.name), svg, rows: rows(&c), cusps: cusps(&c), aspects: asp, sections, info: birth_info(p, &m) })
}

/// The sky for a wall-clock moment at a place, with no person attached.
pub fn sky(place: &Place, date: &str, time: &str, s: &Settings) -> Result<ChartPanel, String> {
    let p = Person { name: "Obloha".into(), date: date.into(), time: time.into(), time_known: true, place: place.clone(), ..Person::default() };
    let (c, m) = natal(&p, s)?;
    let asp = within(&c, s);
    let mut info = vec![format!("{}, {}", place.name, place.country), format!("{}, {} ({})", cz_date(date), time, offset_label(m.offset_min))];
    info.extend(m.note);
    Ok(ChartPanel { title: "Obloha".into(), svg: render(&c, None, &line_list(&asp, &c, &c)), rows: rows(&c), cusps: cusps(&c), aspects: asp, sections: vec![], info })
}

fn moment_at(date: &str, time: &str) -> Result<Moment, String> {
    local_to_moment(parse_date(date)?.and_time(parse_time(time)?), APP_TZ, None)
}

/// Transits for one moment, with `date`/`time` read in `APP_TZ`.
pub fn transits(p: &Person, date: &str, time: &str, s: &Settings) -> Result<ChartPanel, String> {
    let (n, _) = natal(p, s)?;
    let m = moment_at(date, time)?;
    let t = cast(m.jd_ut, None, s)?;
    let mut active: Vec<Aspect> = between(&t, &n, s, false).into_iter().filter(|a| a.orb <= ACTIVE_TRANSIT_ORB && a.a != MOON).collect();
    for a in &mut active {
        a.power = ASPECTS[a.aspect_index].influence * TRANSIT_WEIGHT[a.a] * (1.0 - a.orb / a.max_orb);
    }
    active.sort_by(|x, y| y.power.total_cmp(&x.power));
    let svg = render(&n, Some(&t), &line_list(&active, &t, &n));
    let sections = vec![interpret::active_moving(&active, "Co teď působí", interpret::TRANSIT)];
    let info = vec![format!("Tranzity k {}, {time}", cz_date(date)), "Uvnitř radix, vně planety pro zvolený den.".into()];
    Ok(ChartPanel { title: format!("Tranzity: {}", p.name), svg, rows: rows(&t), cusps: vec![], aspects: active, sections, info })
}

/// Secondary progressions for the given date.
pub fn progressions(p: &Person, date: &str, s: &Settings) -> Result<ChartPanel, String> {
    let (n, _) = natal(p, s)?;
    let target = moment_at(date, "12:00")?;
    let place = p.time_known.then_some((p.place.lat, p.place.lon));
    let c = progressed(n.jd_ut, target.jd_ut, place, s)?;
    let active: Vec<Aspect> = between(&c, &n, s, false).into_iter().filter(|a| a.orb <= ACTIVE_PROGRESSION_ORB).collect();
    let svg = render(&n, Some(&c), &line_list(&active, &c, &n));
    let sections = vec![interpret::active_moving(&active, "Progrese k radixu (orb do 1°)", interpret::progression_family(s))];
    let axes = if s.solar_arc_angles {
        "Osy a domy: solární oblouk, asi 1° za rok."
    } else {
        "Osy a domy jako v Astrologu: za rok oběhnou celý kruh, zásah os trvá jen několik dní."
    };
    let info = vec![format!("Progrese k {}", cz_date(date)), "Sekundární progrese: jeden den po narození odpovídá jednomu roku života.".into(), axes.into()];
    Ok(ChartPanel { title: format!("Progrese: {}", p.name), svg, rows: rows(&c), cusps: cusps(&c), aspects: active, sections, info })
}

/// The moment the Sun comes back to `sun` near `jd`, in UT.
fn sun_returns(sun: f64, mut jd: f64) -> Result<f64, String> {
    const DEGREES_A_DAY: f64 = 0.9856;
    for _ in 0..12 {
        let left = min_diff(sun, ephem::longitude(jd, 0)?);
        jd += left / DEGREES_A_DAY;
        if left.abs() < 1e-7 {
            break;
        }
    }
    Ok(jd)
}

/// The chart of the moment the Sun returns to its birth position in `year`, for a place of choice.
pub fn solar_return(p: &Person, year: i32, place: Option<&Place>, s: &Settings) -> Result<ChartPanel, String> {
    let (n, _) = natal(p, s)?;
    let born = parse_date(&p.date)?.year();
    if year < born {
        return Err("Solární návrat je možný až od roku narození.".into());
    }
    let sun = n.bodies.iter().find(|b| b.index == 0).ok_or("Chybí poloha Slunce.")?.lon;
    let jd = sun_returns(sun, n.jd_ut + (year - born) as f64 * DAYS_IN_YEAR)?;
    let at = place.unwrap_or(&p.place);
    let c = cast(jd, p.time_known.then_some((at.lat, at.lon)), s)?;
    let asp = within(&c, s);
    let (local, utc) = (jd_to_local(jd, &at.tz), jd_to_local(jd, "UTC"));
    let when = format!("{}. {}. {}, {}", local.day(), local.month(), local.year(), local.format("%H:%M"));
    let mut info = vec![format!("Slunce se vrací na svou polohu z narození: {when} ({}), {}", offset_label((local - utc).num_minutes() as i32), at.name)];
    if !p.time_known {
        info.push("Bez času narození nejde určit přesný okamžik návratu, proto chybí domy, Ascendent a MC.".into());
    }
    let sections = vec![interpret::placements_of(&c, Kind::Solar), interpret::aspects_of(&asp, Kind::Solar)];
    Ok(ChartPanel { title: format!("Solární návrat {year}: {}", p.name), svg: render(&c, None, &line_list(&asp, &c, &c)), rows: rows(&c), cusps: cusps(&c), aspects: asp, sections, info })
}

fn jd_of_midnight(d: NaiveDate) -> Result<f64, String> {
    Ok(local_to_moment(d.and_hms_opt(0, 0, 0).unwrap(), APP_TZ, None)?.jd_ut)
}

/// Exact events; `month` 0 searches the whole year.
pub fn events(p: &Person, year: i32, month: u32, progression: bool, s: &Settings) -> Result<Vec<Event>, String> {
    let (n, _) = natal(p, s)?;
    let (first, next) = if month == 0 {
        (NaiveDate::from_ymd_opt(year, 1, 1), NaiveDate::from_ymd_opt(year + 1, 1, 1))
    } else if month == 12 {
        (NaiveDate::from_ymd_opt(year, 12, 1), NaiveDate::from_ymd_opt(year + 1, 1, 1))
    } else {
        (NaiveDate::from_ymd_opt(year, month, 1), NaiveDate::from_ymd_opt(year, month + 1, 1))
    };
    let (first, next) = (first.ok_or("Neplatné období")?, next.ok_or("Neplatné období")?);
    let mode = if progression { Mode::Progression } else { Mode::Transit };
    let hits = find_hits(&n, (p.place.lat, p.place.lon), jd_of_midnight(first)?, jd_of_midnight(next)?, mode, s)?;
    Ok(hits
        .into_iter()
        .map(|h| {
            let local = jd_to_local(h.jd, APP_TZ);
            let r = if h.retro { " (zpětně)" } else { "" };
            Event {
                date: format!("{}. {}. {}", local.day(), local.month(), local.year()),
                time: local.format("%H:%M").to_string(),
                title: format!("{} {}{r}: {} – {}", h.mover_name, h.sign, h.aspect_name.to_lowercase(), h.natal_name),
                text: interpret::transit_text(&h, if progression { interpret::progression_family(s) } else { interpret::TRANSIT }),
                hit: h,
            }
        })
        .collect())
}

#[derive(Serialize)]
pub struct Relation {
    pub synastry: ChartPanel,
    pub composite: ChartPanel,
}

pub fn relation(a: &Person, b: &Person, s: &Settings) -> Result<Relation, String> {
    let ((ca, ma), (cb, mb)) = (natal(a, s)?, natal(b, s)?);
    let asp = between(&ca, &cb, s, true);
    let (na, nb) = names::pair(a, b);
    let mut sections = vec![interpret::relation_aspects(&asp, &na, &nb)];
    if ca.cusps.is_some() {
        sections.push(interpret::overlay(&cb, &ca, &nb, &na));
    }
    if cb.cusps.is_some() {
        sections.push(interpret::overlay(&ca, &cb, &na, &nb));
    }
    let mut info = vec![format!("Tabulka ukazuje planety {} v domech {}.", nb.gen(), na.gen())];
    info.extend(birth_info(a, &ma));
    info.extend(birth_info(b, &mb));
    let synastry = ChartPanel {
        title: format!("Synastrie: {} a {}", a.name, b.name),
        svg: render(&ca, Some(&cb), &line_list(&asp, &ca, &cb)),
        rows: rows_in_houses_of(&cb, &ca),
        cusps: cusps(&ca),
        aspects: asp,
        sections,
        info,
    };
    let comp = composite(&ca, &cb);
    let comp_asp = within(&comp, s);
    let mut comp_sections = vec![interpret::placements_of(&comp, Kind::Composite), interpret::aspects_of(&comp_asp, Kind::Composite)];
    for item in comp_sections.iter_mut().flat_map(|sec| sec.items.iter_mut()) {
        item.text = names::fill(&item.text, &na, &nb);
    }
    let composite_panel = ChartPanel {
        title: format!("Složený horoskop: {} a {}", a.name, b.name),
        svg: render(&comp, None, &line_list(&comp_asp, &comp, &comp)),
        rows: rows(&comp),
        cusps: cusps(&comp),
        sections: comp_sections,
        aspects: comp_asp,
        info: vec!["Složený horoskop tvoří středy mezi polohami obou lidí a popisuje vztah samotný.".into()],
    };
    Ok(Relation { synastry, composite: composite_panel })
}
