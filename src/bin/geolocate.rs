use clap::Parser;
#[allow(unused_imports)]
use dns_lookup::lookup_addr;
use indicatif::{ParallelProgressIterator, ProgressIterator};
use ripe_lhls::aleph::Aleph;
use ripe_lhls::geocity::GeoCity;
use ripe_lhls::models::{GeolocateQuery, GeolocateRecord, HostnameRecord, LocationInfo};
use ripe_lhls::progress::make_style;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File containing unique IP addresses
    #[arg(short, long, default_value = "unique_ips.txt")]
    input: PathBuf,

    /// Output file for geolocation results in JSONL format
    #[arg(short, long, default_value = "geolocation_results.jsonl")]
    output: PathBuf,

    /// Output file for hostname results in JSONL format
    #[arg(long, default_value = "hostname_results.jsonl")]
    hostnames_output: PathBuf,

    /// Batch size for API requests
    #[arg(short, long, default_value_t = 1000)]
    batch_size: usize,

    /// Number of concurrent threads for lookups
    #[arg(short, long, default_value_t = 256)]
    threads: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();

    println!("Reading IPs from {}...", args.input.display());
    let file = File::open(&args.input)?;
    let reader = BufReader::new(file);

    let mut ips = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if let Ok(ip) = line.trim().parse::<IpAddr>() {
            ips.push(ip);
        }
    }

    println!("Found {} valid IP addresses.", ips.len());
    println!("Reading ASN and hostname for each IP address...");

    let hostnames_file = File::open(&args.hostnames_output)?;

    let host_map: HashMap<IpAddr, (i64, String)> = BufReader::new(hostnames_file)
        .lines()
        .map_while(|e| e.ok())
        .filter_map(|e| serde_json::from_str::<HostnameRecord>(&e).ok())
        .filter_map(|e| Some((IpAddr::from_str(&e.ip).ok()?, (e.asn?, e.hostname?))))
        .collect();

    println!("Found {} hostname mappings", host_map.len());
    println!("Generating queries");

    let queries: Vec<GeolocateQuery> = ips
        .par_iter()
        .progress_with_style(make_style("IPs"))
        .map(|&ip| {
            let entry = host_map.get(&ip);
            GeolocateQuery {
                ip,
                asn: entry.map(|e| e.0),
                ptr_record: entry.map(|e| e.1.clone()),
            }
        })
        .collect();

    let mut locations: HashMap<IpAddr, LocationInfo> = HashMap::new();

    println!("Sending queries to TheAleph API...");

    let aleph_locations = Aleph::new("aqualab".to_owned()).geolocate(&queries);

    println!("Found {} locations in TheAleph.", aleph_locations.len());

    for (ip, location) in aleph_locations
        .into_iter()
        .progress_with_style(make_style("insertions"))
    {
        locations.insert(ip, location);
    }

    println!("Sending queries to GeoCity");

    let geocity_locations =
        GeoCity::new("/home/yhe7443/cs445/GeoLite2-City.mmdb").geolocate(&queries);

    println!("Found {} locations in GeoCity.", geocity_locations.len());

    for (ip, location) in geocity_locations
        .into_iter()
        .progress_with_style(make_style("insertions"))
    {
        locations.entry(ip).or_insert(location);
    }

    println!("Writing {} locations", locations.len());

    let output_file = File::create(&args.output)?;
    let mut writer = BufWriter::new(output_file);
    for (ip, loc) in locations.into_iter() {
        let record = GeolocateRecord {
            ip: ip.to_string(),
            hostname: host_map.get(&ip).map(|e| e.1.clone()),
            location: loc,
        };
        writer.write_all(serde_json::to_string(&record)?.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    println!("Done! Results written to {}", args.output.display());
    Ok(())
}

#[allow(dead_code)]
fn get_asn(ip: &IpAddr) -> Option<i64> {
    let output = std::process::Command::new("timeout")
        .arg("1")
        .arg("whois")
        .arg("-h")
        .arg("whois.cymru.com")
        .arg(ip.to_string())
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().nth(1)?.trim();
        if line.is_empty() {
            return None;
        }

        let mut parts = line.split('|');
        let asn_str = parts.next()?;
        let asn = asn_str.trim().parse::<i64>().ok()?;
        return Some(asn);
    }
    None
}
