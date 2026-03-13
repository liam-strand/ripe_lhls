use clap::Parser;
use geo::{Distance, Geodesic, Point};
use indicatif::ProgressIterator;
use ripe_lhls::adtk::{Detector, LevelShiftAD};
use ripe_lhls::executor;
use ripe_lhls::models::{
    GeolocateRecord, LHLKey, LHLRecord, LocationInfo, TracerouteReply, TracerouteResult,
};
use ripe_lhls::progress::make_style;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

const C: f64 = 299792458.0;
const FIBER_C: f64 = C * 2.0 / 3.0;
const MS_PER_S: f64 = 1000.0;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory containing the ripe traceroute .bz2 files
    #[arg(short, long, default_value = "/tank/yhe7443/ripe_traceroutes")]
    data_dir: PathBuf,

    /// Output file for LHLs
    #[arg(short, long, default_value = "lhls.jsonl")]
    output_lhls: PathBuf,

    /// Estimated lines per file for the progress bar calculation
    #[arg(short, long, default_value_t = 11_830_000)]
    pub estimated_lines_per_file: usize,

    /// Geolocation file
    #[arg(short, long, default_value = "geolocation_results.jsonl")]
    pub geolocation_file: PathBuf,

    /// Number of worker threads to spawn. Defaults to the number of logical cores.
    /// Number of worker threads to spawn. Defaults to the number of logical cores.
    #[arg(short = 'j', long = "jobs", default_value_t = 0)]
    pub jobs: usize,

    /// Run in test mode (only process the first 1000 lines of each file)
    #[arg(short, long, default_value_t = false)]
    pub test: bool,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let limit = if args.test { Some(10000) } else { None };
    let lines_per_file = limit.unwrap_or(args.estimated_lines_per_file);

    println!("Loading geolocation data...");

    let geolocation_map: HashMap<IpAddr, LocationInfo> = {
        let geolocation_file = File::open(&args.geolocation_file)?;
        let geolocation_reader = BufReader::new(geolocation_file);

        geolocation_reader
            .lines()
            .map_while(|e| e.ok())
            .filter_map(|e| serde_json::from_str::<GeolocateRecord>(&e).ok())
            .filter_map(|e| Some((IpAddr::from_str(&e.ip).ok()?, e.location)))
            .collect()
    };

    // Process all files in parallel
    let global_lhls = executor::run_extraction(
        &args.data_dir,
        args.get_jobs(),
        lines_per_file,
        limit,
        |local_lhls: &mut HashMap<LHLKey, f64>, data: &TracerouteResult| {
            let mut detector = LevelShiftAD::new(1, 1.0);
            let Some(results) = data.result.as_ref() else {
                return;
            };

            let min_rtt_replies: Vec<&TracerouteReply> = results
                .iter()
                .filter_map(|hop_group| hop_group.result.as_ref())
                .filter_map(|replies| {
                    replies
                        .iter()
                        .filter(|reply| reply.rtt.is_some())
                        .min_by(|a, b| a.rtt.unwrap().total_cmp(&b.rtt.unwrap()))
                })
                .collect();

            let rtts: Vec<f64> = min_rtt_replies
                .iter()
                .map(|reply| reply.rtt.unwrap())
                .collect();

            let anomaly_idxs = detector.fit_detect(&rtts);

            let found_lhls: Vec<(IpAddr, IpAddr, f64)> = anomaly_idxs
                .into_iter()
                .filter(|&idx| idx > 0)
                .filter_map(|idx| {
                    let prev_reply = min_rtt_replies[idx - 1];
                    let curr_reply = min_rtt_replies[idx];

                    let src = prev_reply.from?;
                    let dst = curr_reply.from?;

                    let prev_rtt = prev_reply.rtt?;
                    let curr_rtt = curr_reply.rtt?;

                    let link_rtt = curr_rtt - prev_rtt;

                    Some((src, dst, link_rtt))
                })
                .filter(|(_, _, link_rtt)| *link_rtt > 57.0)
                .filter_map(|(src, dst, link_rtt)| {
                    let src_loc = geolocation_map.get(&src)?;
                    let dst_loc = geolocation_map.get(&dst)?;
                    Some((src, dst, link_rtt, src_loc, dst_loc))
                })
                .filter(|(_, _, _, src_loc, dst_loc)| {
                    src_loc.city.is_some() && dst_loc.city.is_some()
                })
                .filter_map(|(src, dst, link_rtt, src_loc, dst_loc)| {
                    (src_loc.region.as_ref()? != dst_loc.region.as_ref()?)
                        .then_some((src, dst, link_rtt, src_loc, dst_loc))
                })
                .filter_map(|(src, dst, link_rtt, src_loc, dst_loc)| {
                    let src_point = Point::new(src_loc.longitude?, src_loc.latitude?);
                    let dst_point = Point::new(dst_loc.longitude?, dst_loc.latitude?);
                    let distance = Geodesic.distance(src_point, dst_point);

                    let one_way_s = (link_rtt / 2.0) / MS_PER_S;
                    let required_speed = distance / one_way_s;

                    (required_speed <= FIBER_C).then_some((src, dst, link_rtt, src_loc, dst_loc))
                })
                .map(|(src, dst, link_rtt, _, _)| (src, dst, link_rtt))
                .collect();

            for (src, dst, rtt) in found_lhls {
                local_lhls
                    .entry(LHLKey {
                        src_addr: src,
                        dst_addr: dst,
                    })
                    .and_modify(|latency| *latency = f64::min(*latency, rtt))
                    .or_insert(rtt);
            }
        },
        |mut acc: HashMap<LHLKey, f64>, local: HashMap<LHLKey, f64>| {
            for (k, v) in local {
                acc.entry(k)
                    .and_modify(|latency| *latency = f64::min(*latency, v))
                    .or_insert(v);
            }
            acc
        },
    )
    .expect("Failed to extract LHLs");

    println!("Found {} unique LHLs.", global_lhls.len());

    println!("Writing LHLs to {}...", args.output_lhls.display());
    if let Ok(mut output) = File::create(&args.output_lhls) {
        for (key, latency) in global_lhls.iter().progress_with_style(make_style("LHLs")) {
            let res = create_record(key.src_addr, key.dst_addr, *latency, &geolocation_map)
                .and_then(|record| serde_json::to_string(&record).ok())
                .and_then(|json| writeln!(output, "{}", json).ok());
            if res.is_none() {
                eprintln!("Failed to write LHL to file");
            }
        }
    }

    println!("All done!");
    Ok(())
}

fn create_record(
    src: IpAddr,
    dst: IpAddr,
    rtt: f64,
    geolocation_map: &HashMap<IpAddr, LocationInfo>,
) -> Option<LHLRecord> {
    let src_loc = geolocation_map.get(&src)?;
    let dst_loc = geolocation_map.get(&dst)?;

    Some(LHLRecord {
        key: LHLKey {
            src_addr: src,
            dst_addr: dst,
        },
        latency_ms: rtt,
        src_loc: src_loc.clone(),
        dst_loc: dst_loc.clone(),
    })
}
