use std::path::{Path, PathBuf};

use crossbeam_channel::unbounded;
use rayon::prelude::*;
use std::time::Instant;

use crate::extractor;
use crate::models::TracerouteResult;
use crate::progress::{ProgressMsg, progress_thread};

pub fn run_extraction<T, F, R>(
    data_dir: &Path,
    jobs: usize,
    estimated_lines_per_file: usize,
    limit: Option<usize>,
    visit: F,
    reduce: R,
) -> Option<T>
where
    T: Default + Send + Sync,
    F: Fn(&mut T, &TracerouteResult) + Sync + Send,
    R: Fn(T, T) -> T + Sync + Send,
{
    let start_time = Instant::now();

    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global();

    if !data_dir.exists() {
        eprintln!(
            "Data directory '{}' not found. Run the downloader first.",
            data_dir.display()
        );
        return None;
    }

    let mut bz2_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "bz2") {
                bz2_files.push(path);
            }
        }
    }

    if bz2_files.is_empty() {
        eprintln!("No .bz2 files found in the data directory.");
        return None;
    }

    let total_files = bz2_files.len();
    println!(
        "Found {} compressed archives. Starting parallel extraction using {} workers...",
        total_files, jobs
    );

    let (progress_tx, progress_rx) = unbounded();
    let limit_val = limit.unwrap_or(usize::MAX);

    let progress_thread = std::thread::spawn(move || {
        progress_thread(progress_rx, total_files, estimated_lines_per_file);
    });

    // Process all files in parallel
    let result = bz2_files
        .par_iter()
        .map(|filepath| {
            extractor::process_file(
                filepath,
                estimated_lines_per_file,
                limit_val,
                progress_tx.clone(),
                &visit,
            )
        })
        .reduce(T::default, reduce);

    // Tell the progress bar we are done
    let _ = progress_tx.send(ProgressMsg::Done);
    let _ = progress_thread.join();

    let duration = start_time.elapsed();
    println!("\nExtraction complete in {:?}", duration);

    Some(result)
}
