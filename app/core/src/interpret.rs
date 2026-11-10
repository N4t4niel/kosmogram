//! Czech interpretation texts, rewritten from Astrolog's intrpret.cpp and data.cpp tables.
//! Astrolog glues English fragments together; Czech grammar would break that, so subjects are
//! always plural noun pairs and signs are adverbs, which avoids gender agreement entirely.
use crate::aspects::Aspect;
use crate::chart::{Body, Chart};
use crate::data::*;
use crate::model::Settings;
use crate::names::{self, Name};
use crate::search::Hit;
use crate::texts;
use serde::Serialize;

/// (nominative, instrumental) per object in `OBJECTS` order.
pub const PARTS: [(&str, &str); 14] = [
    ("Vůle a vědomí sebe sama", "vůlí a vědomím sebe sama"),
    ("City a nálady", "city a náladami"),
    ("Myšlení a komunikace", "myšlením a komunikací"),
    ("Láska a smysl pro krásu", "láskou a smyslem pro krásu"),
    ("Energie a průbojnost", "energií a průbojností"),
    ("Optimismus a touha po růstu", "optimismem a touhou po růstu"),
    ("Disciplína a smysl pro odpovědnost", "disciplínou a smyslem pro odpovědnost"),
    ("Originalita a touha po svobodě", "originalitou a touhou po svobodě"),
    ("Intuice a duchovní citlivost", "intuicí a duchovní citlivostí"),
    ("Vnitřní síla a schopnost proměny", "vnitřní silou a schopností proměny"),
    ("Staré rány a schopnost léčit", "starými ranami a schopností léčit"),
    ("Životní směr a karmické úkoly", "životním směrem a karmickými úkoly"),
    ("Vnější projev a první dojem", "vnějším projevem a prvním dojmem"),
    ("Pověst a společenské postavení", "pověstí a společenským postavením"),
];

pub const SIGN_STYLE: [&str; 12] = [
    "rázně, energicky, přímo a odvážně",
    "prakticky, často skepticky a tvrdohlavě",
    "zvídavě, vtipně, bystře a přizpůsobivě",
    "hloubavě, citově a ochranitelsky",
    "hrdě, společensky, dramaticky a důstojně",
    "analyticky, kriticky, skromně a ochotně",
    "laskavě, tolerantně, často váhavě a se smyslem pro krásu",
    "pronikavě, podezíravě a hloubavě",
    "žoviálně, otevřeně a svobodomyslně",
    "pilně, prakticky a disciplinovaně",
    "pokrokově, nevyzpytatelně, idealisticky a vynalézavě",
    "obrazotvorně, nepozemsky a vnímavě",
];

pub const SIGN_DESIRE: [&str; 12] = [
    "hledají dobrodružství a výzvy",
    "touží po klidu a vnitřním míru",
    "vyhledávají informace",
    "chtějí vnitřní i vnější bezpečí",
    "touží po sebevyjádření",
    "usilují o dokonalost",
    "hledají rovnováhu, vztah a klid",
    "touží po proměně a odložení vnějších masek",
    "hledají smysl a nové zkušenosti",
    "usilují o soustředění a osobní integritu",
    "touží po jedinečnosti",
    "hledají přesah sebe sama",
];

/// Genitive, follows "v oblasti".
pub const HOUSE_AREA: [&str; 12] = [
    "budování vlastní identity",
    "sebehodnocení a hmotného zabezpečení",
    "komunikace s okolím",
    "představivosti, vnitřních pocitů a domácího života",
    "radosti, potěšení a tvořivého vyjádření",
    "práce a pocitu vlastní užitečnosti",
    "osobních vztahů a blízkosti",
    "sexuality, smrti, okultna a dalších těžko uchopitelných témat",
    "změn, objevování a vybočení ze stereotypu",
    "kariéry, společenského postavení a osudu",
    "budoucnosti, životních cílů, přátel a skupin",
    "toho, co osobnost narušuje nebo od ní odvádí",
];

