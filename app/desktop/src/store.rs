//! People and settings in one JSON file; writes go through a temp file so a crash cannot lose it.
//! The first change of each day keeps a copy of the file as it was, so a mistake can be undone.
use astro_core::model::{Person, Place, Settings};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Daily copies kept beside the data file.
const KEEP_COPIES: usize = 30;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Data {
    #[serde(default)]
    pub people: Vec<Person>,
    #[serde(default)]
    pub settings: Settings,
    /// Last place chosen on the sky screen.
    #[serde(default)]
    pub sky_place: Option<Place>,
}

pub struct Store {
    path: PathBuf,
    data: Mutex<Data>,
}

impl Store {
    pub fn open(path: PathBuf) -> Self {
        let data = match std::fs::read_to_string(&path).map(|t| serde_json::from_str(&t)) {
            Ok(Ok(d)) => d,
            Ok(Err(_)) => {
                // Unreadable file: keep a copy instead of overwriting it on the next save.
                let _ = std::fs::copy(&path, path.with_extension("poskozeno.json"));
                Data::default()
            }
            Err(_) => Data::default(),
        };
        Store { path, data: Mutex::new(data) }
    }

    /// Folder of the automatic daily copies.
    pub fn copies_dir(&self) -> PathBuf {
        self.path.with_file_name("zalohy")
    }

