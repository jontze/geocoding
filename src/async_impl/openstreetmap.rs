use crate::async_impl::{Forward, Reverse};
pub use crate::shared::openstreetmap::{
    AddressDetails, OpenstreetmapParams, OpenstreetmapResponse, OpenstreetmapResult,
    ResultGeometry, ResultProperties,
};
use crate::{Float, Point, GeocodingError, UA_STRING};
use reqwest::{Client, header::{HeaderMap, HeaderValue, USER_AGENT}};
use serde::Deserialize;
use std::pin::Pin;
use std::future::Future;
use std::fmt::Debug;

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
    pub async fn forward_full<T>(
        &self,
        params: &OpenstreetmapParams<'_, T>,
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
            .send().await?
            .error_for_status()?;
        let res: OpenstreetmapResponse<T> = resp.json().await?;
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
    fn forward(
        &self,
        place: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Point<T>>, GeocodingError>> + Send + '_>> {
        let req = self
            .client
            .get(&format!("{}search", self.endpoint))
            .query(&[(&"q", place), (&"format", &String::from("geojson"))])
            .send();

        Box::pin(async move {
            let res: OpenstreetmapResponse<T> = req.await?.error_for_status()?.json().await?;

            Ok(res
                .features
                .iter()
                .map(|res| Point::new(res.geometry.coordinates.0, res.geometry.coordinates.1))
                .collect())
        })
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
    fn reverse(
        &self,
        point: &Point<T>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, GeocodingError>>>> {
        let req = self
            .client
            .get(&format!("{}reverse", self.endpoint))
            .query(&[
                (&"lon", &point.x().to_f64().unwrap().to_string()),
                (&"lat", &point.y().to_f64().unwrap().to_string()),
                (&"format", &String::from("geojson")),
            ])
            .send();

        Box::pin(async move {
            let res: OpenstreetmapResponse<T> = req.await?.error_for_status()?.json().await?;
            let address = &res.features[0];
            Ok(Some(address.properties.display_name.to_string()))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{InputBounds};

    #[tokio::test]
    async fn new_with_endpoint_forward_test() {
        let osm =
            Openstreetmap::new_with_endpoint("https://nominatim.openstreetmap.org/".to_string());
        let address = "Schwabing, München";
        let res = osm.forward(&address).await;
        assert_eq!(res.unwrap(), vec![Point::new(11.5884858, 48.1700887)]);
    }

    #[tokio::test]
    async fn forward_full_test() {
        let osm = Openstreetmap::new();
        let viewbox = InputBounds::new(
            (-0.13806939125061035, 51.51989264641164),
            (-0.13427138328552246, 51.52319711775629),
        );
        let params = OpenstreetmapParams::new(&"UCL CASA")
            .with_addressdetails(true)
            .with_viewbox(&viewbox)
            .build();
        let res: OpenstreetmapResponse<f64> = osm.forward_full(&params).await.unwrap();
        let result = res.features[0].properties.clone(); // FIXME: This test panics with index out of bound
        assert!(result.display_name.contains("Gordon Square"));
        assert_eq!(result.address.unwrap().city.unwrap(), "London");
    }

    #[tokio::test]
    async fn forward_test() {
        let osm = Openstreetmap::new();
        let address = "Schwabing, München";
        let res = osm.forward(&address).await;
        assert_eq!(res.unwrap(), vec![Point::new(11.5884858, 48.1700887)]);
    }

    #[tokio::test]
    async fn reverse_test() {
        let osm = Openstreetmap::new();
        let p = Point::new(2.12870, 41.40139);
        let res = osm.reverse(&p).await;
        assert!(res
            .unwrap()
            .unwrap()
            .contains("Barcelona, Barcelonès, Barcelona, Catalunya"));
    }
}