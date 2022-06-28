//! The [OpenStreetMap Nominatim](https://nominatim.org/) provider.
//!
//! Geocoding methods are implemented on the [`Openstreetmap`](struct.Openstreetmap.html) struct.
//! Please see the [API documentation](https://nominatim.org/release-docs/develop/) for details.
//!
//! While OpenStreetMap's Nominatim API is free, see the [Nominatim Usage Policy](https://operations.osmfoundation.org/policies/nominatim/)
//! for details on usage requirements, including a maximum of 1 request per second.
//!
//! ### Example
//!
//! ```
//! use geocoding::{Openstreetmap, Forward, Point};
//!
//! let osm = Openstreetmap::new();
//! let address = "Schwabing, München";
//! let res = osm.forward(&address);
//! assert_eq!(res.unwrap(), vec![Point::new(11.5884858, 48.1700887)]);
//! ```
pub use crate::shared::openstreetmap::{
    AddressDetails, OpenstreetmapParams, OpenstreetmapResponse, OpenstreetmapResult,
    ResultGeometry, ResultProperties,
};
use crate::Deserialize;
use crate::GeocodingError;
use crate::Point;
use crate::UA_STRING;
use crate::{Client, HeaderMap, HeaderValue, USER_AGENT};
use crate::{Forward, Reverse};
use num_traits::Float;
use std::fmt::Debug;

/// An instance of the Openstreetmap geocoding service
pub struct Openstreetmap {
    client: Client,
    endpoint: String,
}

impl Openstreetmap {
    /// Create a new Openstreetmap geocoding instance using the default endpoint
    pub fn new() -> Self {
        Openstreetmap::new_with_endpoint("https://nominatim.openstreetmap.org/".to_string())
    }

    /// Create a new Openstreetmap geocoding instance with a custom endpoint.
    ///
    /// Endpoint should include a trailing slash (i.e. "https://nominatim.openstreetmap.org/")
    pub fn new_with_endpoint(endpoint: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA_STRING));
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Couldn't build a client!");
        Openstreetmap { client, endpoint }
    }

    /// A forward-geocoding lookup of an address, returning a full detailed response
    ///
    /// Accepts an [`OpenstreetmapParams`](struct.OpenstreetmapParams.html) struct for specifying
    /// options, including whether to include address details in the response and whether to filter
    /// by a bounding box.
    ///
    /// Please see [the documentation](https://nominatim.org/release-docs/develop/api/Search/) for details.
    ///
    /// This method passes the `format` parameter to the API.
    ///
    /// # Examples
    ///
    /// ```
    /// use geocoding::{Openstreetmap, InputBounds, Point};
    /// use geocoding::openstreetmap::{OpenstreetmapParams, OpenstreetmapResponse};
    ///
    /// let osm = Openstreetmap::new();
    /// let viewbox = InputBounds::new(
    ///     (-0.13806939125061035, 51.51989264641164),
    ///     (-0.13427138328552246, 51.52319711775629),
    /// );
    /// let params = OpenstreetmapParams::new(&"UCL CASA")
    ///     .with_addressdetails(true)
    ///     .with_viewbox(&viewbox)
    ///     .build();
    /// let res: OpenstreetmapResponse<f64> = osm.forward_full(&params).unwrap();
    /// let result = res.features[0].properties.clone();
    /// assert!(result.display_name.contains("Gordon Square"));
    /// ```
    pub fn forward_full<T>(
        &self,
        params: &OpenstreetmapParams<T>,
    ) -> Result<OpenstreetmapResponse<T>, GeocodingError>
    where
        T: Float + Debug,
        for<'de> T: Deserialize<'de>,
    {
        let format = String::from("geojson");
        let addressdetails = String::from(if params.addressdetails { "1" } else { "0" });
        // For lifetime issues
        let viewbox;

        let mut query = vec![
            (&"q", params.query),
            (&"format", &format),
            (&"addressdetails", &addressdetails),
        ];

        if let Some(vb) = params.viewbox {
            viewbox = String::from(*vb);
            query.push((&"viewbox", &viewbox));
        }

        let resp = self
            .client
            .get(&format!("{}search", self.endpoint))
            .query(&query)
            .send()?
            .error_for_status()?;
        let res: OpenstreetmapResponse<T> = resp.json()?;
        Ok(res)
    }
}

