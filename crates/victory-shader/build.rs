//! What identifies the translator, for translations kept on disk
//! (`emit::translation_fingerprint`, `victory-gpu`'s `shader_cache`): a hash
//! of its sources, so a rebuild that does not change them keeps the
//! translations, and the names of every `VICTORY_*` switch they read, whose
//! values join the key at run time. Writes nothing but cargo directives.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Two FNV-1a hashes with different starts: not against anyone's will, only
/// to tell one set of sources from the next.
struct Hash(u64, u64);

impl Hash {
    fn add(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
            self.1 = (self.1.rotate_left(5) ^ u64::from(b)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
        // Where one part ends and the next begins.
        self.0 = (self.0 ^ bytes.len() as u64).wrapping_mul(0x100_0000_01b3);
        self.1 = (self.1 ^ bytes.len() as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("the crate's sources");
    for entry in entries {
        let path = entry.expect("an entry of the crate's sources").path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs")
            // Test modules of their own translate nothing for a title.
            && !path.file_name().is_some_and(|n| n.to_string_lossy().ends_with("_tests.rs"))
        {
            out.push(path);
        }
    }
}

/// The quoted `VICTORY_*` names in `text`.
fn switches(text: &str, out: &mut BTreeSet<String>) {
    let mut rest = text;
    while let Some(at) = rest.find("\"VICTORY_") {
        let tail = &rest[at + 1..];
        let end = tail
            .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
            .unwrap_or(tail.len());
        if tail[end..].starts_with('"') {
            out.insert(tail[..end].to_owned());
        }
        rest = &tail[end..];
    }
}

fn main() {
    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-changed=build.rs");
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    // What the translator is built with: its dependencies' versions.
    for lock in [root.join("../../Cargo.lock"), root.join("Cargo.toml")] {
        if lock.is_file() {
            println!("cargo::rerun-if-changed={}", lock.display());
            files.push(lock);
        }
    }
    // The same sources give the same hash wherever they lie and whatever
    // their line endings.
    let mut named: Vec<(String, PathBuf)> = files
        .into_iter()
        .map(|f| {
            let name = f.strip_prefix(&root).unwrap_or(&f).to_string_lossy().replace('\\', "/");
            (name, f)
        })
        .collect();
    named.sort();
    let mut hash = Hash(0xcbf2_9ce4_8422_2325, 0x1234_5678_9abc_def0);
    let mut names = BTreeSet::new();
    for (name, file) in named {
        let mut bytes = std::fs::read(&file).expect("a source of the crate");
        bytes.retain(|&b| b != b'\r');
        hash.add(name.as_bytes());
        hash.add(&bytes);
        switches(&String::from_utf8_lossy(&bytes), &mut names);
    }
    // And the compiler (what it folds of the float code is its own).
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    if let Ok(out) = std::process::Command::new(rustc).arg("-vV").output() {
        hash.add(&out.stdout);
    }
    println!("cargo::rustc-env=VICTORY_SHADER_SOURCE_HASH={:016x}{:016x}", hash.0, hash.1);
    let names: Vec<String> = names.into_iter().collect();
    println!("cargo::rustc-env=VICTORY_SHADER_SWITCHES={}", names.join(","));
}
