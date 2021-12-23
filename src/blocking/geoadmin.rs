//! The [GeoAdmin](https://api3.geo.admin.ch) provider for geocoding in Switzerland exclusively.
//!
//! Based on the [Search API](https://api3.geo.admin.ch/services/sdiservices.html#search)
//! and [Identify Features API](https://api3.geo.admin.ch/services/sdiservices.html#identify-features)
//!
//! While GeoAdmin API is free, please respect their fair usage policy.
//!
//! ### Example
//!
//! ```
//! use geocoding::{GeoAdmin, Forward, Point};
//!
//! let geoadmin = GeoAdmin::new();
//! let address = "Seftigenstrasse 264, 3084 Wabern";
//! let res = geoadmin.forward(&address);
//! assert_eq!(res.unwrap(), vec![Point::new(7.451352119445801, 46.92793655395508)]);
//! ```
use crate::blocking::{Forward, Reverse};
use crate::geoadmin::{
    wgs84_to_lv03, GeoAdminForwardResponse, GeoAdminParams, GeoAdminReverseResponse,
};
use crate::{Deserialize, GeocodingError, InputBounds, Point};
use crate::{HeaderMap, HeaderValue, UA_STRING, USER_AGENT};
use num_traits::Float;

/// An instance of the GeoAdmin geocoding service
pub struct GeoAdmin {
    client: reqwest::blocking::Client,
    endpoint: String,
    sr: String,
}

impl GeoAdmin {
    /// Create a new GeoAdmin geocoding instance using the default endpoint and sr
    pub fn new() -> Self {
        GeoAdmin::default()
    }

    /// Set a custom endpoint of a GeoAdmin geocoding instance
    ///
    /// Endpoint should include a trailing slash (i.e. "https://api3.geo.admin.ch/rest/services/api/")
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_owned();
        self
    }

    /// Set a custom sr of a GeoAdmin geocoding instance
    ///
    /// Supported values: 21781 (LV03), 2056 (LV95), 4326 (WGS84) and 3857 (Web Pseudo-Mercator)
    pub fn with_sr(mut self, sr: &str) -> Self {
        self.sr = sr.to_owned();
        self
    }

    /// A forward-geocoding search of a location, returning a full detailed response
    ///
    /// Accepts an [`GeoAdminParams`](struct.GeoAdminParams.html) struct for specifying
    /// options, including what origins to response and whether to filter
    /// by a bounding box.
    ///
    /// Please see [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search) for details.
    ///
    /// This method passes the `format` parameter to the API.
    ///
    /// # Examples
    ///
    /// ```
    /// use geocoding::{GeoAdmin, InputBounds, Point};
    /// use geocoding::geoadmin::{GeoAdminParams, GeoAdminForwardResponse};
    ///
    /// let geoadmin = GeoAdmin::new();
    /// let bbox = InputBounds::new(
    ///     (7.4513398, 46.92792859),
    ///     (7.4513662, 46.9279467),
    /// );
    /// let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
    ///     .with_origins("address")
    ///     .with_bbox(&bbox)
    ///     .build();
    /// let res: GeoAdminForwardResponse<f64> = geoadmin.forward_full(&params).unwrap();
    /// let result = &res.features[0];
    /// assert_eq!(
    ///     result.properties.label,
    ///     "Seftigenstrasse 264 <b>3084 Wabern</b>",
    /// );
    /// ```
    pub fn forward_full<T>(
        &self,
        params: &GeoAdminParams<T>,
    ) -> Result<GeoAdminForwardResponse<T>, GeocodingError>
    where
        T: Float,
        for<'de> T: Deserialize<'de>,
    {
        // For lifetime issues
        let bbox;
        let limit;

        let mut query = vec![
            ("searchText", params.searchtext),
            ("type", "locations"),
            ("origins", params.origins),
            ("sr", &self.sr),
            ("geometryFormat", "geojson"),
        ];

        if let Some(bb) = params.bbox.cloned().as_mut() {
            if vec!["4326", "3857"].contains(&self.sr.as_str()) {
                *bb = InputBounds::new(
                    wgs84_to_lv03(&bb.minimum_lonlat),
                    wgs84_to_lv03(&bb.maximum_lonlat),
                );
            }
            bbox = String::from(*bb);
            query.push(("bbox", &bbox));
        }

        if let Some(lim) = params.limit {
            limit = lim.to_string();
            query.push(("limit", &limit));
        }

        let resp = self
            .client
            .get(&format!("{}SearchServer", self.endpoint))
            .query(&query)
            .send()?
            .error_for_status()?;
        let res: GeoAdminForwardResponse<T> = resp.json()?;
        Ok(res)
    }
}

impl Default for GeoAdmin {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA_STRING));
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Couldn't build a client!");
        GeoAdmin {
            client,
            endpoint: "https://api3.geo.admin.ch/rest/services/api/".to_string(),
            sr: "4326".to_string(),
        }
    }
}

