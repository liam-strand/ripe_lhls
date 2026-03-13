use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::net::IpAddr;

#[derive(Deserialize, Debug)]
pub struct TracerouteResult {
    pub af: Option<i32>,
    pub bundle: Option<i32>,
    pub dst_addr: Option<IpAddr>,
    pub dst_name: Option<String>,
    pub endtime: Option<i64>,
    pub from: Option<IpAddr>,
    pub group_id: Option<i32>,
    pub lts: Option<i32>,
    pub msm_id: Option<i32>,
    pub msm_name: Option<String>,
    pub paris_id: Option<i32>,
    pub prb_id: Option<i32>,
    pub proto: Option<String>,
    pub destination_ip_responded: Option<bool>,
    pub result: Option<Vec<TracerouteHop>>,
    pub size: Option<i32>,
    pub src_addr: Option<IpAddr>,
    pub timestamp: Option<i64>,
    pub tos: Option<i32>,
    pub ttr: Option<f64>,
    #[serde(rename = "type")]
    pub result_type: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TracerouteHop {
    pub hop: Option<i32>,
    pub error: Option<String>,
    pub result: Option<Vec<TracerouteReply>>,
}

#[derive(Deserialize, Debug)]
pub struct TracerouteReply {
    pub x: Option<String>,
    pub err: Option<Value>,
    pub from: Option<IpAddr>,
    pub itos: Option<i32>,
    pub ittl: Option<i32>,
    pub edst: Option<IpAddr>,
    pub late: Option<i32>,
    pub mtu: Option<i32>,
    pub rtt: Option<f64>,
    pub size: Option<i32>,
    pub ttl: Option<i32>,
    pub flags: Option<String>,
    pub dstoptsize: Option<i32>,
    pub hbhoptsize: Option<i32>,
    pub icmpext: Option<IcmpExt>,
}

#[derive(Deserialize, Debug)]
pub struct IcmpExt {
    pub version: Option<i32>,
    pub rfc4884: Option<i32>,
    pub obj: Option<Vec<IcmpExtObj>>,
}

#[derive(Deserialize, Debug)]
pub struct IcmpExtObj {
    pub class: Option<i32>,
    #[serde(rename = "type")]
    pub obj_type: Option<i32>,
    pub mpls: Option<Vec<MplsData>>,
}

#[derive(Deserialize, Debug)]
pub struct MplsData {
    pub exp: Option<i32>,
    pub label: Option<i32>,
    pub s: Option<i32>,
    pub ttl: Option<i32>,
}

pub struct GeolocateQuery {
    pub ip: IpAddr,
    pub asn: Option<i64>,
    pub ptr_record: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LocationInfo {
    pub city: Option<String>,
    pub state: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub count: i64,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct HostnameRecord {
    pub ip: String,
    pub asn: Option<i64>,
    pub hostname: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GeolocateRecord {
    pub ip: String,
    pub hostname: Option<String>,
    pub location: LocationInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct LHLKey {
    pub src_addr: IpAddr,
    pub dst_addr: IpAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LHLRecord {
    pub key: LHLKey,
    pub latency_ms: f64,
    pub src_loc: LocationInfo,
    pub dst_loc: LocationInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnEntityId {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnCableShort {
    pub id: String,
    pub name: String,
    pub rfs_year: Option<i32>,
    pub is_planned: Option<bool>,
    pub landing_points: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnLandingPointShort {
    pub id: String,
    pub name: String,
    pub country: String,
    pub is_tbd: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnCable {
    pub id: String,
    pub name: String,
    pub length: Option<String>,
    pub landing_points: Vec<ScnLandingPointShort>,
    pub owners: Option<String>,
    pub suppliers: Option<String>,
    pub rfs: Option<String>,
    pub rfs_year: Option<i32>,
    pub is_planned: Option<bool>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub profiled: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnLandingPoint {
    pub id: String,
    pub name: String,
    pub country: String,
    pub cables: Vec<ScnCableShort>,
    pub landing_points: Vec<String>,
}

// Below models hold a name + list of related IDs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnCountry {
    pub id: String,
    pub name: String,
    pub cables: Option<Vec<ScnCableShort>>,
    pub landing_points: Option<Vec<String>>,
    pub landing_points_in_country: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnOwner {
    pub id: String,
    pub name: String,
    pub cables: Option<Vec<ScnCableShort>>,
    pub landing_points: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnReadyForService {
    pub id: String,
    pub name: String,
    pub cables: Option<Vec<ScnCableShort>>,
    pub landing_points: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnRegion {
    pub id: String,
    pub name: String,
    pub cables: Option<Vec<ScnCableShort>>,
    pub landing_points: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnRoute {
    pub id: String,
    pub name: String,
    pub cables: Option<Vec<ScnCableShort>>,
    pub landing_points: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnStatus {
    pub id: String,
    pub name: String,
    pub cables: Option<Vec<ScnCableShort>>,
    pub landing_points: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnSubregion {
    pub id: String,
    pub name: String,
    pub cables: Option<Vec<ScnCableShort>>,
    pub landing_points: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScnSupplier {
    pub id: String,
    pub name: String,
    pub cables: Option<Vec<ScnCableShort>>,
    pub landing_points: Option<Vec<String>>,
}

// GeoJSON Models
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum GeoJsonGeometry {
    Point { coordinates: (f64, f64) },
    LineString { coordinates: Vec<(f64, f64)> },
    MultiLineString { coordinates: Vec<Vec<(f64, f64)>> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeoJsonProperties {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_tbd: Option<bool>,
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeoJsonFeature<T: FromGeoJsonGeometry> {
    #[serde(rename = "type")]
    pub feat_type: String,
    pub properties: GeoJsonProperties,

    #[serde(deserialize_with = "deserialize_geojson_feature")]
    pub geometry: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeoJsonFeatureCollection<T: FromGeoJsonGeometry> {
    #[serde(rename = "type")]
    pub feat_type: String,
    pub features: Vec<GeoJsonFeature<T>>,
}

pub trait FromGeoJsonGeometry: Sized {
    fn from_geometry(geom: GeoJsonGeometry) -> Result<Self, String>;
}

impl FromGeoJsonGeometry for geo::Point<f64> {
    fn from_geometry(geom: GeoJsonGeometry) -> Result<Self, String> {
        match geom {
            GeoJsonGeometry::Point { coordinates } => {
                Ok(geo::Point::new(coordinates.0, coordinates.1))
            }
            _ => Err("Expected Point geometry".into()),
        }
    }
}

impl FromGeoJsonGeometry for geo::LineString<f64> {
    fn from_geometry(geom: GeoJsonGeometry) -> Result<Self, String> {
        match geom {
            GeoJsonGeometry::LineString { coordinates } => {
                let coords = coordinates
                    .into_iter()
                    .map(|(x, y)| geo::coord! { x: x, y: y })
                    .collect();
                Ok(geo::LineString::new(coords))
            }
            _ => Err("Expected LineString geometry".into()),
        }
    }
}

impl FromGeoJsonGeometry for geo::MultiLineString<f64> {
    fn from_geometry(geom: GeoJsonGeometry) -> Result<Self, String> {
        match geom {
            GeoJsonGeometry::MultiLineString { coordinates } => {
                let lines = coordinates
                    .into_iter()
                    .map(|line| {
                        let coords = line
                            .into_iter()
                            .map(|(x, y)| geo::coord! { x: x, y: y })
                            .collect();
                        geo::LineString::new(coords)
                    })
                    .collect();
                Ok(geo::MultiLineString::new(lines))
            }
            _ => Err("Expected MultiLineString geometry".into()),
        }
    }
}

fn deserialize_geojson_feature<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromGeoJsonGeometry,
{
    let geom = GeoJsonGeometry::deserialize(d)?;
    T::from_geometry(geom).map_err(serde::de::Error::custom)
}
