use crate::Deserialize;
use crate::InputBounds;
use crate::Point;
use num_traits::{Float, Pow};

/// An instance of a parameter builder for GeoAdmin geocoding
pub struct GeoAdminParams<'a, T>
where
    T: Float,
{
    pub searchtext: &'a str,
    pub origins: &'a str,
    pub bbox: Option<&'a InputBounds<T>>,
    pub limit: Option<u8>,
}

impl<'a, T> GeoAdminParams<'a, T>
where
    T: Float,
{
    /// Create a new GeoAdmin parameter builder
    /// # Example:
    ///
    /// ```
    /// use geocoding::{GeoAdmin, InputBounds, Point};
    /// use geocoding::geoadmin::{GeoAdminParams};
    ///
    /// let bbox = InputBounds::new(
    ///     (7.4513398, 46.92792859),
    ///     (7.4513662, 46.9279467),
    /// );
    /// let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
    ///     .with_origins("address")
    ///     .with_bbox(&bbox)
    ///     .build();
    /// ```
    pub fn new(searchtext: &'a str) -> GeoAdminParams<'a, T> {
        GeoAdminParams {
            searchtext,
            origins: "zipcode,gg25,district,kantone,gazetteer,address,parcel",
            bbox: None,
            limit: Some(50),
        }
    }

    /// Set the `origins` property
    pub fn with_origins(&mut self, origins: &'a str) -> &mut Self {
        self.origins = origins;
        self
    }

    /// Set the `bbox` property
    pub fn with_bbox(&mut self, bbox: &'a InputBounds<T>) -> &mut Self {
        self.bbox = Some(bbox);
        self
    }

    /// Set the `limit` property
    pub fn with_limit(&mut self, limit: u8) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    /// Build and return an instance of GeoAdminParams
    pub fn build(&self) -> GeoAdminParams<'a, T> {
        GeoAdminParams {
            searchtext: self.searchtext,
            origins: self.origins,
            bbox: self.bbox,
            limit: self.limit,
        }
    }
}

// Approximately transform Point from WGS84 to LV03
//
// See [the documentation](https://www.swisstopo.admin.ch/content/swisstopo-internet/en/online/calculation-services/_jcr_content/contentPar/tabs/items/documents_publicatio/tabPar/downloadlist/downloadItems/19_1467104393233.download/ch1903wgs84_e.pdf) for more details
pub fn wgs84_to_lv03<T>(p: &Point<T>) -> Point<T>
where
    T: Float,
{
    let lambda = (p.x().to_f64().unwrap() * 3600.0 - 26782.5) / 10000.0;
    let phi = (p.y().to_f64().unwrap() * 3600.0 - 169028.66) / 10000.0;
    let x = 2600072.37 + 211455.93 * lambda
        - 10938.51 * lambda * phi
        - 0.36 * lambda * phi.pow(2)
        - 44.54 * lambda.pow(3);
    let y = 1200147.07 + 308807.95 * phi + 3745.25 * lambda.pow(2) + 76.63 * phi.pow(2)
        - 194.56 * lambda.pow(2) * phi
        + 119.79 * phi.pow(3);
    Point::new(
        T::from(x - 2000000.0).unwrap(),
        T::from(y - 1000000.0).unwrap(),
    )
}
/// The top-level full JSON (GeoJSON Feature Collection) response returned by a forward-geocoding request
///
/// See [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search) for more details
///
///```json
///{
///     "type": "FeatureCollection",
///     "features": [
///         {
///             "properties": {
///                 "origin": "address",
///                 "geom_quadindex": "021300220302203002031",
///                 "weight": 1512,
///                 "zoomlevel": 10,
///                 "lon": 7.451352119445801,
///                 "detail": "seftigenstrasse 264 3084 wabern 355 koeniz ch be",
///                 "rank": 7,
///                 "lat": 46.92793655395508,
///                 "num": 264,
///                 "y": 2600968.75,
///                 "x": 1197427.0,
///                 "label": "Seftigenstrasse 264 <b>3084 Wabern</b>"
///                 "id": 1420809,
///             }
///         }
///     ]
/// }
///```
#[derive(Debug, Deserialize)]
pub struct GeoAdminForwardResponse<T>
where
    T: Float,
{
    pub features: Vec<GeoAdminForwardLocation<T>>,
}

/// A forward geocoding location
#[derive(Debug, Deserialize)]
pub struct GeoAdminForwardLocation<T>
where
    T: Float,
{
    id: Option<usize>,
    pub properties: ForwardLocationProperties<T>,
}

/// Forward Geocoding location attributes
#[derive(Clone, Debug, Deserialize)]
pub struct ForwardLocationProperties<T> {
    pub origin: String,
    pub geom_quadindex: String,
    pub weight: u32,
    pub rank: u32,
    pub detail: String,
    pub lat: T,
    pub lon: T,
    pub num: Option<usize>,
    pub x: T,
    pub y: T,
    pub label: String,
    pub zoomlevel: u32,
}

/// The top-level full JSON (GeoJSON FeatureCollection) response returned by a reverse-geocoding request
///
/// See [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#identify-features) for more details
///
///```json
/// {
///     "results": [
///         {
///             "type": "Feature"
///             "id": "1272199_0"
///             "attributes": {
///                 "xxx": "xxx",
///                 "...": "...",
///             },
///             "layerBodId": "ch.bfs.gebaeude_wohnungs_register",
///             "layerName": "Register of Buildings and Dwellings",
///         }
///     ]
/// }
///```
#[derive(Debug, Deserialize)]
pub struct GeoAdminReverseResponse {
    pub results: Vec<GeoAdminReverseLocation>,
}

/// A reverse geocoding result
#[derive(Debug, Deserialize)]
pub struct GeoAdminReverseLocation {
    id: String,
    #[serde(rename = "featureId")]
    pub feature_id: String,
    #[serde(rename = "layerBodId")]
    pub layer_bod_id: String,
    #[serde(rename = "layerName")]
    pub layer_name: String,
    pub properties: ReverseLocationAttributes,
}

/// Reverse geocoding result attributes
#[derive(Clone, Debug, Deserialize)]
pub struct ReverseLocationAttributes {
    pub egid: Option<String>,
    pub ggdenr: u32,
    pub ggdename: String,
    pub gdekt: String,
    pub edid: Option<String>,
    pub egaid: u32,
    pub deinr: Option<String>,
    pub dplz4: u32,
    pub dplzname: String,
    pub egrid: Option<String>,
    pub esid: u32,
    pub strname: Vec<String>,
    pub strsp: Vec<String>,
    pub strname_deinr: String,
    pub label: String,
}
