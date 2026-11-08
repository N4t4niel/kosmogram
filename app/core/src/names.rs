//! Czech names in relationship readings: gender guess, declension and text templates.
//!
//! Templates: `{1}` is the first person, `{1g}`, `{1d}`, `{1a}`, `{1l}` and `{1i}` the other
//! cases, `{1:on|ona}` picks by gender. `{p:oba|obě}` uses the second form only for two women.

use crate::model::Person;
use serde::Serialize;

const NOM: usize = 0;
const GEN: usize = 1;
const DAT: usize = 2;
const ACC: usize = 3;
const LOC: usize = 4;
const INS: usize = 5;

/// Names and family words ending in -a that belong to men.
const MALE_A: [&str; 22] = [
    "honza", "jirka", "standa", "jarda", "mirda", "franta", "pepa", "tonda", "vojta", "kuba", "míša", "saša", "ilja", "nikita", "joshua", "luca", "andrea", "táta", "tatka", "děda", "strejda", "brácha",
];
/// Women's names that do not end in a vowel typical for Czech.
const FEMALE_OTHER: [&str; 14] = ["ester", "karin", "ingrid", "dagmar", "miriam", "carmen", "rút", "rut", "ruth", "noemi", "naomi", "sandy", "kim", "abigail"];
/// Foreign men's names ending in -e.
const MALE_E: [&str; 9] = ["mike", "joe", "dave", "steve", "george", "uwe", "luke", "pierre", "jose"];
const FLEETING_EL: [&str; 3] = ["pavel", "karel", "havel"];
const SOFT: [char; 9] = ['š', 'ž', 'č', 'ř', 'j', 'c', 'ď', 'ť', 'ň'];
const SOFT_LIKE: [char; 3] = ['s', 'x', 'z'];

#[derive(Clone, Debug, Serialize)]
pub struct Name {
    /// The name as the person was entered, for headings.
    pub full: String,
    pub cases: [String; 6],
    pub female: bool,
}

fn drop_last(s: &str, n: usize) -> String {
    s.chars().take(s.chars().count().saturating_sub(n)).collect()
}

fn ends_soft(s: &str) -> bool {
    s.chars().last().is_some_and(|c| SOFT.contains(&c))
}

fn same(w: &str) -> [String; 6] {
    std::array::from_fn(|_| w.to_string())
}

fn with_endings(stem: &str, e: [&str; 6]) -> [String; 6] {
    e.map(|x| format!("{stem}{x}"))
}

/// Dative and locative of a feminine name in -a, built from the stem.
fn feminine_soft_form(stem: &str) -> String {
    let lower = stem.to_lowercase();
    if lower.ends_with("ch") {
        return format!("{}še", drop_last(stem, 2));
    }
    match lower.chars().last() {
        Some('k') => format!("{}ce", drop_last(stem, 1)),
        Some('g' | 'h') => format!("{}ze", drop_last(stem, 1)),
        Some('r') => format!("{}ře", drop_last(stem, 1)),
        Some('l' | 's' | 'z') => format!("{stem}e"),
        Some(c) if SOFT.contains(&c) => format!("{stem}e"),
        _ => format!("{stem}ě"),
    }
}

fn feminine(w: &str) -> [String; 6] {
    let lower = w.to_lowercase();
    if lower.ends_with("ová") {
        let s = drop_last(w, 1);
        return [w.to_string(), format!("{s}é"), format!("{s}é"), format!("{s}ou"), format!("{s}é"), format!("{s}ou")];
    }
    if lower.ends_with("ia") {
        let s = drop_last(w, 1);
        return with_endings(&s, ["a", "e", "i", "i", "i", "í"]);
    }
    if lower.ends_with('a') {
        let s = drop_last(w, 1);
        let g = if ends_soft(&s.to_lowercase()) { "i" } else { "y" };
        return [w.to_string(), format!("{s}{g}"), feminine_soft_form(&s), format!("{s}u"), feminine_soft_form(&s), format!("{s}ou")];
    }
    if ["ie", "ce", "še", "že", "ře", "če", "ě"].iter().any(|e| lower.ends_with(e)) {
        let s = drop_last(w, 1);
        return [w.to_string(), w.to_string(), format!("{s}i"), format!("{s}i"), format!("{s}i"), format!("{s}í")];
    }
    same(w)
}

fn masculine_stem(w: &str) -> String {
    let lower = w.to_lowercase();
    if lower.ends_with("něk") {
        return format!("{}ňk", drop_last(w, 3));
    }
    if lower.ends_with("ek") || lower.ends_with("ec") {
        return format!("{}{}", drop_last(w, 2), w.chars().last().unwrap_or('k'));
    }
    if FLEETING_EL.contains(&lower.as_str()) {
        return format!("{}l", drop_last(w, 2));
    }
    w.to_string()
}