/// Verb with `{}` for the strength word, per major aspect, then the wording for each strength.
pub const ASPECT_VERB: [(&str, [&str; 3]); 5] = [
    ("{}splývají s", ["vždy ", "", "částečně "]),
    ("se {}střetávají a jsou v napětí s", ["vždy ", "", "občas "]),
    ("jsou {}v rozporu s", ["nesmiřitelně ", "", "poněkud "]),
    ("jsou {}v souladu s", ["vždy ", "", "do jisté míry "]),
    ("mají {}příležitostí k růstu a výměně ve vztahu s", ["hodně ", "", "několik "]),
];

pub const ASPECT_MINOR: &str = "jsou v jemném, méně výrazném vztahu s";

pub const ASPECT_NOTE: [&str; 5] = [
    "Obě části mají v osobnosti výrazné místo.",
    "Je třeba hledat rovnováhu.",
    "Přizpůsobit se musejí obě strany.",
    "",
    "",
];

/// What a reading item points at in the wheel, so the UI can highlight it.
/// `outer` bodies are in the outer ring of a double wheel; `line` is an aspect (a, b) as drawn.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Mark {
    pub inner: Vec<usize>,
    pub outer: Vec<usize>,
    pub house: Option<usize>,
    pub line: Option<(usize, usize)>,
}

impl Mark {
    pub fn body(i: usize, house: Option<usize>) -> Self {
        Mark { inner: vec![i], house, ..Default::default() }
    }
    pub fn aspect(a: usize, b: usize, a_outer: bool, b_outer: bool) -> Self {
        let mut m = Mark { line: Some((a, b)), ..Default::default() };
        if a_outer { m.outer.push(a) } else { m.inner.push(a) }
        if b_outer { m.outer.push(b) } else { m.inner.push(b) }
        m
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub title: String,
    pub text: String,
    pub mark: Mark,
}

#[derive(Clone, Debug, Serialize)]
pub struct Section {
    pub title: String,
    pub items: Vec<Item>,
}

fn verb(asp: usize, strength: usize) -> String {
    match ASPECT_VERB.get(asp) {
        Some((v, words)) => v.replace("{}", words[strength]),
        None => ASPECT_MINOR.to_string(),
    }
}

/// Astrolog: every 2.5 degrees of orb weakens the wording (nOrb).
fn strength(orb: f64) -> usize {
    ((orb / 2.5) as usize).min(2)
}

fn has_house(b: &Body) -> bool {
    !matches!(OBJECTS[b.index].kind, ObjKind::Asc | ObjKind::Mc)
}

fn sign_sentence(b: &Body) -> String {
    let very = if (b.lon % 30.0) < 10.0 { "velmi " } else { "" };
    format!("{} se projevují {very}{}; {}.", PARTS[b.index].0, SIGN_STYLE[b.sign], SIGN_DESIRE[b.sign])
}

fn dignity_sentence(b: &Body) -> &'static str {
    match b.dignity {
        Some('R') => " Je to výrazná část osobnosti.",
        Some('d') => " V osobnosti to hraje spíše menší roli.",
        Some('X') => " Tuto stránku lze projevit snadno.",
        Some('f') => " Tuto stránku je těžší projevit.",
        _ => "",
    }
}

fn retro_note(b: &Body) -> &'static str {
    if b.retro && b.index != OBJECTS.iter().position(|o| o.id == "node").unwrap() {
        "nezávisle, spíše dovnitř a uzavřeně, a to "
    } else {
        ""
    }
}

fn placement_title(b: &Body) -> String {
    let house = match (b.house, has_house(b)) {
        (Some(h), true) => format!(", {h}. dům"),
        _ => String::new(),
    };
    let r = if b.retro { " (zpětný pohyb)" } else { "" };
    format!("{} {}{house}{r}", b.name, SIGNS_LOC[b.sign])
}

