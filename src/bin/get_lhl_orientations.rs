use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use geo::{Bearing, Geodesic, Point};
use ripe_lhls::models::LHLRecord;

const LHL_PATH: &str = "/home/yhe7443/cs445/lhls/lhls.jsonl";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading LHLs from {}...", LHL_PATH);

    let lhl_records = {
        let f = File::open(LHL_PATH)?;
        let buf_reader = BufReader::new(f);
        buf_reader
            .lines()
            .map_while(|l| l.ok())
            .filter_map(|l| serde_json::from_str::<LHLRecord>(&l).ok())
            .collect::<Vec<_>>()
    };

    println!("Found {} LHL records", lhl_records.len());

    let orientations: HashMap<String, Vec<f64>> = lhl_records
        .iter()
        .filter_map(|l| {
            let src_cont = l.src_loc.region.clone()?;
            let src_loc = Point::new(l.src_loc.longitude?, l.src_loc.latitude?);
            let dst_loc = Point::new(l.dst_loc.longitude?, l.dst_loc.latitude?);
            let bearing = Geodesic.bearing(src_loc, dst_loc);
            Some((src_cont, bearing))
        })
        .fold(HashMap::new(), |mut acc, (a, b)| {
            acc.entry(a).or_default().push(b);
            acc
        });

    println!("Writing orientations to lhl_orientations.json");

    {
        let f = File::create("lhl_orientations.json")?;
        let mut writer = BufWriter::new(f);
        serde_json::to_writer(&mut writer, &orientations)?;
        writer.flush()?;
    }

    Ok(())
}
