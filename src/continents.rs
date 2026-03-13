use crate::models::LocationInfo;

// ==========================================
// 1. Define the Continents
// ==========================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continent {
    Africa,
    Antarctica,
    Asia,
    Europe,
    NorthAmerica,
    Oceania,
    SouthAmerica,
}

impl Continent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Continent::Africa => "AF",
            Continent::Antarctica => "AN",
            Continent::Asia => "AS",
            Continent::Europe => "EU",
            Continent::NorthAmerica => "NA",
            Continent::Oceania => "OC",
            Continent::SouthAmerica => "SA",
        }
    }
}

// ==========================================
// 2. Fast Continent Resolution
// ==========================================
pub trait ContinentResolver {
    /// Determines the continent based on the country code.
    fn continent(&self) -> Option<Continent>;

    /// Checks if this location is in a different continent than another location.
    fn is_different_continent(&self, other: &Self) -> Option<bool>;
}

impl ContinentResolver for LocationInfo {
    fn continent(&self) -> Option<Continent> {
        let resolve_str = |s: &str| -> Option<Continent> {
            // First, see if it's already a well-known continent code or name
            match s.to_uppercase().as_str() {
                "NA" | "NORTH AMERICA" => return Some(Continent::NorthAmerica),
                "EU" | "EUROPE" | "EMEA" | "UK" | "SU" | "CRIMEA" => {
                    return Some(Continent::Europe);
                }
                "AS" | "ASIA" => return Some(Continent::Asia),
                "SA" | "SOUTH AMERICA" => return Some(Continent::SouthAmerica),
                "AF" | "AFRICA" | "AC" => return Some(Continent::Africa),
                "OC" | "OCEANIA" => return Some(Continent::Oceania),
                "AN" | "ANTARCTICA" => return Some(Continent::Antarctica),
                _ => {}
            }

            // Otherwise, use nationify to look up the country
            let country = nationify::by_country_name_or_code_case_insensitive(s);
            if let Some(c) = country {
                match c.continent_code {
                    "NA" => Some(Continent::NorthAmerica),
                    "EU" => Some(Continent::Europe),
                    "AS" => Some(Continent::Asia),
                    "SA" => Some(Continent::SouthAmerica),
                    "AF" => Some(Continent::Africa),
                    "OC" => Some(Continent::Oceania),
                    "AN" => Some(Continent::Antarctica),
                    _ => None,
                }
            } else {
                None
            }
        };

        if let Some(country) = self.country.as_deref()
            && let Some(cont) = resolve_str(country)
        {
            return Some(cont);
        }

        if let Some(region) = self.region.as_deref()
            && let Some(cont) = resolve_str(region)
        {
            return Some(cont);
        }

        None
    }

    fn is_different_continent(&self, other: &Self) -> Option<bool> {
        let my_continent = self.continent()?;
        let other_continent = other.continent()?;
        Some(my_continent != other_continent)
    }
}

pub trait CountryResolver {
    /// Determines the continent based on the country code.
    fn country(&self) -> Option<String>;
}

impl CountryResolver for LocationInfo {
    fn country(&self) -> Option<String> {
        let country = self.country.as_deref()?.trim().to_owned();

        let new_country =
            nationify::by_country_name_or_code_case_insensitive(&country).or_else(|| {
                let cty = match country.as_str() {
                    "Russia" => Some("RU"),
                    "United Kingdom" => Some("GB"),
                    "Vietnam" => Some("VN"),
                    _ => None,
                };
                cty.and_then(nationify::by_country_name_or_code_case_insensitive)
            })?;

        Some(new_country.iso_code.to_string())
    }
}