fn masculine(w: &str) -> [String; 6] {
    let lower = w.to_lowercase();
    let last = lower.chars().last().unwrap_or(' ');
    match last {
        'a' => {
            let s = drop_last(w, 1);
            let g = if ends_soft(&s.to_lowercase()) { "i" } else { "y" };
            with_endings(&s, ["a", g, "ovi", "u", "ovi", "ou"])
        }
        'o' => {
            let s = drop_last(w, 1);
            with_endings(&s, ["o", "a", "ovi", "a", "ovi", "em"])
        }
        'í' | 'ý' => [w.to_string(), format!("{w}ho"), format!("{w}mu"), format!("{w}ho"), format!("{w}m"), format!("{w}m")],
        'e' | 'é' | 'ě' | 'i' | 'y' | 'u' | 'ú' | 'ů' | 'á' => same(w),
        c => {
            let s = masculine_stem(w);
            let soft = s.to_lowercase().chars().last().is_some_and(|x| SOFT.contains(&x)) || SOFT_LIKE.contains(&c);
            let e = if soft { "e" } else { "a" };
            [w.to_string(), format!("{s}{e}"), format!("{s}ovi"), format!("{s}{e}"), format!("{s}ovi"), format!("{s}em")]
        }
    }
}

pub fn decline(word: &str, female: bool) -> [String; 6] {
    if female {
        feminine(word)
    } else {
        masculine(word)
    }
}

pub fn looks_female(word: &str) -> bool {
    let w = word.to_lowercase();
    if MALE_A.contains(&w.as_str()) || MALE_E.contains(&w.as_str()) {
        return false;
    }
    if FEMALE_OTHER.contains(&w.as_str()) {
        return true;
    }
    w.ends_with('a') || w.ends_with('e') || w.ends_with('ě') || w.ends_with("ová")
}

/// The word that identifies a person in a sentence, without titles such as Ing.
fn given(name: &str) -> &str {
    name.split_whitespace().find(|w| !w.ends_with('.')).unwrap_or(name)
}

fn is_female(name: &str, sex: &str) -> bool {
    match sex {
        "f" => true,
        "m" => false,
        _ => {
            let last = name.split_whitespace().last().unwrap_or("").to_lowercase();
            last.ends_with("ová") || looks_female(given(name))
        }
    }
}

pub fn name_of(name: &str, sex: &str) -> Name {
    let female = is_female(name, sex);
    Name { full: name.to_string(), cases: decline(given(name), female), female }
}

/// Two people with the same given name are told apart by a surname initial.
pub fn pair(a: &Person, b: &Person) -> (Name, Name) {
    let (mut x, mut y) = (name_of(&a.name, &a.sex), name_of(&b.name, &b.sex));
    if x.cases[NOM].to_lowercase() == y.cases[NOM].to_lowercase() {
        add_initial(&mut x);
        add_initial(&mut y);
    }
    (x, y)
}

fn add_initial(n: &mut Name) {
    let Some(i) = n.full.split_whitespace().filter(|w| !w.ends_with('.')).nth(1).and_then(|w| w.chars().next()) else { return };
    for c in &mut n.cases {
        c.push_str(&format!(" {i}."));
    }
}

impl Name {
    pub fn nom(&self) -> &str {
        &self.cases[NOM]
    }
    pub fn gen(&self) -> &str {
        &self.cases[GEN]
    }
}

/// Adds the vowel to a one-letter preposition before a name that needs it.
fn vocalize(out: &mut String, name: &str) {
    let Some(first) = name.chars().next().and_then(|c| c.to_lowercase().next()) else { return };
    let needs = |p: char| match p {
        'v' => matches!(first, 'v' | 'f'),
        's' | 'z' => matches!(first, 's' | 'š' | 'z' | 'ž'),
        'k' => matches!(first, 'k' | 'g'),
        _ => false,
    };
    let mut tail = out.chars().rev();
    let (Some(' '), Some(p), before) = (tail.next(), tail.next(), tail.next()) else { return };
    if !matches!(before, None | Some(' ' | '(')) || !needs(p.to_lowercase().next().unwrap_or(p)) {
        return;
    }
    out.pop();
    out.push_str("e ");
}

fn case_index(c: &str) -> Option<usize> {
    Some(match c {
        "" => NOM,
        "g" => GEN,
        "d" => DAT,
        "a" => ACC,
        "l" => LOC,
        "i" => INS,
        _ => return None,
    })
}

fn token(t: &str, a: &Name, b: &Name) -> Option<String> {
    if let Some((who, forms)) = t.split_once(':') {
        let (m, f) = forms.split_once('|')?;
        let female = match who {
            "1" => a.female,
            "2" => b.female,
            "p" => a.female && b.female,
            _ => return None,
        };
        return Some(if female { f } else { m }.to_string());
    }
    let person = match t.chars().next()? {
        '1' => a,
        '2' => b,
        _ => return None,
    };
    Some(person.cases[case_index(&t[1..])?].clone())
}

/// Puts two people into a template. An unknown token stays visible instead of being lost.
pub fn fill(text: &str, a: &Name, b: &Name) -> String {
    let mut out = String::with_capacity(text.len() + 32);
    let mut rest = text;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let Some(j) = rest[i..].find('}') else {
            rest = &rest[i..];
            break;
        };
        let inner = &rest[i + 1..i + j];
        match token(inner, a, b) {
            Some(v) => {
                if !inner.contains(':') {
                    vocalize(&mut out, &v);
                }
                out.push_str(&v);
            }
            None => out.push_str(&rest[i..=i + j]),
        }
        rest = &rest[i + j + 1..];
    }
    out.push_str(rest);
    out
}
