//! Joins every text file in data/vyklady into one, so adding a file needs no code change.
use std::path::Path;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=data/vyklady");
    let mut files: Vec<_> = fs::read_dir("data/vyklady").unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|x| x == "txt")).collect();
    files.sort();
    let mut all = String::new();
    for f in files {
        all += &fs::read_to_string(f).unwrap();
        all.push('\n');
    }
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("vyklady.txt"), all).unwrap();
}