/// Which family of hand-written texts a chart uses. Radix texts speak about a birth,
/// so solar returns and composites have their own, and fall back to composed sentences.
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Radix,
    Solar,
    Lunar,
    Composite,
}

impl Kind {
    fn prefix(self) -> &'static str {
        match self {
            Kind::Radix => "",
            Kind::Solar => "Solární | ",
            Kind::Lunar => "Lunární | ",
            Kind::Composite => "Složený | ",
        }
    }

    /// Returns show only their own texts; a sentence composed for a birth would not fit a year or a month.
    fn own_texts_only(self) -> bool {
        matches!(self, Kind::Solar | Kind::Lunar)
    }
}

pub fn placements(c: &Chart) -> Section {
    placement_section(c, Kind::Radix)
}

pub fn placements_of(c: &Chart, kind: Kind) -> Section {
    placement_section(c, kind)
}

fn placement_section(c: &Chart, kind: Kind) -> Section {
    let p = kind.prefix();
    let items = c
        .bodies
        .iter()
        .map(|b| {
            let own_sign = texts::sign(&format!("{p}{}", b.name), b.sign_name);
            let mut text = own_sign.map(str::to_string).unwrap_or_else(|| if kind.own_texts_only() { String::new() } else { sign_sentence(b) });
            if let (Some(h), true) = (b.house, has_house(b)) {
                match texts::house(&format!("{p}{}", b.name), h) {
                    Some(t) => text = format!("{text} {t}").trim_start().to_string(),
                    None if !kind.own_texts_only() => text += &format!(" Nejčastěji se to projeví {}v oblasti {}.", retro_note(b), HOUSE_AREA[h - 1]),
                    None => {}
                }
            }
            if own_sign.is_none() && !kind.own_texts_only() {
                text += dignity_sentence(b);
            }
            if let (true, Kind::Radix, Some(t)) = (b.retro, kind, texts::get(&format!("{} | Zpětný pohyb", b.name))) {
                text += &format!(" {t}");
            }
            Item { title: placement_title(b), text, mark: Mark::body(b.index, b.house.filter(|_| has_house(b))) }
        })
        // A return without its own text for a body leaves it out rather than showing a bare title.
        .filter(|i| !(kind.own_texts_only() && i.text.trim().is_empty()))
        .collect();
    Section { title: "Planety ve znameních a domech".into(), items }
}

/// Relative to the aspect's own orb, because minor aspects allow only a few degrees.
fn exactness(a: &Aspect) -> &'static str {
    match a.orb / a.max_orb {
        r if r < 1.0 / 3.0 => " Aspekt je těsný, téma působí velmi výrazně.",
        r if r > 2.0 / 3.0 => " Aspekt je volný, téma působí spíše v pozadí.",
        _ => "",
    }
}

/// Czech preposition "s" becomes "se" before s, š, z, ž.
fn with(verb: &str, tail: &str) -> String {
    let needs_e = tail.chars().next().is_some_and(|c| "sšzž".contains(c));
    match verb.strip_suffix(" s") {
        Some(stem) if needs_e => format!("{stem} se {tail}"),
        _ => format!("{verb} {tail}"),
    }
}

fn aspect_title(a: &Aspect) -> String {
    format!("{} – {} – {} (orb {:.1}°)", a.a_name, a.aspect_name.to_lowercase(), a.b_name, a.orb)
}

fn note_of(asp: usize) -> String {
    match ASPECT_NOTE.get(asp) {
        Some(n) if !n.is_empty() => format!(" {n}"),
        _ => String::new(),
    }
}

pub fn natal_aspects(list: &[Aspect]) -> Section {
    aspect_section(list, Kind::Radix)
}

pub fn aspects_of(list: &[Aspect], kind: Kind) -> Section {
    aspect_section(list, kind)
}

