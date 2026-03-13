use clap::Parser;
use geo::{Bearing, Geodesic};
use itertools::iproduct;
use ripe_lhls::{cable_graph::CableGraph, scn::ScnDataset};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

const KM_PER_MS: f64 = 89.0;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file for LHLs
    #[arg(short, long, default_value = "/home/yhe7443/cs445/scn/v3")]
    scns: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading data...");
    let args = Args::parse();
    let scns = ScnDataset::load_from_dir(&args.scns)?;
    println!("Building graphs...");
    let graphs = scns
        .cables
        .keys()
        .map(|id| {
            (
                id.clone(),
                CableGraph::new(&scns.cable_geometries.get(id).unwrap().geometry),
            )
        })
        .collect::<HashMap<String, CableGraph>>();

    println!("Calculating latencies...");
    let mut latencies: Vec<f64> = scns
        .cables
        .values()
        .flat_map(|c| {
            let endpoints = &c.landing_points;
            iproduct!(endpoints.iter(), endpoints.iter()).map(move |(a, b)| (c, a, b))
        })
        .filter_map(|(c, a, b)| {
            let a_cont = nationify::by_country_name_or_code_case_insensitive(&a.country)?.continent;
            let b_cont = nationify::by_country_name_or_code_case_insensitive(&b.country)?.continent;
            if a_cont == b_cont {
                return None;
            }
            Some((c, a, b))
        })
        .filter(|(c, a, b)| {
            let mut iter = c.landing_points.iter().map(|l| &l.id);
            iter.clone().any(|i| i == &a.id) && iter.any(|i| i == &b.id)
        })
        .filter_map(|(c, a, b)| {
            let a_geo = &scns.landing_point_geometries.get(&a.id)?.geometry;
            let b_geo = &scns.landing_point_geometries.get(&b.id)?.geometry;
            Some((c, a_geo, b_geo))
        })
        .filter_map(|(c, a, b)| {
            let graph = graphs.get(&c.id)?;
            graph.traverse(a, b)
        })
        .map(|d| (d / 1000.0) / KM_PER_MS)
        .collect::<Vec<_>>();

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{} SCN latencies", latencies.len());

    println!("Calculating orientations...");
    let orientations: HashMap<String, Vec<f64>> = scns
        .cables
        .values()
        .flat_map(|c| {
            let endpoints = &c.landing_points;
            iproduct!(endpoints.iter(), endpoints.iter()).map(move |(a, b)| (c, a, b))
        })
        .filter(|(_, a, b)| a.id != b.id)
        .filter_map(|(c, a, b)| {
            let a_cont =
                nationify::by_country_name_or_code_case_insensitive(&a.country)?.continent_code;
            let b_cont =
                nationify::by_country_name_or_code_case_insensitive(&b.country)?.continent_code;
            if a_cont == b_cont {
                return None;
            }
            Some((c, a_cont, a, b))
        })
        .filter(|(c, _, a, b)| {
            let mut iter = c.landing_points.iter().map(|l| &l.id);
            iter.clone().any(|i| i == &a.id) && iter.any(|i| i == &b.id)
        })
        .filter_map(|(_, a_cont, a, b)| {
            let a_geo = &scns.landing_point_geometries.get(&a.id)?.geometry;
            let b_geo = &scns.landing_point_geometries.get(&b.id)?.geometry;
            Some((a_cont, a_geo, b_geo))
        })
        .map(|(a_cont, a, b)| (a_cont, Geodesic.bearing(*a, *b)))
        .fold(HashMap::new(), |mut acc, (a_cont, bearing)| {
            acc.entry(a_cont.to_string()).or_default().push(bearing);
            acc
        });

    println!(
        "{} SCN orientations",
        orientations.values().map(|v| v.len()).sum::<usize>()
    );

    let f = File::create("scn_latency.json")?;
    let mut writer = BufWriter::new(f);
    serde_json::to_writer(&mut writer, &latencies)?;
    writer.flush()?;

    let f = File::create("scn_orientations.json")?;
    let mut writer = BufWriter::new(f);
    serde_json::to_writer(&mut writer, &orientations)?;
    writer.flush()?;

    Ok(())
}
