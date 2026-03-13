use clap::Parser;
use indicatif::ProgressIterator;
use ripe_lhls::continents::{ContinentResolver, CountryResolver};
use ripe_lhls::models::GeolocateRecord;
use ripe_lhls::progress::make_style;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
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

    let mut locations: Vec<GeolocateRecord> = {
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

    println!("Found {} valid Geolocations .", locations.len());

    let mut unknown_countries = HashSet::new();

    for loc in locations.iter_mut().progress_with_style(make_style("Locs")) {
        if let Some(country) = loc.location.country() {
            loc.location.country = Some(country);
        }
        if let Some(country) = loc.location.country.as_ref() {
            if let Some(continent) = loc.location.continent() {
                loc.location.region = Some(continent.as_str().to_string());
            } else {
                unknown_countries.insert(country);
            }
        } else if loc.location.region.is_some()
            && let Some(continent) = loc.location.continent()
        {
            loc.location.region = Some(continent.as_str().to_string());
        }
    }

    println!("Unknown countries: {}", unknown_countries.len());
    for country in unknown_countries.iter() {
        print!("{}, ", country);
    }
    println!();

    println!("Writing {} locations", locations.len());

    let output_file = File::create(&args.input)?;
    let mut writer = BufWriter::new(output_file);
    for loc in locations.into_iter() {
        writer.write_all(serde_json::to_string(&loc)?.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    println!("Done! Results written to {}", args.input.display());
    Ok(())
}