fn aspect_section(list: &[Aspect], kind: Kind) -> Section {
    let own = |a: &&Aspect| texts::aspect(&format!("{}{}", kind.prefix(), a.a_name), a.b_name, a.aspect_name).is_some();
    let items = list
        .iter()
        .filter(|a| !kind.own_texts_only() || own(a))
        .map(|a| Item {
            title: aspect_title(a),
            mark: Mark::aspect(a.a, a.b, false, false),
            text: match texts::aspect(&format!("{}{}", kind.prefix(), a.a_name), a.b_name, a.aspect_name) {
                Some(t) => format!("{t}{}", exactness(a)),
                None => format!("{} {}.{}", PARTS[a.a].0, with(&verb(a.aspect_index, strength(a.orb)), PARTS[a.b].1), note_of(a.aspect_index)),
            },
        })
        .collect();
    Section { title: "Aspekty".into(), items }
}

/// One line about a moving body touching a natal point; shared by transits and progressions.
fn moving_text(family: &str, mover: usize, asp: usize, natal: usize, adjective: &str) -> String {
    let axis = matches!(OBJECTS[mover].kind, ObjKind::Asc | ObjKind::Mc);
    let family = if family == PROGRESSION_FAST_AXES && !axis { PROGRESSION } else { family };
    if let Some(t) = texts::aspect(&format!("{family} | {}", OBJECTS[mover].name), OBJECTS[natal].name, ASPECTS[asp].name) {
        return t.to_string();
    }
    let tail = if mover == natal { format!("stejnou oblastí v {adjective}") } else { format!("{} v {adjective}", PARTS[natal].1) };
    let note = if asp == 0 { " Tato část osobnosti bude silně ovlivněna.".to_string() } else { note_of(asp) };
    format!("{} {}.{}", PARTS[mover].0, with(&verb(asp, 1), &tail), note)
}

pub const TRANSIT: &str = "Tranzit";
pub const PROGRESSION: &str = "Progrese";
/// Astrolog's axes circle the chart once a year, so their contacts last days and have their own texts.
/// Planets move the same under both methods and read the plain progression texts.
const PROGRESSION_FAST_AXES: &str = "Progrese denní";

pub fn progression_family(s: &Settings) -> &'static str {
    if s.solar_arc_angles { PROGRESSION } else { PROGRESSION_FAST_AXES }
}

pub fn transit_text(h: &Hit, family: &str) -> String {
    moving_text(family, h.mover, h.aspect, h.natal, "radixu")
}

pub fn active_moving(list: &[Aspect], title: &str, family: &str) -> Section {
    let items = list
        .iter()
        .map(|a| Item { title: aspect_title(a), text: moving_text(family, a.a, a.aspect_index, a.b, "radixu"), mark: Mark::aspect(a.a, a.b, true, false) })
        .collect();
    Section { title: title.into(), items }
}

/// How the planets of `who` fall into the houses of `host`.
/// `drawn` says the guest is the outer ring of the wheel and the host's houses are the ones drawn.
pub fn overlay(guest: &Chart, host: &Chart, owner: &Name, host_person: &Name, drawn: bool) -> Section {
    let (who, host_name) = (owner.nom(), host_person.nom());
    let items = guest
        .bodies
        .iter()
        .filter(|b| has_house(b))
        .filter_map(|b| {
            let h = crate::chart::house_of(b.lon, host.cusps.as_ref()?);
            let mark = if drawn { Mark { outer: vec![b.index], house: Some(h), ..Default::default() } } else { Mark::body(b.index, None) };
            if let Some(t) = texts::house(&format!("Synastrie dům | {}", b.name), h) {
                return Some(Item { title: format!("{} ({who}) {}, {h}. dům ({host_name})", b.name, SIGNS_LOC[b.sign]), text: names::fill(t, owner, host_person), mark });
            }
            let mut text = format!("{who}: {}", lower_first(&sign_sentence(b)));
            text += &format!(" {host_name} to pocítí v oblasti {}.", HOUSE_AREA[h - 1]);
            match b.dignity {
                Some('X') => text += " Působí to harmonicky.",
                Some('f') => text += " Působí to spíše nesouladně.",
                _ => {}
            }
            Some(Item { title: format!("{} ({who}) {}, {h}. dům ({host_name})", b.name, SIGNS_LOC[b.sign]), text, mark })
        })
        .collect();
    Section { title: format!("Planety {} v domech {}", owner.gen(), host_person.gen()), items }
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().chain(c).collect(),
        None => String::new(),
    }
}

