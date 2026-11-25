//! Populate the runtime shader cache from the user's local files.

use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result, bail};
use victory_shader::{Container, FourCC};

use crate::shader::{Source, sources};

pub fn run(paths: Vec<PathBuf>, jobs: Option<usize>) -> Result<()> {
    let paths = crate::default_paths(paths)?;
    let config = victory_core::Config::load()?;
    let cache = config.cache_subdir("shaders")?;
    victory_gpu::shader_cache::prepare_precompile(cache.clone()).map_err(anyhow::Error::msg)?;
    let workers = jobs.unwrap_or_else(|| {
        std::thread::available_parallelism().map_or(1, |n| (n.get() / 2).max(1))
    });
    if workers == 0 {
        bail!("worker count must be at least one");
    }
    println!("Shader cache: {}", cache.display());
    println!("Translator workers: {workers}");
    let seen = Mutex::new(HashSet::new());
    let done = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let mut skipped = 0usize;
    let mut files = 0usize;
    for root in paths {
        if !root.exists() {
            bail!("input path does not exist: {}", root.display());
        }
        for entry in walkdir::WalkDir::new(&root) {
            let entry = entry.with_context(|| format!("walking {}", root.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            const CHUNK: usize = 64 << 20;
            let mut file = match std::fs::File::open(path) {
                Ok(file) => file,
                Err(error) => {
                    skipped += 1;
                    eprintln!("Could not read {}: {error}", path.display());
                    continue;
                }
            };
            let file_bytes = entry.metadata()?.len();
            println!("Reading {} ({} MiB)", path.display(), file_bytes / (1 << 20));
            let mut read_bytes = 0u64;
            let mut next_report = 1u64 << 30;
            let mut data = Vec::new();
            let mut found_in_file = false;
            loop {
                let previous = data.len();
                data.resize(previous + CHUNK, 0);
                let mut added = 0;
                while added < CHUNK {
                    match file.read(&mut data[previous + added..]) {
                        Ok(0) => break,
                        Ok(n) => added += n,
                        Err(error) => {
                            eprintln!("Could not read {}: {error}", path.display());
                            skipped += 1;
                            break;
                        }
                    }
                }
                data.truncate(previous + added);
                read_bytes += added as u64;
                if read_bytes >= next_report {
                    println!("Read {} of {} MiB", read_bytes / (1 << 20), file_bytes / (1 << 20));
                    next_report = read_bytes.saturating_add(1 << 30);
                }
                if added == 0 {
                    break;
                }
                let mut found = sources(&data);
                if !victory_game::sgd5::is_sgd5(&data) {
                    found.extend(victory_game::sgd5::records(&data).map(Source::Packed));
                }
                if !found.is_empty() && !found_in_file {
                    files += 1;
                    found_in_file = true;
                    println!("Scanning {}", path.display());
                }
                let next = AtomicUsize::new(0);
                std::thread::scope(|scope| {
                    for _ in 0..workers.min(found.len()) {
                        scope.spawn(|| {
                            while let Some(source) = found.get(next.fetch_add(1, Ordering::Relaxed))
                            {
                                let Some(bytes) = source.bytes() else {
                                    failed.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                };
                                let Ok(container) = Container::parse(&bytes) else {
                                    continue;
                                };
                                if container.part(FourCC::DXIL).is_none() {
                                    continue;
                                }
                                let len = u32::from_le_bytes(bytes[0x18..0x1c].try_into().unwrap())
                                    as usize;
                                let bytes = &bytes[..len];
                                let hash = blake3::hash(bytes);
                                if !seen.lock().unwrap_or_else(|e| e.into_inner()).insert(hash) {
                                    continue;
                                }
                                if victory_gpu::shader_cache::host_shader(bytes).is_err() {
                                    failed.fetch_add(1, Ordering::Relaxed);
                                }
                                let count = done.fetch_add(1, Ordering::Relaxed) + 1;
                                if count.is_multiple_of(100) {
                                    println!(
                                        "Processed {count} shaders ({} failed)",
                                        failed.load(Ordering::Relaxed)
                                    );
                                }
                            }
                        });
                    }
                });
                if added < CHUNK {
                    break;
                }
                if data.len() > CHUNK {
                    let keep = data.len() - CHUNK;
                    data.copy_within(keep.., 0);
                    data.truncate(CHUNK);
                }
            }
        }
    }
    let (hits, misses) = victory_gpu::shader_cache::counts();
    println!("Cache hits: {hits}, newly translated: {misses}");
    let count = done.load(Ordering::Relaxed);
    let failures = failed.load(Ordering::Relaxed);
    println!(
        "Finished: {count} shaders processed, {failures} failed, {files} files with candidates, {skipped} files skipped."
    );
    if count == 0 {
        bail!("no shader containers were found");
    }
    Ok(())
}
