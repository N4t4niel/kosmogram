use crate::model::Person;
use crate::ephem::julday;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike};
use chrono_tz::Tz;

#[derive(Clone, Debug)]
pub struct Moment {
    pub jd_ut: f64,
    pub offset_min: i32,
    pub note: Option<String>,
}

pub fn parse_date(s: &str) -> Result<NaiveDate, String> {
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| format!("Neplatné datum: {s}"))?;
    if !(1800..=2399).contains(&d.year()) {
        return Err("Výpočty fungují pro roky 1800 až 2399.".into());
    }
    Ok(d)
}

pub fn parse_time(s: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(s, "%H:%M").map_err(|_| format!("Neplatný čas: {s}"))
}

fn jd_from_utc(dt: NaiveDateTime) -> f64 {
    let h = dt.hour() as f64 + dt.minute() as f64 / 60.0 + dt.second() as f64 / 3600.0;
    julday(dt.year(), dt.month(), dt.day(), h)
}

const TWICE: &str = "Tento čas nastal dvakrát (návrat z letního času). Použil jsem první z nich, případně ručně nastavte posun od UTC.";
const MISSING: &str = "Tento čas v daném dni neexistoval (přechod na letní čas). Použil jsem čas o hodinu pozdější.";
const MEAN_TIME: &str = "Pásmový čas tu tehdy ještě neplatil. Počítám s místním časem podle zeměpisné délky, případně ručně nastavte posun od UTC.";
const EARLY_STANDARD: &str = "Města přecházela na pásmový čas postupně. Pokud čas narození pochází z místních hodin, ověřte ho a případně ručně nastavte posun od UTC.";
const WARTIME: &str = "Letní čas se za války měnil a tyto údaje nemáme ověřené ze zákonů. Pokud máte jistotu o tehdejším čase, ručně nastavte posun od UTC.";
const SLOVAK_STATE: &str = "Pro Slovensko v letech 1939 až 1945 nemáme časové údaje ověřené ze zákonů. Pokud máte jistotu o tehdejším čase, ručně nastavte posun od UTC.";
const ABROAD: &str = "Historii času v této zemi před rokem 1970 jsme neověřovali. Pokud máte jistotu o tehdejším čase, ručně nastavte posun od UTC.";

/// Where the IANA history for Czechoslovakia differs from its law, the offset the law gave: UTC start
/// and end as (year, month, day, hour), then minutes east of UTC. The laws are 181/1946 and 54/1949 Zb.
/// for the wrong days, and 36/1979 with 31/1980 Zb. for clocks changed at midnight, not at two. The
/// same midnight changes in 1981 and 1982 come from kalendar.beda.cz, the source IANA itself names.
type Hour = (i32, u32, u32, u32);
const CZECHOSLOVAK_LAW: [(Hour, Hour, i32); 10] = [
    ((1946, 10, 6, 1), (1946, 10, 7, 1), 120),
    ((1949, 4, 9, 1), (1949, 4, 10, 1), 60),
    ((1979, 3, 31, 23), (1979, 4, 1, 1), 120),
    ((1979, 9, 29, 23), (1979, 9, 30, 1), 60),
    ((1980, 4, 5, 23), (1980, 4, 6, 1), 120),
    ((1980, 9, 27, 23), (1980, 9, 28, 1), 60),
    ((1981, 3, 28, 23), (1981, 3, 29, 1), 120),
    ((1981, 9, 26, 23), (1981, 9, 27, 1), 60),
    ((1982, 3, 27, 23), (1982, 3, 28, 1), 120),
    ((1982, 9, 25, 23), (1982, 9, 26, 1), 60),
];

fn at_hour((y, m, d, h): Hour) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(y, m, d).unwrap().and_hms_opt(h, 0, 0).unwrap()
}

fn is_czechoslovak(tz_name: &str) -> bool {
    matches!(tz_name, "Europe/Prague" | "Europe/Bratislava")
}

/// Minutes east of UTC at a moment, with the law taking precedence over the IANA history.
fn offset_at(tz: Tz, tz_name: &str, utc: NaiveDateTime) -> f64 {
    if is_czechoslovak(tz_name) {
        if let Some(law) = CZECHOSLOVAK_LAW.iter().find(|(from, to, _)| (at_hour(*from)..at_hour(*to)).contains(&utc)) {
            return law.2 as f64;
        }
    }
    tz.offset_from_utc_datetime(&utc).fix().local_minus_utc() as f64 / 60.0
}