/// A synastry pair reads the same both ways, so its key keeps the bodies in table order.
fn relation_text(x: &Aspect, a: &Name, b: &Name) -> String {
    let (first, second, one, other) = if x.a <= x.b { (x.a_name, x.b_name, a, b) } else { (x.b_name, x.a_name, b, a) };
    match texts::aspect(&format!("Synastrie | {first}"), second, x.aspect_name) {
        Some(t) => format!("{}{}", names::fill(t, one, other), exactness(x)),
        None => format!("{} ({}) {} ({}).{}", PARTS[x.a].0, a.nom(), with(&verb(x.aspect_index, strength(x.orb)), PARTS[x.b].1), b.nom(), note_of(x.aspect_index)),
    }
}

pub fn relation_aspects(list: &[Aspect], a: &Name, b: &Name) -> Section {
    let items = list
        .iter()
        .map(|x| Item {
            title: format!("{} ({}) – {} – {} ({}), orb {:.1}°", x.a_name, a.nom(), x.aspect_name.to_lowercase(), x.b_name, b.nom(), x.orb),
            text: relation_text(x, a, b),
            mark: Mark::aspect(x.a, x.b, false, true),
        })
        .collect();
    Section { title: "Aspekty mezi horoskopy".into(), items }
}

const PHASES: [&str; 8] = ["Novoluní", "Dorůstající srpek", "První čtvrť", "Dorůstající Měsíc", "Úplněk", "Couvající Měsíc", "Poslední čtvrť", "Couvající srpek"];
const ELEMENTS: [&str; 4] = ["Oheň", "Země", "Vzduch", "Voda"];
const QUALITIES: [&str; 3] = ["Kardinální", "Pevná", "Proměnlivá"];
/// Modern ruler of each sign, indices into `OBJECTS`.
const SIGN_RULER: [usize; 12] = [4, 3, 2, 1, 0, 2, 3, 9, 5, 6, 7, 8];
const PLANETS: std::ops::Range<usize> = 0..10;
const STRONG: usize = 5;
const WEAK: usize = 1;
const HEMISPHERE_MIN: usize = 7;
const PATTERN_ORB: f64 = 6.0;
const FINE_ORB: f64 = 3.0;

fn item(title: String, key: &str) -> Option<Item> {
    marked(title, key, Mark::default())
}

fn marked(title: String, key: &str, mark: Mark) -> Option<Item> {
    texts::get(key).map(|t| Item { title, text: t.to_string(), mark })
}

fn planets(c: &Chart) -> Vec<&Body> {
    c.bodies.iter().filter(|b| PLANETS.contains(&b.index)).collect()
}

fn balance(c: &Chart, out: &mut Vec<Item>) {
    let list = planets(c);
    let count = |f: &dyn Fn(usize) -> bool| list.iter().filter(|b| f(b.sign)).count();
    for (i, e) in ELEMENTS.iter().enumerate() {
        let n = count(&|s| s % 4 == i);
        if n >= STRONG {
            out.extend(item(format!("Převaha živlu: {e}"), &format!("Živel | {e} | převaha")));
        } else if n <= WEAK {
            out.extend(item(format!("Slabý živel: {e}"), &format!("Živel | {e} | nedostatek")));
        }
    }
    for (i, q) in QUALITIES.iter().enumerate() {
        if count(&|s| s % 3 == i) >= STRONG {
            out.extend(item(format!("Převaha kvality: {q}"), &format!("Kvalita | {q} | převaha")));
        }
    }
}

