use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::models::*;
use crate::progress::ProgressMsg;
use bzip2::read::BzDecoder;

pub fn process_file<T, F>(
    filepath: &Path,
    estimated_lines_per_file: usize,
    limit: usize,
    progress_tx: crossbeam_channel::Sender<ProgressMsg>,
    mut visit: F,
) -> T
where
    T: Default,
    F: FnMut(&mut T, &TracerouteResult),
{
    let mut local_state = T::default();
    let mut lines_processed: usize = 0;
    let mut lines_since_update: usize = 0;

    // Add a pseudo-random jitter to the start time based on the filepath
    // so that threads stagger their progress bar updates and don't clump.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    filepath.hash(&mut hasher);
    let offset_millis = hasher.finish() % 1000;
    let mut last_update = Instant::now()
        .checked_sub(Duration::from_millis(offset_millis))
        .unwrap_or_else(Instant::now);

    let file = match File::open(filepath) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("\n[!] Error opening {:?}: {}", filepath.file_name(), e);
            let correction = -(estimated_lines_per_file as i64);
            let _ = progress_tx.send(ProgressMsg::FileDone {
                remainder: 0,
                correction,
            });
            return local_state;
        }
    };

    let bz = BzDecoder::new(file);
    let reader = BufReader::new(bz);

    for line_res in reader.lines().take(limit) {
        lines_processed += 1;
        lines_since_update += 1;

        if last_update.elapsed().as_secs() >= 1 {
            let _ = progress_tx.send(ProgressMsg::Update(lines_since_update));
            lines_since_update = 0;
            last_update = Instant::now();
        }

        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue, // Corrupted compressed block
        };

        let data: TracerouteResult = match serde_json::from_str(&line) {
            Ok(d) => d,
            Err(_) => continue, // Corrupted JSON
        };

        visit(&mut local_state, &data);
    }

    let remainder = lines_since_update;
    let correction = (lines_processed as i64) - (estimated_lines_per_file as i64);
    let _ = progress_tx.send(ProgressMsg::FileDone {
        remainder,
        correction,
    });

    local_state
}
