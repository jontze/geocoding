use crate::{Float, InputBounds};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// An instance of a parameter builder for Openstreetmap geocoding
pub struct OpenstreetmapParams<'a, T>
where
    T: Float + Debug,
{
    pub query: &'a str,
    pub addressdetails: bool,
    pub viewbox: Option<&'a InputBounds<T>>,
}

impl<'a, T> OpenstreetmapParams<'a, T>
where
    T: Float + Debug,
{
    /// Create a new OpenStreetMap parameter builder
    /// # Example:
    ///
    /// ```
    /// use geocoding::{Openstreetmap, InputBounds, Point};
    /// use geocoding::openstreetmap::{OpenstreetmapParams};
    ///
    /// let viewbox = InputBounds::new(
    ///     (-0.13806939125061035, 51.51989264641164),
    ///     (-0.13427138328552246, 51.52319711775629),
    /// );
    /// let params = OpenstreetmapParams::new(&"UCL CASA")
    ///     .with_addressdetails(true)
    ///     .with_viewbox(&viewbox)
    ///     .build();
    /// ```
    pub fn new(query: &'a str) -> OpenstreetmapParams<'a, T> {
        OpenstreetmapParams {
            query,
            addressdetails: false,
            viewbox: None,
        }
    }

    /// Set the `addressdetails` property
    pub fn with_addressdetails(&mut self, addressdetails: bool) -> &mut Self {
        self.addressdetails = addressdetails;
        self
    }

    /// Set the `viewbox` property
    pub fn with_viewbox(&mut self, viewbox: &'a InputBounds<T>) -> &mut Self {
        self.viewbox = Some(viewbox);
        self
    }

    /// Build and return an instance of OpenstreetmapParams
    pub fn build(&self) -> OpenstreetmapParams<'a, T> {
        OpenstreetmapParams {
            query: self.query,
            addressdetails: self.addressdetails,
            viewbox: self.viewbox,
        }
    }
}

/// The top-level full GeoJSON response returned by a forward-geocoding request
///
/// See [the documentation](https://nominatim.org/release-docs/develop/api/Search/#geojson) for more details
///
///```json
///{
///  "type": "FeatureCollection",
///  "licence": "Data © OpenStreetMap contributors, ODbL 1.0. https://osm.org/copyright",
///  "features": [
///    {
///      "type": "Feature",
///      "properties": {
///        "place_id": 263681481,
///        "osm_type": "way",
///        "osm_id": 355421084,
///        "display_name": "68, Carrer de Calatrava, les Tres Torres, Sarrià - Sant Gervasi, Barcelona, BCN, Catalonia, 08017, Spain",
///        "place_rank": 30,
///        "category": "building",
///        "type": "apartments",
///        "importance": 0.7409999999999999,
///        "address": {
///          "house_number": "68",
///          "road": "Carrer de Calatrava",
///          "suburb": "les Tres Torres",
///          "city_district": "Sarrià - Sant Gervasi",
///          "city": "Barcelona",
///          "county": "BCN",
///          "state": "Catalonia",
///          "postcode": "08017",
///          "country": "Spain",
///          "country_code": "es"
///        }
///      },
///      "bbox": [
///        2.1284918,
///        41.401227,
///        2.128952,
///        41.4015815
///      ],
///      "geometry": {
///        "type": "Point",
///        "coordinates": [
///          2.12872241167437,
///          41.40140675
///        ]
///      }
///    }
///  ]
///}
///```
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenstreetmapResponse<T>
where
    T: Float + Debug,
{
    pub r#type: String,
    pub licence: String,
    pub features: Vec<OpenstreetmapResult<T>>,
}

/// A geocoding result
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenstreetmapResult<T>
where
    T: Float + Debug,
{
    pub r#type: String,
    pub properties: ResultProperties,
    pub bbox: (T, T, T, T),
    pub geometry: ResultGeometry<T>,
}

/// Geocoding result properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultProperties {
    pub place_id: u64,
    pub osm_type: String,
    pub osm_id: u64,
    pub display_name: String,
    pub place_rank: u64,
    pub category: String,
    pub r#type: String,
    pub importance: f64,
    pub address: Option<AddressDetails>,
}

/// Address details in the result object
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddressDetails {
    pub city: Option<String>,
    pub city_district: Option<String>,
    pub construction: Option<String>,
    pub continent: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub house_number: Option<String>,
    pub neighbourhood: Option<String>,
    pub postcode: Option<String>,
    pub public_building: Option<String>,
    pub state: Option<String>,
    pub suburb: Option<String>,
}

/// A geocoding result geometry
#[derive(Debug, Serialize, Deserialize)]
pub struct ResultGeometry<T>
where
    T: Float + Debug,
{
    pub r#type: String,
    pub coordinates: (T, T),
}