/// Wall clock near a corrected change: UTC, offset and note worked out from `offset_at`.
fn around_law_change(tz: Tz, tz_name: &str, local: NaiveDateTime) -> Option<(NaiveDateTime, i32, Option<String>)> {
    let close = |h: &Hour| (local - at_hour(*h)).num_days().abs() < 3;
    if !is_czechoslovak(tz_name) || !CZECHOSLOVAK_LAW.iter().any(|(from, _, _)| close(from)) {
        return None;
    }
    let fits: Vec<i32> = [120, 60].into_iter().filter(|&o| offset_at(tz, tz_name, local - Duration::minutes(o as i64)) == o as f64).collect();
    Some(match fits[..] {
        [one] => (local - Duration::minutes(one as i64), one, None),
        [] => (local - Duration::minutes(60), 120, Some(MISSING.to_string())),
        _ => (local - Duration::minutes(120), 120, Some(TWICE.to_string())),
    })
}

/// Converts wall-clock time in `tz` to UT, using the IANA history (summer time included).
pub fn local_to_moment(local: NaiveDateTime, tz_name: &str, manual: Option<i32>) -> Result<Moment, String> {
    if let Some(off) = manual {
        return Ok(Moment { jd_ut: jd_from_utc(local - Duration::minutes(off as i64)), offset_min: off, note: None });
    }
    let tz: Tz = tz_name.parse().map_err(|_| format!("Neznámé časové pásmo: {tz_name}"))?;
    if let Some((utc, offset_min, note)) = around_law_change(tz, tz_name, local) {
        return Ok(Moment { jd_ut: jd_from_utc(utc), offset_min, note });
    }
    use chrono::LocalResult as L;
    let (dt, note) = match tz.from_local_datetime(&local) {
        L::Single(d) => (d, None),
        L::Ambiguous(a, _) => (a, Some(TWICE.to_string())),
        L::None => {
            let d = tz.from_local_datetime(&(local + Duration::hours(1))).earliest().ok_or("Tento čas v daném pásmu neexistuje.")?;
            (d, Some(MISSING.to_string()))
        }
    };
    let offset_min = dt.offset().fix().local_minus_utc() / 60;
    Ok(Moment { jd_ut: jd_from_utc(dt.naive_utc()), offset_min, note })
}

/// Before standard time the zone only holds the mean time of its main city, seconds and all.
fn before_standard_time(local: NaiveDateTime, tz_name: &str) -> bool {
    tz_name.parse::<Tz>().is_ok_and(|tz| tz.offset_from_utc_datetime(&local).fix().local_minus_utc() % 900 != 0)
}

/// A warning for periods where the history of the clock is not known for certain.
fn uncertain_period(local: NaiveDateTime, tz_name: &str) -> Option<&'static str> {
    let (year, day) = (local.year(), local.date());
    let on = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    if !is_czechoslovak(tz_name) {
        return (year < 1970).then_some(ABROAD);
    }
    if year <= 1899 {
        return Some(EARLY_STANDARD);
    }
    if tz_name == "Europe/Bratislava" && (1939..=1945).contains(&year) {
        return Some(SLOVAK_STATE);
    }
    (on(1940, 4, 1)..on(1945, 10, 2)).contains(&day).then_some(WARTIME)
}

pub fn person_moment(p: &Person) -> Result<Moment, String> {
    let date = parse_date(&p.date)?;
    let time = if p.time_known { parse_time(&p.time)? } else { NaiveTime::from_hms_opt(12, 0, 0).unwrap() };
    let local = date.and_time(time);
    if p.offset_min.is_none() && before_standard_time(local, &p.place.tz) {
        let seconds = (p.place.lon * 240.0).round() as i64;
        return Ok(Moment { jd_ut: jd_from_utc(local - Duration::seconds(seconds)), offset_min: (seconds / 60) as i32, note: Some(MEAN_TIME.to_string()) });
    }
    let mut m = local_to_moment(local, &p.place.tz, p.offset_min)?;
    if p.offset_min.is_none() && p.time_known {
        if let Some(warning) = uncertain_period(local, &p.place.tz) {
            m.note = Some(m.note.map_or(warning.to_string(), |n| format!("{n} {warning}")));
        }
    }
    Ok(m)
}

/// JD (UT) back to wall-clock time in the given zone.
pub fn jd_to_local(jd: f64, tz_name: &str) -> NaiveDateTime {
    let secs = ((jd - 2440587.5) * 86400.0).round() as i64;
    let utc = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default().naive_utc();
    match tz_name.parse::<Tz>() {
        Ok(tz) => utc + Duration::seconds((offset_at(tz, tz_name, utc) * 60.0) as i64),
        Err(_) => utc,
    }
}

pub fn offset_label(min: i32) -> String {
    let sign = if min < 0 { '-' } else { '+' };
    let a = min.abs();
    if a % 60 == 0 { format!("UTC{sign}{}", a / 60) } else { format!("UTC{sign}{}:{:02}", a / 60, a % 60) }
}
