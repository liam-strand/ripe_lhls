use crate::models::{
    GeoJsonFeature, GeoJsonFeatureCollection, ScnCable, ScnCountry, ScnLandingPoint, ScnOwner,
    ScnReadyForService, ScnRegion, ScnRoute, ScnStatus, ScnSubregion, ScnSupplier,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ScnDataset {
    pub cables: HashMap<String, ScnCable>,
    pub landing_points: HashMap<String, ScnLandingPoint>,
    pub countries: HashMap<String, ScnCountry>,
    pub owners: HashMap<String, ScnOwner>,
    pub ready_for_service: HashMap<String, ScnReadyForService>,
    pub regions: HashMap<String, ScnRegion>,
    pub routes: HashMap<String, ScnRoute>,
    pub statuses: HashMap<String, ScnStatus>,
    pub subregions: HashMap<String, ScnSubregion>,
    pub suppliers: HashMap<String, ScnSupplier>,
    pub cable_geometries: HashMap<String, GeoJsonFeature<geo::MultiLineString>>,
    pub landing_point_geometries: HashMap<String, GeoJsonFeature<geo::Point>>,
}

impl ScnDataset {
    pub fn load_from_dir(base_path: &Path) -> std::io::Result<Self> {
        let mut cables =
            Self::load_json_dir::<ScnCable>(base_path, "cable", &["all", "cable-geo"])?;
        let mut landing_points = Self::load_json_dir::<ScnLandingPoint>(
            base_path,
            "landing-point",
            &["landing-point-geo"],
        )?;
        let mut countries = Self::load_json_dir::<ScnCountry>(base_path, "country", &["all"])?;
        let mut owners = Self::load_json_dir::<ScnOwner>(base_path, "owner", &["all", "meta"])?;
        let mut ready_for_service =
            Self::load_json_dir::<ScnReadyForService>(base_path, "ready-for-service", &["all"])?;
        let mut regions = Self::load_json_dir::<ScnRegion>(base_path, "region", &["all"])?;
        let mut routes = Self::load_json_dir::<ScnRoute>(base_path, "route", &["all"])?;
        let mut statuses = Self::load_json_dir::<ScnStatus>(base_path, "status", &["all"])?;
        let mut subregions = Self::load_json_dir::<ScnSubregion>(base_path, "subregion", &["all"])?;
        let mut suppliers = Self::load_json_dir::<ScnSupplier>(base_path, "supplier", &["all"])?;

        let mut cable_geometries = HashMap::new();
        let cable_geo_path = base_path.join("cable").join("cable-geo.json");
        if cable_geo_path.exists()
            && let Ok(content) = fs::read_to_string(&cable_geo_path)
            && let Ok(collection) =
                serde_json::from_str::<GeoJsonFeatureCollection<geo::MultiLineString>>(&content)
        {
            for feature in collection.features {
                let id = feature.properties.id.clone();
                cable_geometries.insert(id, feature);
            }
        }

        let mut landing_point_geometries = HashMap::new();
        let lp_geo_path = base_path
            .join("landing-point")
            .join("landing-point-geo.json");
        if lp_geo_path.exists()
            && let Ok(content) = fs::read_to_string(&lp_geo_path)
            && let Ok(collection) =
                serde_json::from_str::<GeoJsonFeatureCollection<geo::Point>>(&content)
        {
            for feature in collection.features {
                let id = feature.properties.id.clone();
                landing_point_geometries.insert(id, feature);
            }
        }

        cables.shrink_to_fit();
        landing_points.shrink_to_fit();
        countries.shrink_to_fit();
        owners.shrink_to_fit();
        ready_for_service.shrink_to_fit();
        regions.shrink_to_fit();
        routes.shrink_to_fit();
        statuses.shrink_to_fit();
        subregions.shrink_to_fit();
        suppliers.shrink_to_fit();
        cable_geometries.shrink_to_fit();
        landing_point_geometries.shrink_to_fit();

        Ok(Self {
            cables,
            landing_points,
            countries,
            owners,
            ready_for_service,
            regions,
            routes,
            statuses,
            subregions,
            suppliers,
            cable_geometries,
            landing_point_geometries,
        })
    }

    fn load_json_dir<T: serde::de::DeserializeOwned + HasId>(
        base_path: &Path,
        dir_name: &str,
        excludes: &[&str],
    ) -> std::io::Result<HashMap<String, T>> {
        let mut map = HashMap::new();
        let dir_path = base_path.join(dir_name);
        if dir_path.exists() {
            for entry in fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if excludes.contains(&file_stem) {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        let item = serde_json::from_str::<T>(&content)?;
                        map.insert(item.get_id(), item);
                    }
                }
            }
        }
        Ok(map)
    }
}

pub trait HasId {
    fn get_id(&self) -> String;
}

macro_rules! impl_has_id {
    ($($t:ty),+) => {
        $(impl HasId for $t {
            fn get_id(&self) -> String {
                self.id.clone()
            }
        })+
    };
}

impl_has_id!(
    ScnCable,
    ScnLandingPoint,
    ScnCountry,
    ScnOwner,
    ScnReadyForService,
    ScnRegion,
    ScnRoute,
    ScnStatus,
    ScnSubregion,
    ScnSupplier
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_dataset() {
        let base_path = Path::new("/home/yhe7443/cs445/scn/v3");
        let result = ScnDataset::load_from_dir(base_path);
        assert!(result.is_ok(), "Failed to load dataset: {:?}", result.err());
        let dataset = result.unwrap();
        assert!(!dataset.cables.is_empty(), "Cables shouldn't be empty");
        assert!(
            !dataset.landing_points.is_empty(),
            "Landing points shouldn't be empty"
        );
        assert!(dataset.cables.contains_key("2africa"));
        assert!(dataset.landing_points.contains_key("luanda-angola"));
        assert!(dataset.countries.contains_key("curaao"));
        assert!(dataset.owners.contains_key("zayo"));
        assert!(dataset.ready_for_service.contains_key("2013"));
        assert!(dataset.regions.contains_key("asia"));
        assert!(dataset.routes.contains_key("trans-atlantic"));
        assert!(dataset.statuses.contains_key("planned"));
        assert!(dataset.subregions.is_empty());
        assert!(dataset.suppliers.contains_key("nec"));
        assert!(
            dataset
                .cable_geometries
                .contains_key("asia-connect-cable-1-acc-1")
        );
        assert!(
            dataset
                .landing_point_geometries
                .contains_key("changi-south-singapore")
        );
    }
}