    /// Automatic copies, newest first.
    pub fn copies(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.copies_dir())
            .map(|dir| dir.flatten().map(|f| f.file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".json")).collect())
            .unwrap_or_default();
        names.sort_unstable_by(|a, b| b.cmp(a));
        names
    }

    fn keep_daily_copy(&self, day: &str) {
        let dir = self.copies_dir();
        let target = dir.join(format!("lide-{day}.json"));
        if target.exists() || !self.path.exists() || std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        // A failed copy must not stop the saving of real work.
        let _ = std::fs::copy(&self.path, target);
        for old in self.copies().iter().skip(KEEP_COPIES) {
            let _ = std::fs::remove_file(dir.join(old));
        }
    }

    /// Writes everything to a file the user chose, and returns how many people it holds.
    pub fn export(&self, path: &Path) -> Result<usize, String> {
        let data = self.get();
        let text = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
        std::fs::write(path, text).map_err(|e| format!("Zálohu se nepodařilo uložit: {e}"))?;
        Ok(data.people.len())
    }

    /// Adds people from a backup and changes nobody already here; returns how many were added.
    pub fn import(&self, path: &Path) -> Result<(Data, usize), String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("Zálohu se nepodařilo přečíst: {e}"))?;
        let backup: Data = serde_json::from_str(&text).map_err(|_| "Tento soubor není záloha programu Kosmogram.".to_string())?;
        if backup.people.is_empty() {
            return Err("V tomto souboru nejsou žádní lidé.".into());
        }
        let mut added = 0;
        let data = self.update(|d| {
            for person in backup.people {
                match d.people.iter_mut().find(|p| p.id == person.id) {
                    // Someone already here keeps their data, only an empty group is taken from the backup.
                    Some(p) => if p.group.is_empty() { p.group = person.group },
                    None => {
                        d.people.push(person);
                        added += 1;
                    }
                }
            }
            d.people.sort_by_key(|p| p.name.to_lowercase());
        })?;
        Ok((data, added))
    }

    pub fn get(&self) -> Data {
        self.data.lock().unwrap().clone()
    }

    pub fn update(&self, f: impl FnOnce(&mut Data)) -> Result<Data, String> {
        let mut d = self.data.lock().unwrap();
        let mut next = d.clone();
        f(&mut next);
        self.keep_daily_copy(&today());
        let text = serde_json::to_string_pretty(&next).map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, text).and_then(|_| std::fs::rename(&tmp, &self.path)).map_err(|e| format!("Uložení selhalo: {e}"))?;
        *d = next;
        Ok(d.clone())
    }
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(id: &str, name: &str) -> Person {
        Person { id: id.into(), name: name.into(), ..Person::default() }
    }

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("astrolog-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn names(d: &Data) -> Vec<&str> {
        d.people.iter().map(|p| p.name.as_str()).collect()
    }

    #[test]
    fn import_adds_the_missing_and_changes_nobody() {
        let dir = folder("import");
        let store = Store::open(dir.join("lide.json"));
        store.update(|d| d.people = vec![person("1", "Anna"), person("2", "Bára")]).unwrap();
        let backup = dir.join("zaloha.json");
        assert_eq!(store.export(&backup).unwrap(), 2);
        store.update(|d| {
            d.people.retain(|p| p.id != "1");
            d.people[0].name = "Bára nová".into();
            d.people.push(person("3", "Cyril"));
        }).unwrap();
        let (data, added) = store.import(&backup).unwrap();
        assert_eq!(added, 1);
        assert_eq!(names(&data), ["Anna", "Bára nová", "Cyril"]);
        assert_eq!(store.import(&backup).unwrap().1, 0, "podruhé už nic");
        let reopened = Store::open(dir.join("lide.json"));
        assert_eq!(names(&reopened.get()), ["Anna", "Bára nová", "Cyril"]);
    }

    #[test]
    fn import_keeps_groups_and_fills_only_an_empty_one() {
        let dir = folder("groups");
        let store = Store::open(dir.join("lide.json"));
        let with = |id, name, group: &str| Person { group: group.into(), ..person(id, name) };
        store.update(|d| d.people = vec![with("1", "Anna", "Rodina"), with("2", "Bára", "Klienti"), with("3", "Cyril", "Známí")]).unwrap();
        let backup = dir.join("zaloha.json");
        store.export(&backup).unwrap();
        store.update(|d| {
            d.people.retain(|p| p.id != "1");
            d.people[0].group = "Rodina".into();
            d.people[1].group.clear();
        }).unwrap();
        let (data, _) = store.import(&backup).unwrap();
        let groups: Vec<&str> = data.people.iter().map(|p| p.group.as_str()).collect();
        assert_eq!(groups, ["Rodina", "Rodina", "Známí"], "Anna se vrátí se skupinou, Bára si nechá novou, Cyril dostane zpět svou");
    }

    #[test]
    fn wrong_files_are_refused_and_change_nothing() {
        let dir = folder("wrong");
        let store = Store::open(dir.join("lide.json"));
        store.update(|d| d.people = vec![person("1", "Anna")]).unwrap();
        let (bad, empty) = (dir.join("spatny.json"), dir.join("prazdny.json"));
        std::fs::write(&bad, "toto není záloha").unwrap();
        std::fs::write(&empty, "{}").unwrap();
        assert!(store.import(&bad).unwrap_err().contains("není záloha"));
        assert!(store.import(&empty).unwrap_err().contains("žádní lidé"));
        assert!(store.import(&dir.join("neni.json")).is_err());
        assert_eq!(names(&store.get()), ["Anna"]);
    }

    #[test]
    fn a_daily_copy_holds_the_state_before_the_first_change() {
        let dir = folder("daily");
        let store = Store::open(dir.join("lide.json"));
        store.update(|d| d.people = vec![person("1", "Anna")]).unwrap();
        assert!(store.copies().is_empty() || !store.copies_dir().join("lide-1999-01-01.json").exists());
        store.keep_daily_copy("1999-01-01");
        store.update(|d| d.people.clear()).unwrap();
        store.keep_daily_copy("1999-01-01");
        let copy = store.copies_dir().join("lide-1999-01-01.json");
        let kept = Store::open(copy);
        assert_eq!(names(&kept.get()), ["Anna"], "druhá kopie téhož dne ji nepřepsala");
    }

    #[test]
    fn old_copies_are_dropped() {
        let dir = folder("rotate");
        let store = Store::open(dir.join("lide.json"));
        store.update(|d| d.people = vec![person("1", "Anna")]).unwrap();
        for day in 1..=40 {
            store.keep_daily_copy(&format!("2000-{:02}-{:02}", day / 28 + 1, day % 28 + 1));
        }
        let copies = store.copies();
        assert_eq!(copies.len(), KEEP_COPIES);
        assert!(copies.iter().any(|n| n.contains("2000-02")), "nejnovější zůstávají");
        assert!(!copies.iter().any(|n| n == "lide-2000-01-02.json"), "nejstarší pryč");
    }
}