fn hemispheres(c: &Chart, out: &mut Vec<Item>) {
    let houses: Vec<usize> = planets(c).iter().filter_map(|b| b.house).collect();
    let sides: [(&str, &dyn Fn(usize) -> bool); 4] = [
        ("Východ", &|h| h >= 10 || h <= 3),
        ("Západ", &|h| (4..=9).contains(&h)),
        ("Sever", &|h| h <= 6),
        ("Jih", &|h| h >= 7),
    ];
    for (name, inside) in sides {
        if houses.iter().filter(|&&h| inside(h)).count() >= HEMISPHERE_MIN {
            out.extend(item(format!("Převaha polokoule: {name}"), &format!("Polokoule | {name} | převaha")));
        }
    }
}

fn at(a: &Body, b: &Body, angle: f64, orb: f64) -> bool {
    (crate::chart::min_diff(a.lon, b.lon).abs() - angle).abs() <= orb
}

fn names(list: &[&Body]) -> String {
    list.iter().map(|b| b.name).collect::<Vec<_>>().join(", ")
}

fn triples<'a>(l: &[&'a Body], mut found: impl FnMut(&'a Body, &'a Body, &'a Body)) {
    for (i, a) in l.iter().enumerate() {
        for (j, b) in l.iter().enumerate().skip(i + 1) {
            for c in &l[j + 1..] {
                found(a, b, c);
            }
        }
    }
}

fn patterns(c: &Chart, out: &mut Vec<Item>) {
    let l = planets(c);
    let mut add = |name: &str, members: &[&Body]| {
        let mark = Mark { inner: members.iter().map(|b| b.index).collect(), ..Default::default() };
        out.extend(marked(format!("{name}: {}", names(members)), &format!("Obrazec | {name}"), mark))
    };
    let mut trines = Vec::new();
    triples(&l, |a, b, d| {
        if at(a, b, 120.0, PATTERN_ORB) && at(b, d, 120.0, PATTERN_ORB) && at(a, d, 120.0, PATTERN_ORB) {
            trines.push([a, b, d]);
        }
    });
    for t in &trines {
        add("Velký trigon", t);
    }
    let mut oppositions = Vec::new();
    for (i, a) in l.iter().enumerate() {
        for b in &l[i + 1..] {
            if at(a, b, 180.0, PATTERN_ORB) {
                oppositions.push((*a, *b));
            }
        }
    }
    let mut crossed = false;
    for (i, (a, b)) in oppositions.iter().enumerate() {
        for (d, e) in &oppositions[i + 1..] {
            if at(a, d, 90.0, PATTERN_ORB) && at(b, d, 90.0, PATTERN_ORB) && at(a, e, 90.0, PATTERN_ORB) {
                add("Velký kříž", &[*a, *b, *d, *e]);
                crossed = true;
            }
        }
    }
    if !crossed {
        for (a, b) in &oppositions {
            for d in &l {
                if at(a, d, 90.0, PATTERN_ORB) && at(b, d, 90.0, PATTERN_ORB) {
                    add("T-kvadrát", &[*a, *b, *d]);
                }
            }
        }
    }
    for (i, a) in l.iter().enumerate() {
        for b in &l[i + 1..] {
            if !at(a, b, 60.0, PATTERN_ORB) {
                continue;
            }
            for d in &l {
                if at(a, d, 150.0, FINE_ORB) && at(b, d, 150.0, FINE_ORB) {
                    add("Jod", &[*a, *b, *d]);
                }
            }
        }
    }
    for t in &trines {
        for (a, b) in &oppositions {
            let base = t.iter().any(|x| x.index == a.index) as u8 + t.iter().any(|x| x.index == b.index) as u8;
            if base == 1 {
                add("Drak", &[t[0], t[1], t[2], if t.iter().any(|x| x.index == a.index) { *b } else { *a }]);
            }
        }
    }
    for s in 0..12 {
        let group: Vec<&Body> = l.iter().copied().filter(|b| b.sign == s).collect();
        if group.len() >= 3 {
            add("Seskupení planet", &group);
        }
    }
}

