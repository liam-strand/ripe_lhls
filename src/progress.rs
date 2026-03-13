use crossbeam_channel::Receiver;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn make_style(units: &'static str) -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(&format!(
            "[{{elapsed_precise}}] |{{wide_bar}}| {{pos}}/{{len}} {} ({{eta}})",
            units
        ))
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏  ")
}

pub enum ProgressMsg {
    Update(usize),
    FileDone { remainder: usize, correction: i64 },
    Done,
}

pub fn progress_thread(
    progress_rx: Receiver<ProgressMsg>,
    total_files: usize,
    estimated_lines_per_file: usize,
) {
    let total_lines = total_files * estimated_lines_per_file;
    let pb = ProgressBar::new(total_lines as u64);
    pb.set_style(make_style("lines"));
    pb.enable_steady_tick(Duration::from_secs_f32(0.07));

    while let Ok(msg) = progress_rx.recv() {
        match msg {
            ProgressMsg::Update(lines) => {
                pb.inc(lines as u64);
            }
            ProgressMsg::FileDone {
                remainder,
                correction,
            } => {
                pb.inc(remainder as u64);
                if correction != 0 {
                    let current_len = pb.length().unwrap_or(0);
                    let new_len = if correction > 0 {
                        current_len + (correction as u64)
                    } else {
                        current_len - ((-correction) as u64)
                    };
                    pb.set_length(new_len);
                }
            }
            ProgressMsg::Done => {
                pb.finish_with_message("Done");
                break;
            }
        }
    }
}