impl<T> Forward<T> for GeoAdmin
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A forward-geocoding lookup of an address. Please see [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search) for details.
    ///
    /// This method passes the `type`,  `origins`, `limit` and `sr` parameter to the API.
    fn forward(&self, place: &str) -> Result<Vec<Point<T>>, GeocodingError> {
        let resp = self
            .client
            .get(&format!("{}SearchServer", self.endpoint))
            .query(&[
                ("searchText", place),
                ("type", "locations"),
                ("origins", "address"),
                ("limit", "1"),
                ("sr", &self.sr),
                ("geometryFormat", "geojson"),
            ])
            .send()?
            .error_for_status()?;
        let res: GeoAdminForwardResponse<T> = resp.json()?;
        // return easting & northing consistent
        let results = if vec!["2056", "21781"].contains(&self.sr.as_str()) {
            res.features
                .iter()
                .map(|feature| Point::new(feature.properties.y, feature.properties.x)) // y = west-east, x = north-south
                .collect()
        } else {
            res.features
                .iter()
                .map(|feature| Point::new(feature.properties.x, feature.properties.y)) // x = west-east, y = north-south
                .collect()
        };
        Ok(results)
    }
}

impl<T> Reverse<T> for GeoAdmin
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A reverse lookup of a point. More detail on the format of the
    /// returned `String` can be found [here](https://api3.geo.admin.ch/services/sdiservices.html#identify-features)
    ///
    /// This method passes the `format` parameter to the API.
    fn reverse(&self, point: &Point<T>) -> Result<Option<String>, GeocodingError> {
        let resp = self
            .client
            .get(&format!("{}MapServer/identify", self.endpoint))
            .query(&[
                (
                    "geometry",
                    format!(
                        "{},{}",
                        point.x().to_f64().unwrap(),
                        point.y().to_f64().unwrap()
                    )
                    .as_str(),
                ),
                ("geometryType", "esriGeometryPoint"),
                ("layers", "all:ch.bfs.gebaeude_wohnungs_register"),
                ("mapExtent", "0,0,100,100"),
                ("imageDisplay", "100,100,100"),
                ("tolerance", "50"),
                ("geometryFormat", "geojson"),
                ("sr", &self.sr),
                ("lang", "en"),
            ])
            .send()?
            .error_for_status()?;
        let res: GeoAdminReverseResponse = resp.json()?;
        if !res.results.is_empty() {
            let properties = &res.results[0].properties;
            let address = format!(
                "{}, {} {}",
                properties.strname_deinr, properties.dplz4, properties.dplzname
            );
            Ok(Some(address))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_with_sr_forward_test() {
        let geoadmin = GeoAdmin::new().with_sr("2056");
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(&address);
        assert_eq!(res.unwrap(), vec![Point::new(2_600_968.75, 1_197_427.0)]);
    }

    #[test]
    fn new_with_endpoint_forward_test() {
        let geoadmin =
            GeoAdmin::new().with_endpoint("https://api3.geo.admin.ch/rest/services/api/");
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(&address);
        assert_eq!(
            res.unwrap(),
            vec![Point::new(7.451352119445801, 46.92793655395508)]
        );
    }

    #[test]
    fn with_sr_forward_full_test() {
        let geoadmin = GeoAdmin::new().with_sr("2056");
        let bbox = InputBounds::new((2_600_967.75, 1_197_426.0), (2_600_969.75, 1_197_428.0));
        let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
            .with_origins("address")
            .with_bbox(&bbox)
            .build();
        let res: GeoAdminForwardResponse<f64> = geoadmin.forward_full(&params).unwrap();
        let result = &res.features[0];
        assert_eq!(
            result.properties.label,
            "Seftigenstrasse 264 <b>3084 Wabern</b>",
        );
    }

    #[test]
    fn forward_full_test() {
        let geoadmin = GeoAdmin::new();
        let bbox = InputBounds::new((7.4513398, 46.92792859), (7.4513662, 46.9279467));
        let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
            .with_origins("address")
            .with_bbox(&bbox)
            .build();
        let res: GeoAdminForwardResponse<f64> = geoadmin.forward_full(&params).unwrap();
        let result = &res.features[0];
        assert_eq!(
            result.properties.label,
            "Seftigenstrasse 264 <b>3084 Wabern</b>",
        );
    }

    #[test]
    fn forward_test() {
        let geoadmin = GeoAdmin::new();
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(&address);
        assert_eq!(
            res.unwrap(),
            vec![Point::new(7.451352119445801, 46.92793655395508)]
        );
    }

    #[test]
    fn with_sr_reverse_test() {
        let geoadmin = GeoAdmin::new().with_sr("2056");
        let p = Point::new(2_600_968.75, 1_197_427.0);
        let res = geoadmin.reverse(&p);
        assert_eq!(
            res.unwrap(),
            Some("Seftigenstrasse 264, 3084 Wabern".to_string()),
        );
    }

    #[test]
    fn reverse_test() {
        let geoadmin = GeoAdmin::new();
        let p = Point::new(7.451352119445801, 46.92793655395508);
        let res = geoadmin.reverse(&p);
        assert_eq!(
            res.unwrap(),
            Some("Seftigenstrasse 264, 3084 Wabern".to_string()),
        );
    }
}
