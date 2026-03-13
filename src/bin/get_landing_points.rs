use clap::Parser;
use geo::{Distance, Geodesic};
use indicatif::ParallelProgressIterator;
use ripe_lhls::{models::LHLRecord, progress::make_style, scn::ScnDataset};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

#[derive(serde::Serialize)]
struct DistancesOutput {
    nearside: Vec<u64>,
    farside: Vec<u64>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file for LHLs
    #[arg(short, long, default_value = "/home/yhe7443/cs445/scn/v3")]
    scns: PathBuf,
    /// Input file for LHLs
    #[arg(short, long, default_value = "/home/yhe7443/cs445/lhls/lhls.jsonl")]
    lhls: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let lhls: Vec<LHLRecord> = {
        let f = File::open(&args.lhls)?;
        let reader = BufReader::new(f);
        reader
            .lines()
            .map_while(|e| e.ok())
            .filter_map(|e| serde_json::from_str::<LHLRecord>(&e).ok())
            .collect()
    };

    let scns = ScnDataset::load_from_dir(&args.scns)?;

    let landing_points = scns
        .landing_points
        .keys()
        .filter_map(|k| Some(scns.landing_point_geometries.get(k)?.geometry))
        .collect::<Vec<geo::Point>>();

    let nearside_routers = lhls
        .iter()
        .map(|lhl| &lhl.src_loc)
        .filter_map(|loc| Some(geo::Point::new(loc.longitude?, loc.latitude?)))
        .collect::<Vec<geo::Point>>();

    let farside_routers = lhls
        .iter()
        .map(|lhl| &lhl.dst_loc)
        .filter_map(|loc| Some(geo::Point::new(loc.longitude?, loc.latitude?)))
        .collect::<Vec<geo::Point>>();

    eprintln!("Found {} landing points.", landing_points.len());
    eprintln!("Found {} nearside routers.", nearside_routers.len());
    eprintln!("Found {} farside routers.", farside_routers.len());

    let nearside_distances = nearside_routers
        .par_iter()
        .progress_with_style(make_style("Routers"))
        .map(|r| get_distance_to_closest_landing_point(r, &landing_points))
        .collect::<Vec<_>>();
    let farside_distances = farside_routers
        .par_iter()
        .progress_with_style(make_style("Routers"))
        .map(|r| get_distance_to_closest_landing_point(r, &landing_points))
        .collect::<Vec<_>>();

    let mut farside = farside_distances
        .into_iter()
        .map(|e| e / 1000000)
        .collect::<Vec<_>>();
    let mut nearside = nearside_distances
        .into_iter()
        .map(|e| e / 1000000)
        .collect::<Vec<_>>();

    nearside.sort();
    farside.sort();

    {
        let f = File::create("distances.json")?;
        let mut writer = BufWriter::new(f);
        serde_json::to_writer(&mut writer, &DistancesOutput { nearside, farside })?;
        writer.flush()?;
    }

    Ok(())
}

fn get_distance_to_closest_landing_point(point: &geo::Point, landing_points: &[geo::Point]) -> u64 {
    landing_points
        .iter()
        .map(|p| Geodesic.distance(*p, *point))
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .map(|e| e * 1000000.0)
        .unwrap() as u64
}