/// The eight astrological phases of 45° in the title. Each starts at its exact point and lasts 45°,
/// so "fáze úplňku" runs from the full moon to about 3,7 days after it.
const PHASE_TITLES: [&str; 8] = [
    "fáze novoluní",
    "fáze dorůstajícího srpku",
    "fáze první čtvrti",
    "fáze dorůstajícího Měsíce",
    "fáze úplňku",
    "fáze couvajícího Měsíce",
    "fáze poslední čtvrti",
    "fáze couvajícího srpku",
];
/// The four points a calendar names, in the genitive after "po" and the instrumental after "před".
const MAIN_PHASES: [(&str, &str); 4] = [("novu", "novem"), ("první čtvrti", "první čtvrtí"), ("úplňku", "úplňkem"), ("poslední čtvrti", "poslední čtvrtí")];
/// Mean daily motion of the Moon away from the Sun, used when the speeds are missing.
const MEAN_ELONGATION_SPEED: f64 = 360.0 / 29.530589;

fn days_text(days: f64) -> String {
    let rounded = (days * 10.0).round() / 10.0;
    if rounded == 1.0 {
        "1 den".into()
    } else if rounded.fract() == 0.0 && (2.0..=4.0).contains(&rounded) {
        format!("{rounded:.0} dny")
    } else if rounded.fract() == 0.0 {
        format!("{rounded:.0} dní")
    } else {
        format!("{} dne", format!("{rounded:.1}").replace('.', ","))
    }
}

/// How the Moon actually looked, as a calendar would say it.
fn moon_state(angle: f64, speed: f64) -> String {
    let lit = ((1.0 - angle.to_radians().cos()) / 2.0 * 100.0).round();
    let quarter = ((angle / 90.0).round() as usize) % 4;
    let off = crate::chart::min_diff(angle, quarter as f64 * 90.0);
    let days = off.abs() / speed;
    let (after, before) = MAIN_PHASES[quarter];
    let when = if days < 0.05 {
        format!("přesně v {after}")
    } else if off > 0.0 {
        format!("{} po {after}", days_text(days))
    } else {
        format!("{} před {before}", days_text(days))
    };
    let trend = if lit >= 100.0 || lit <= 0.0 {
        String::new()
    } else if angle < 180.0 {
        ", dorůstá".into()
    } else {
        ", ubývá".into()
    };
    format!("Měsíc {when}, osvětlený z {lit:.0} %{trend}, {angle:.0}° od Slunce")
}

fn moon_phase(c: &Chart) -> Option<Item> {
    let body = |id: &str| c.bodies.iter().find(|b| b.id == id);
    let (moon, sun) = (body("moon")?, body("sun")?);
    let angle = crate::chart::norm360(moon.lon - sun.lon);
    let n = (angle / 45.0) as usize % 8;
    let speed = moon.speed - sun.speed;
    let speed = if speed > 9.0 { speed } else { MEAN_ELONGATION_SPEED };
    let mark = Mark { inner: vec![sun.index, moon.index], ..Default::default() };
    marked(format!("Fáze Měsíce: {} ({})", PHASE_TITLES[n], moon_state(angle, speed)), &format!("Fáze Měsíce | {}", PHASES[n]), mark)
}

/// The chart as a whole: moon phase, balance of elements, hemispheres and aspect patterns.
pub fn overview(c: &Chart) -> Section {
    let mut items: Vec<Item> = moon_phase(c).into_iter().collect();
    balance(c, &mut items);
    hemispheres(c, &mut items);
    patterns(c, &mut items);
    Section { title: "Celkový obraz".into(), items }
}

