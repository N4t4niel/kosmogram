//! Tables taken from Astrolog's data.cpp defaults.

pub const SIGNS: [&str; 12] = ["Beran", "Býk", "Blíženci", "Rak", "Lev", "Panna", "Váhy", "Štír", "Střelec", "Kozoroh", "Vodnář", "Ryby"];
pub const SIGNS_LOC: [&str; 12] = ["v Beranu", "v Býku", "v Blížencích", "v Raku", "ve Lvu", "v Panně", "ve Vahách", "ve Štíru", "ve Střelci", "v Kozorohu", "ve Vodnáři", "v Rybách"];
/// Astronomicon font characters.
pub const SIGN_GLYPHS: [char; 12] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', '\\', 'K', 'L'];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjKind { Body(crate::ephem::Id), Asc, Mc }

#[derive(Clone, Copy, Debug)]
pub struct Obj {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ObjKind,
    pub glyph: char,
    /// Astrolog rObjInf
    pub influence: f64,
    /// Astrolog rObjOrb cap (360 = none) and rObjAdd
    pub orb_cap: f64,
    pub orb_add: f64,
    /// sign indices (0 = Aries)
    pub rulers: &'static [usize],
    pub exalt: Option<usize>,
}

pub const MOON: usize = 1;
pub const CHIRON: usize = 10;

const fn o(id: &'static str, name: &'static str, kind: ObjKind, glyph: char, influence: f64, orb_cap: f64, orb_add: f64, rulers: &'static [usize], exalt: Option<usize>) -> Obj {
    Obj { id, name, kind, glyph, influence, orb_cap, orb_add, rulers, exalt }
}

pub const OBJECTS: [Obj; 14] = [
    o("sun", "Slunce", ObjKind::Body(0), 'Q', 30.0, 360.0, 1.0, &[4], Some(0)),
    o("moon", "Měsíc", ObjKind::Body(1), 'R', 25.0, 360.0, 1.0, &[3], Some(1)),
    o("mercury", "Merkur", ObjKind::Body(2), 'S', 10.0, 360.0, 0.0, &[2, 5], Some(5)),
    o("venus", "Venuše", ObjKind::Body(3), 'T', 10.0, 360.0, 0.0, &[6, 1], Some(11)),
    o("mars", "Mars", ObjKind::Body(4), 'U', 10.0, 360.0, 0.0, &[0, 7], Some(9)),
    o("jupiter", "Jupiter", ObjKind::Body(5), 'V', 10.0, 360.0, 0.0, &[8, 11], Some(3)),
    o("saturn", "Saturn", ObjKind::Body(6), 'W', 10.0, 360.0, 0.0, &[9, 10], Some(6)),
    o("uranus", "Uran", ObjKind::Body(7), 'X', 8.0, 360.0, 0.0, &[10], Some(7)),
    o("neptune", "Neptun", ObjKind::Body(8), 'Y', 8.0, 360.0, 0.0, &[11], Some(3)),
    o("pluto", "Pluto", ObjKind::Body(9), 'Z', 8.0, 360.0, 0.0, &[7], Some(10)),
    o("chiron", "Chiron", ObjKind::Body(15), 'q', 6.0, 360.0, 0.0, &[8], Some(11)),
    o("node", "Severní uzel", ObjKind::Body(10), 'g', 5.0, 2.0, 0.0, &[], None),
    o("asc", "Ascendent", ObjKind::Asc, 'c', 20.0, 360.0, 0.0, &[], None),
    o("mc", "Medium coeli", ObjKind::Mc, 'd', 15.0, 360.0, 0.0, &[], None),
];

#[derive(Clone, Copy, Debug)]
pub struct AspectDef {
    pub id: &'static str,
    pub name: &'static str,
    pub angle: f64,
    pub orb: f64,
    pub influence: f64,
    pub glyph: char,
    pub color: &'static str,
}

pub const ASPECTS: [AspectDef; 9] = [
    AspectDef { id: "con", name: "Konjunkce", angle: 0.0, orb: 7.0, influence: 1.0, glyph: '!', color: "#c98a12" },
    AspectDef { id: "opp", name: "Opozice", angle: 180.0, orb: 7.0, influence: 0.8, glyph: '"', color: "#c0392b" },
    AspectDef { id: "squ", name: "Kvadratura", angle: 90.0, orb: 7.0, influence: 0.8, glyph: '#', color: "#c0392b" },
    AspectDef { id: "tri", name: "Trigon", angle: 120.0, orb: 7.0, influence: 0.6, glyph: '$', color: "#2e7fb8" },
    AspectDef { id: "sex", name: "Sextil", angle: 60.0, orb: 6.0, influence: 0.6, glyph: '%', color: "#2e9e6b" },
    AspectDef { id: "qcx", name: "Kvinkunx", angle: 150.0, orb: 3.0, influence: 0.4, glyph: '&', color: "#8a6db8" },
    AspectDef { id: "ssx", name: "Polosextil", angle: 30.0, orb: 3.0, influence: 0.4, glyph: '\'', color: "#8a6db8" },
    AspectDef { id: "ssq", name: "Polokvadrát", angle: 45.0, orb: 3.0, influence: 0.2, glyph: '(', color: "#8a6db8" },
    AspectDef { id: "ses", name: "Seskvikvadrát", angle: 135.0, orb: 3.0, influence: 0.2, glyph: ')', color: "#8a6db8" },
];

pub const MAJOR_ASPECTS: usize = 5;

pub const HOUSE_SYSTEMS: [(char, &str); 8] = [
    ('P', "Placidus"), ('K', "Koch"), ('E', "Rovné domy"), ('W', "Celá znamení"),
    ('R', "Regiomontanus"), ('C', "Campanus"), ('O', "Porphyrius"), ('B', "Alcabitius"),
];

pub const DAYS_IN_YEAR: f64 = 365.24219;
