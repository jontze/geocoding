use crate::chrono::naive::serde::ts_seconds::deserialize as from_ts;
use crate::chrono::NaiveDateTime;
use crate::InputBounds;
use crate::{Deserialize, Serialize};
use num_traits::Float;
use serde::Deserializer;
use std::collections::HashMap;

macro_rules! add_optional_param {
    ($query:expr, $param:expr, $name:expr) => {
        if let Some(p) = $param {
            $query.push(($name, p))
        }
    };
}

// Please see the [API documentation](https://opencagedata.com/api#forward-opt) for details.
#[derive(Default)]
pub struct Parameters<'a> {
    pub language: Option<&'a str>,
    pub countrycode: Option<&'a str>,
    pub limit: Option<&'a str>,
}

impl<'a> Parameters<'a> {
    pub fn as_query(&self) -> Vec<(&'a str, &'a str)> {
        let mut query = vec![];
        add_optional_param!(query, self.language, "language");
        add_optional_param!(query, self.countrycode, "countrycode");
        add_optional_param!(query, self.limit, "limit");
        query
    }
}

pub fn deserialize_string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i32),
    }

    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::String(s) => Ok(s),
        StringOrInt::Int(i) => Ok(i.to_string()),
    }
}

// OpenCage has a custom rate-limit header, indicating remaining calls
// header! { (XRatelimitRemaining, "X-RateLimit-Remaining") => [i32] }
pub static XRL: &str = "x-ratelimit-remaining";
/// Use this constant if you don't need to restrict a `forward_full` call with a bounding box
pub static NOBOX: Option<InputBounds<f64>> = None::<InputBounds<f64>>;

/// The top-level full JSON response returned by a forward-geocoding request
///
/// See [the documentation](https://opencagedata.com/api#response) for more details
///
///```json
/// {
///   "documentation": "https://opencagedata.com/api",
///   "licenses": [
///     {
///       "name": "CC-BY-SA",
///       "url": "http://creativecommons.org/licenses/by-sa/3.0/"
///     },
///     {
///       "name": "ODbL",
///       "url": "http://opendatacommons.org/licenses/odbl/summary/"
///     }
///   ],
///   "rate": {
///     "limit": 2500,
///     "remaining": 2499,
///     "reset": 1523318400
///   },
///   "results": [
///     {
///       "annotations": {
///         "DMS": {
///           "lat": "41° 24' 5.06412'' N",
///           "lng": "2° 7' 43.40064'' E"
///         },
///         "MGRS": "31TDF2717083684",
///         "Maidenhead": "JN11bj56ki",
///         "Mercator": {
///           "x": 236968.295,
///           "y": 5043465.71
///         },
///         "OSM": {
///           "edit_url": "https://www.openstreetmap.org/edit?way=355421084#map=17/41.40141/2.12872",
///           "url": "https://www.openstreetmap.org/?mlat=41.40141&mlon=2.12872#map=17/41.40141/2.12872"
///         },
///         "callingcode": 34,
///         "currency": {
///           "alternate_symbols": [
///
///           ],
///           "decimal_mark": ",",
///           "html_entity": "&#x20AC;",
///           "iso_code": "EUR",
///           "iso_numeric": 978,
///           "name": "Euro",
///           "smallest_denomination": 1,
///           "subunit": "Cent",
///           "subunit_to_unit": 100,
///           "symbol": "€",
///           "symbol_first": 1,
///           "thousands_separator": "."
///         },
///         "flag": "🇪🇸",
///         "geohash": "sp3e82yhdvd7p5x1mbdv",
///         "qibla": 110.53,
///         "sun": {
///           "rise": {
///             "apparent": 1523251260,
///             "astronomical": 1523245440,
///             "civil": 1523249580,
///             "nautical": 1523247540
///           },
///           "set": {
///             "apparent": 1523298360,
///             "astronomical": 1523304180,
///             "civil": 1523300040,
///             "nautical": 1523302080
///           }
///         },
///         "timezone": {
///           "name": "Europe/Madrid",
///           "now_in_dst": 1,
///           "offset_sec": 7200,
///           "offset_string": 200,
///           "short_name": "CEST"
///         },
///         "what3words": {
///           "words": "chins.pictures.passes"
///         }
///       },
///       "bounds": {
///         "northeast": {
///           "lat": 41.4015815,
///           "lng": 2.128952
///         },
///         "southwest": {
///           "lat": 41.401227,
///           "lng": 2.1284918
///         }
///       },
///       "components": {
///         "ISO_3166-1_alpha-2": "ES",
///         "_type": "building",
///         "city": "Barcelona",
///         "city_district": "Sarrià - Sant Gervasi",
///         "country": "Spain",
///         "country_code": "es",
///         "county": "BCN",
///         "house_number": "68",
///         "political_union": "European Union",
///         "postcode": "08017",
///         "road": "Carrer de Calatrava",
///         "state": "Catalonia",
///         "suburb": "les Tres Torres"
///       },
///       "confidence": 10,
///       "formatted": "Carrer de Calatrava, 68, 08017 Barcelona, Spain",
///       "geometry": {
///         "lat": 41.4014067,
///         "lng": 2.1287224
///       }
///     }
///   ],
///   "status": {
///     "code": 200,
///     "message": "OK"
///   },
///   "stay_informed": {
///     "blog": "https://blog.opencagedata.com",
///     "twitter": "https://twitter.com/opencagedata"
///   },
///   "thanks": "For using an OpenCage Data API",
///   "timestamp": {
///     "created_http": "Mon, 09 Apr 2018 12:33:01 GMT",
///     "created_unix": 1523277181
///   },
///   "total_results": 1
/// }
///```
#[derive(Debug, Serialize, Deserialize)]
pub struct OpencageResponse<T>
where
    T: Float,
{
    pub documentation: String,
    pub licenses: Vec<HashMap<String, String>>,
    pub rate: Option<HashMap<String, i32>>,
    pub results: Vec<Results<T>>,
    pub status: Status,
    pub stay_informed: HashMap<String, String>,
    pub thanks: String,
    pub timestamp: Timestamp,
    pub total_results: i32,
}