impl Default for Openstreetmap {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Forward<T> for Openstreetmap
where
    T: Float + Debug,
    for<'de> T: Deserialize<'de>,
{
    /// A forward-geocoding lookup of an address. Please see [the documentation](https://nominatim.org/release-docs/develop/api/Search/) for details.
    ///
    /// This method passes the `format` parameter to the API.
    fn forward(&self, place: &str) -> Result<Vec<Point<T>>, GeocodingError> {
        let resp = self
            .client
            .get(&format!("{}search", self.endpoint))
            .query(&[(&"q", place), (&"format", &String::from("geojson"))])
            .send()?
            .error_for_status()?;
        let res: OpenstreetmapResponse<T> = resp.json()?;
        Ok(res
            .features
            .iter()
            .map(|res| Point::new(res.geometry.coordinates.0, res.geometry.coordinates.1))
            .collect())
    }
}

impl<T> Reverse<T> for Openstreetmap
where
    T: Float + Debug,
    for<'de> T: Deserialize<'de>,
{
    /// A reverse lookup of a point. More detail on the format of the
    /// returned `String` can be found [here](https://nominatim.org/release-docs/develop/api/Reverse/)
    ///
    /// This method passes the `format` parameter to the API.
    fn reverse(&self, point: &Point<T>) -> Result<Option<String>, GeocodingError> {
        let resp = self
            .client
            .get(&format!("{}reverse", self.endpoint))
            .query(&[
                (&"lon", &point.x().to_f64().unwrap().to_string()),
                (&"lat", &point.y().to_f64().unwrap().to_string()),
                (&"format", &String::from("geojson")),
            ])
            .send()?
            .error_for_status()?;
        let res: OpenstreetmapResponse<T> = resp.json()?;
        let address = &res.features[0];
        Ok(Some(address.properties.display_name.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::InputBounds;

    #[test]
    fn new_with_endpoint_forward_test() {
        let osm =
            Openstreetmap::new_with_endpoint("https://nominatim.openstreetmap.org/".to_string());
        let address = "Schwabing, München";
        let res = osm.forward(&address);
        assert_eq!(res.unwrap(), vec![Point::new(11.5884858, 48.1700887)]);
    }

    #[test]
    fn forward_full_test() {
        let osm = Openstreetmap::new();
        let viewbox = InputBounds::new(
            (-0.13806939125061035, 51.51989264641164),
            (-0.13427138328552246, 51.52319711775629),
        );
        let params = OpenstreetmapParams::new(&"UCL CASA")
            .with_addressdetails(true)
            .with_viewbox(&viewbox)
            .build();
        let res: OpenstreetmapResponse<f64> = osm.forward_full(&params).unwrap();
        let result = res.features[0].properties.clone();
        assert!(result.display_name.contains("Gordon Square"));
        assert_eq!(result.address.unwrap().city.unwrap(), "London");
    }

    #[test]
    fn forward_test() {
        let osm = Openstreetmap::new();
        let address = "Schwabing, München";
        let res = osm.forward(&address);
        assert_eq!(res.unwrap(), vec![Point::new(11.5884858, 48.1700887)]);
    }

    #[test]
    fn reverse_test() {
        let osm = Openstreetmap::new();
        let p = Point::new(2.12870, 41.40139);
        let res = osm.reverse(&p);
        assert!(res
            .unwrap()
            .unwrap()
            .contains("Barcelona, Barcelonès, Barcelona, Catalunya"));
    }
}
