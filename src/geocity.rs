use indicatif::ProgressIterator;
use maxminddb::{Reader, geoip2};
use std::{collections::HashMap, net::IpAddr, path::Path};

use crate::{
    models::{GeolocateQuery, LocationInfo},
    progress::make_style,
};

pub struct GeoCity {
    reader: Reader<Vec<u8>>,
}

impl GeoCity {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            reader: Reader::open_readfile(path).unwrap(),
        }
    }

    pub fn geolocate(&self, queries: &[GeolocateQuery]) -> HashMap<IpAddr, LocationInfo> {
        queries
            .iter()
            .progress_with_style(make_style("queries"))
            .filter_map(|query| {
                let result = self.reader.lookup(query.ip).unwrap();
                let city = result.decode::<geoip2::City>().ok().flatten()?;

                let location = LocationInfo {
                    city: city.city.names.english.map(|s| s.to_owned()),
                    state: city
                        .subdivisions
                        .first()
                        .and_then(|s| s.names.english.map(|s| s.to_owned())),
                    region: city.continent.code.map(|s| s.to_owned()),
                    country: city.country.names.english.map(|s| s.to_owned()),
                    count: 1,
                    latitude: city.location.latitude,
                    longitude: city.location.longitude,
                };

                Some((query.ip, location))
            })
            .collect()
    }
}