/// Where the ruler of each house stands, and which sign opens the house.
pub fn houses(c: &Chart) -> Section {
    let mut items = Vec::new();
    if let Some(cusps) = &c.cusps {
        for h in 1..=12 {
            let sign = (cusps[h - 1] / 30.0) as usize % 12;
            if h != 1 && h != 10 {
                items.extend(marked(format!("{h}. dům ve znamení {}", SIGNS[sign]), &format!("Hrot | {h}. dům | {}", SIGNS[sign]), Mark { house: Some(h), ..Default::default() }));
            }
            if let Some(ruler) = c.bodies.iter().find(|b| b.index == SIGN_RULER[sign]) {
                if let Some(in_house) = ruler.house {
                    items.extend(marked(format!("Vládce {h}. domu ({}) v {in_house}. domě", ruler.name), &format!("Vládce | {h}. dům | {in_house}. dům"), Mark::body(ruler.index, Some(h))));
                }
            }
        }
    }
    Section { title: "Domy a jejich vládci".into(), items }
}

/// Aspects in the sky reading: only close ones, they are what the days actually feel like.
pub const SKY_ORB: f64 = 3.0;

/// The sky of a moment for everyone: the Moon, the planets in their signs, retrograde planets
/// and the close aspects between them. Texts are the `Obloha` keys; a missing one is left out.
pub fn sky(c: &Chart, asp: &[Aspect]) -> Vec<Section> {
    let body = |i: usize| c.bodies.iter().find(|b| b.index == i);
    let mut moon = Vec::new();
    if let (Some(m), Some(s)) = (body(MOON), body(0)) {
        let angle = crate::chart::norm360(m.lon - s.lon);
        let n = (angle / 45.0) as usize % 8;
        let speed = m.speed - s.speed;
        let speed = if speed > 9.0 { speed } else { MEAN_ELONGATION_SPEED };
        let mark = Mark { inner: vec![s.index, m.index], ..Default::default() };
        moon.extend(marked(format!("Fáze Měsíce: {} ({})", PHASE_TITLES[n], moon_state(angle, speed)), &format!("Obloha | Fáze | {}", PHASES[n]), mark));
        moon.extend(marked(format!("Měsíc {}", SIGNS_LOC[m.sign]), &format!("Obloha | Měsíc | {}", m.sign_name), Mark::body(m.index, None)));
    }
    let planets: Vec<&Body> = c.bodies.iter().filter(|b| PLANETS.contains(&b.index) && b.index != MOON).collect();
    let signs = planets.iter().filter_map(|b| marked(format!("{} {}", b.name, SIGNS_LOC[b.sign]), &format!("Obloha | {} | {}", b.name, b.sign_name), Mark::body(b.index, None))).collect();
    let retro = planets
        .iter()
        .filter(|b| b.retro)
        .filter_map(|b| marked(format!("{} se pohybuje zpětně", b.name), &format!("Obloha | {} | Zpětný pohyb", b.name), Mark::body(b.index, None)))
        .collect();
    let aspects = asp
        .iter()
        .filter(|a| PLANETS.contains(&a.a) && PLANETS.contains(&a.b) && a.aspect_index < MAJOR_ASPECTS && a.orb <= SKY_ORB)
        .filter_map(|a| {
            let (first, second) = if a.a < a.b { (a.a_name, a.b_name) } else { (a.b_name, a.a_name) };
            marked(aspect_title(a), &format!("Obloha | {first} | {second} | {}", a.aspect_name), Mark::aspect(a.a, a.b, false, false))
        })
        .collect();
    [("Měsíc", moon), ("Planety ve znameních", signs), ("Zpětné planety", retro), ("Aspekty na obloze", aspects)]
        .into_iter()
        .filter(|(_, items): &(&str, Vec<Item>)| !items.is_empty())
        .map(|(title, items)| Section { title: title.into(), items })
        .collect()
}
