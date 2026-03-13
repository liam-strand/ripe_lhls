use clap::Parser;
use ripe_lhls::models::GeolocateRecord;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output file for geolocation results in JSONL format
    #[arg(short, long, default_value = "geolocation_results.jsonl")]
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Reading Geolocations from {}...", args.input.display());

    let locations: Vec<GeolocateRecord> = {
        let file = File::open(&args.input)?;
        let reader = BufReader::new(file);

        reader
            .lines()
            .filter_map(|line| {
                line.ok()
                    .and_then(|l| serde_json::from_str::<GeolocateRecord>(&l).ok())
            })
            .collect()
    };

    println!("Found {} Geolocations .", locations.len());

    let mut n_cities = 0;

    for loc in locations.iter() {
        if loc.location.city.is_some() && loc.location.region.is_none() {
            n_cities += 1;
        }
    }

    println!("Found {} cities.", n_cities);

    Ok(())
}
