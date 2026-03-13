use geo::{Distance, Geodesic, Point};
use ripe_lhls::models::LHLRecord;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

const C: f64 = 299792458.0;
const MS_PER_S: f64 = 1000.0;
const FIBER_C: f64 = C * 2.0 / 3.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("/home/yhe7443/cs445/lhls/lhls.jsonl")?;
    let reader = BufReader::new(f);
    
    let out = File::create("/home/yhe7443/cs445/lhls/lhls.jsonl.tmp")?;
    let mut writer = BufWriter::new(out);

    let mut violations = 0;
    let mut kept = 0;

    for line in reader.lines() {
        let line = line?;
        if let Ok(record) = serde_json::from_str::<LHLRecord>(&line) {
            if let (Some(lon1), Some(lat1), Some(lon2), Some(lat2)) = (
                record.src_loc.longitude,
                record.src_loc.latitude,
                record.dst_loc.longitude,
                record.dst_loc.latitude,
            ) {
                let src_point = Point::new(lon1, lat1);
                let dst_point = Point::new(lon2, lat2);
                let distance = Geodesic.distance(src_point, dst_point);

                let link_rtt = record.latency_ms;
                let one_way_s = (link_rtt / 2.0) / MS_PER_S;
                let required_speed = distance / one_way_s;

                if required_speed <= FIBER_C {
                    kept += 1;
                    writer.write_all(line.as_bytes())?;
                    writer.write_all(b"\n")?;
                } else {
                    violations += 1;
                }
            }
        }
    }

    writer.flush()?;

    println!("Kept {} records, removed {} violations.", kept, violations);
    std::fs::rename("/home/yhe7443/cs445/lhls/lhls.jsonl.tmp", "/home/yhe7443/cs445/lhls/lhls.jsonl")?;
    println!("Successfully replaced lhls.jsonl");

    Ok(())
}
