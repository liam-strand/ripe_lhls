use std::fs::File;
use std::io::Write;
use std::net::IpAddr;

use ahash::AHashSet;
use clap::Parser;
use ripe_lhls::executor;
use ripe_lhls::models::TracerouteResult;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory containing the ripe traceroute .bz2 files
    #[arg(short, long, default_value = "/tank/yhe7443/ripe_traceroutes")]
    data_dir: PathBuf,

    /// Output file for unique IPs
    #[arg(short, long, default_value = "unique_ips.txt")]
    output_ips: PathBuf,

    /// Estimated lines per file for the progress bar calculation
    #[arg(short, long, default_value_t = 11_830_000)]
    pub estimated_lines_per_file: usize,

    /// Number of worker threads to spawn. Defaults to the number of logical cores.
    #[arg(short = 'j', long = "jobs", default_value_t = 0)]
    pub jobs: usize,
}

impl Args {
    pub fn get_jobs(&self) -> usize {
        if self.jobs == 0 {
            std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1)
        } else {
            self.jobs
        }
    }
}

fn main() {
    let args = Args::parse();

    // Process all files in parallel
    let global_ips = executor::run_extraction(
        &args.data_dir,
        args.get_jobs(),
        args.estimated_lines_per_file,
        None,
        |local_ips: &mut AHashSet<IpAddr>, data: &TracerouteResult| {
            // Check top-level IPs
            local_ips.extend([data.src_addr, data.dst_addr, data.from].iter().flatten());

            // Extract hop information
            local_ips.extend::<Vec<IpAddr>>(
                data.result
                    .as_ref()
                    .map(|results| {
                        results
                            .iter()
                            .filter_map(|hop_group| hop_group.result.as_ref())
                            .flatten()
                            .flat_map(|reply| [reply.from, reply.edst].into_iter().flatten())
                            .collect()
                    })
                    .unwrap_or_default(),
            );
        },
        |mut acc: AHashSet<IpAddr>, local: AHashSet<IpAddr>| {
            acc.extend(local);
            acc
        },
    )
    .expect("Failed to extract IPs");

    println!("Found {} unique IP addresses.", global_ips.len());

    println!("Writing IPs to {}...", args.output_ips.display());
    if let Ok(mut output) = File::create(&args.output_ips) {
        for ip in global_ips {
            let _ = writeln!(output, "{}", ip);
        }
    }

    println!("All done!");
}
