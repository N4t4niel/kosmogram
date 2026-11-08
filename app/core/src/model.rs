use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Place {
    pub name: String,
    pub region: String,
    pub country: String,
    pub lat: f64,
    /// East positive (Astrolog's atlas uses the opposite sign).
    pub lon: f64,
    pub tz: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Person {
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// YYYY-MM-DD
    pub date: String,
    /// HH:MM, ignored when `time_known` is false
    pub time: String,
    pub time_known: bool,
    pub place: Place,
    /// Manual UTC offset in minutes; replaces the automatic time zone.
    #[serde(default)]
    pub offset_min: Option<i32>,
    #[serde(default)]
    pub note: String,
    /// "m" or "f", empty means the gender is guessed from the name.
    #[serde(default)]
    pub sex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub house_system: char,
    pub chiron: bool,
    pub minor_aspects: bool,
    /// Progressed angles and cusps by solar arc, about 1° a year, instead of Astrolog's daily turn.
    #[serde(default)]
    pub solar_arc_angles: bool,
    /// Night theme of the app.
    #[serde(default)]
    pub dark: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { house_system: 'P', chiron: false, minor_aspects: false, solar_arc_angles: false, dark: false }
    }
}
