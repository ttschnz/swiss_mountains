use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
enum ObjectType {
    #[serde(rename = "Huegel")]
    Huegel,
    #[serde(rename = "Haltestelle Bus")]
    HaltestelleBus,
    #[serde(rename = "Grotte, Hoehle")]
    GrotteHoehle,
    #[serde(rename = "Flurname swisstopo")]
    Flurnameswisstopo,
    #[serde(rename = "Landesgrenzstein")]
    Landesgrenzstein,
    #[serde(rename = "Zollamt eingeschraenkt")]
    Zollamteingeschraenkt,
    #[serde(rename = "Sakrales Gebaeude")]
    SakralesGebaeude,
    #[serde(rename = "Gipfel")]
    Gipfel,
    #[serde(rename = "Bildstock")]
    Bildstock,
    #[serde(rename = "Lokalname swisstopo")]
    Lokalnameswisstopo,
    #[serde(rename = "Denkmal")]
    Denkmal,
    #[serde(rename = "Alpiner Gipfel")]
    AlpinerGipfel,
    #[serde(rename = "Ausfahrt")]
    Ausfahrt,
    #[serde(rename = "Verzweigung")]
    Verzweigung,
    #[serde(rename = "Erratischer Block")]
    ErratischerBlock,
    #[serde(rename = "Haltestelle Schiff")]
    HaltestelleSchiff,
    #[serde(rename = "Ein- und Ausfahrt")]
    EinUndAusfahrt,
    #[serde(rename = "Verladestation")]
    Verladestation,
    #[serde(rename = "Hauptgipfel")]
    Hauptgipfel,
    #[serde(rename = "Pass")]
    Pass,
    #[serde(rename = "Felskopf")]
    Felskopf,
    #[serde(rename = "Offenes Gebaeude")]
    OffenesGebaeude,
    #[serde(rename = "Brunnen")]
    Brunnen,
    #[serde(rename = "Felsblock")]
    Felsblock,
    #[serde(rename = "Kapelle")]
    Kapelle,
    #[serde(rename = "Uebrige Bahnen")]
    UebrigeBahnen,
    #[serde(rename = "Gebaeude")]
    Gebaeude,
    #[serde(rename = "Strassenpass")]
    Strassenpass,
    #[serde(rename = "Turm")]
    Turm,
    #[serde(rename = "Zollamt 24h eingeschraenkt")]
    Zollamt24heingeschraenkt,
    #[serde(rename = "Zollamt 24h 24h")]
    Zollamt24h24h,
    #[serde(rename = "Wasserfall")]
    Wasserfall,
    #[serde(rename = "Haltestelle Bahn")]
    HaltestelleBahn,
    #[serde(rename = "Aussichtspunkt")]
    Aussichtspunkt,
    #[serde(rename = "Haupthuegel")]
    Haupthuegel,
    #[serde(rename = "Quelle")]
    Quelle,
}

impl ObjectType {
    pub fn is_peak(&self) -> bool {
        match self {
            Self::AlpinerGipfel | Self::Hauptgipfel /*| Self::Gipfel*/ => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
enum Status {
    #[serde(rename = "ueblich")]
    Ueblich,
    #[serde(rename = "informell")]
    Informell,
    #[serde(rename = "offiziell")]
    Offiziell,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NamedPlace {
    #[serde(rename = "OBJEKTART")]
    object_type: ObjectType,
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "STATUS")]
    status: Status,
    #[serde(rename = "E")]
    pub easting: u32,
    #[serde(rename = "N")]
    pub northing: u32,
    #[serde(rename = "Z")]
    pub altitude: u16,
}

impl NamedPlace {
    #[allow(unused)]
    pub fn distance_to(&self, other_easting: i64, other_northing: i64) -> f64 {
        (((self.easting as i64 - other_easting).wrapping_pow(2)
            + (self.northing as i64 - other_northing).wrapping_pow(2)) as f64)
            .sqrt()
    }
}

const SWISSNAMES: &str = include_str!("./swissNAMES3D_PKT.csv");

pub fn get_peaks() -> Vec<NamedPlace> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(SWISSNAMES.as_bytes());

    rdr.deserialize::<NamedPlace>()
        .filter_map(|record| {
            if let Ok(place) = record {
                if place.object_type.is_peak() && matches!(place.status, Status::Offiziell) {
                    Some(place)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_get_peak() {
        let peaks = get_peaks();
        assert_eq!(
            peaks
                .iter()
                .filter(|named_place| { named_place.name == "Niesen" })
                .count(),
            1,
            "Niesen should exist exactly 1 times"
        );
    }
}