/// A forward geocoding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Results<T>
where
    T: Float,
{
    pub annotations: Option<Annotations<T>>,
    pub bounds: Option<Bounds<T>>,
    pub components: HashMap<String, String>,
    pub confidence: i8,
    pub formatted: String,
    pub geometry: HashMap<String, T>,
}

/// Annotations pertaining to the geocoding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotations<T>
where
    T: Float,
{
    pub dms: Option<HashMap<String, String>>,
    pub mgrs: Option<String>,
    pub maidenhead: Option<String>,
    pub mercator: Option<HashMap<String, T>>,
    pub osm: Option<HashMap<String, String>>,
    pub callingcode: i16,
    pub currency: Currency,
    pub flag: String,
    pub geohash: String,
    pub qibla: T,
    pub sun: Sun,
    pub timezone: Timezone,
    pub what3words: HashMap<String, String>,
}

/// Currency metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub alternate_symbols: Option<Vec<String>>,
    pub decimal_mark: String,
    pub html_entity: String,
    pub iso_code: String,
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub iso_numeric: String,
    pub name: String,
    pub smallest_denomination: i16,
    pub subunit: String,
    pub subunit_to_unit: i16,
    pub symbol: String,
    pub symbol_first: i16,
    pub thousands_separator: String,
}

/// Sunrise and sunset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sun {
    pub rise: HashMap<String, i64>,
    pub set: HashMap<String, i64>,
}

/// Timezone metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timezone {
    pub name: String,
    pub now_in_dst: i16,
    pub offset_sec: i32,
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub offset_string: String,
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub short_name: String,
}

/// HTTP status metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub message: String,
    pub code: i16,
}

/// Timestamp metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct Timestamp {
    pub created_http: String,
    #[serde(deserialize_with = "from_ts")]
    pub created_unix: NaiveDateTime,
}

/// Bounding-box metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounds<T>
where
    T: Float,
{
    pub northeast: HashMap<String, T>,
    pub southwest: HashMap<String, T>,
}
