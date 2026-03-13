use crate::{
    models::{GeolocateQuery, LocationInfo},
    progress::make_style,
};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, time::Duration};

#[derive(Serialize, Debug)]
struct PtrQuery {
    pub ip: IpAddr,
    pub ptr_record: String,
    pub asn: i64,
}

#[derive(Serialize, Debug)]
struct BatchPtrQuery {
    pub queries: Vec<PtrQuery>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct PtrResponse {
    pub ip: Option<IpAddr>,
    pub ptr_record: String,
    pub asn: i64,
    pub location_info: LocationInfo,
    pub regular_expression: String,
    pub geo_hint: String,
}

#[derive(Deserialize, Debug)]
struct BatchPtrResponse {
    pub responses: Vec<PtrResponse>,
}

pub struct Aleph {
    token: String,
}

impl Aleph {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    #[allow(clippy::result_large_err)]
    fn batch_query(&self, query: &BatchPtrQuery) -> Result<BatchPtrResponse, ureq::Error> {
        let raw_response = ureq::post("https://thealeph.ai/api/v2/batch_query")
            .set("Content-Type", "application/json")
            .set("accept", "application/json")
            .set("Authorization", &format!("Bearer {}", self.token))
            .timeout(Duration::from_mins(10))
            .send_json(query)?;
        Ok(raw_response.into_json::<BatchPtrResponse>()?)
    }

    // fn query(&self, query: &PtrQuery) -> Result<PtrResponse, ureq::Error> {
    //     let raw_response = ureq::post(&format!("{}/query", Self::ENDPOINT))
    //         .set("Content-Type", "application/json")
    //         .set("accept", "application/json")
    //         .set("Authorization", &format!("Bearer {}", self.token))
    //         .timeout(Duration::from_secs_f32(600))
    //         .send_json(&query)?;
    //     Ok(raw_response.into_json::<PtrResponse>()?)
    // }

    pub fn geolocate(&self, queries: &[GeolocateQuery]) -> HashMap<IpAddr, LocationInfo> {
        let ips_by_asn: HashMap<i64, Vec<(IpAddr, String)>> = queries
            .iter()
            .filter_map(|q| Some((q.ip, q.asn?, q.ptr_record.clone()?)))
            .fold(HashMap::new(), |mut acc, (ip, asn, ptr_record)| {
                acc.entry(asn).or_default().push((ip, ptr_record));
                acc
            });

        let queries = ips_by_asn
            .iter()
            .flat_map(|(asn, chunk)| {
                chunk
                    .chunks(1000)
                    .map(|sub_chunk| {
                        let queries: Vec<PtrQuery> = sub_chunk
                            .iter()
                            .map(|(ip, ptr)| PtrQuery {
                                ip: *ip,
                                ptr_record: ptr.clone(),
                                asn: *asn,
                            })
                            .collect();
                        BatchPtrQuery { queries }
                    })
                    .collect::<Vec<BatchPtrQuery>>()
            })
            .collect::<Vec<BatchPtrQuery>>();

        queries
            .into_par_iter()
            .progress_with_style(make_style("queries"))
            .map(|query| self.batch_query(&query))
            .flatten()
            .map(|response| response.responses)
            .flatten()
            .filter_map(|response| Some((response.ip?, response.location_info)))
            .collect()
    }
}
