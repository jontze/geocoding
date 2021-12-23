// FIXME: All docs in this module need a rework and all examples adjustments
use async_trait::async_trait;
pub use geo_types::{Coordinate, Point};
use num_traits::Float;

use crate::GeocodingError;

pub mod geoadmin;
pub mod opencage;
pub mod openstreetmap;

#[async_trait(?Send)]
pub trait Reverse<T>
where
    T: Float + Send,
{
    // NOTE TO IMPLEMENTERS: Point coordinates are lon, lat (x, y)
    // You may have to provide these coordinates in reverse order,
    // depending on the provider's requirements (see e.g. OpenCage)
    async fn reverse(&self, point: &Point<T>) -> Result<Option<String>, GeocodingError>;
}

#[async_trait]
pub trait Forward<T>
where
    T: Float,
{
    // NOTE TO IMPLEMENTERS: while returned provider point data may not be in
    // lon, lat (x, y) order, Geocoding requires this order in its output Point
    // data. Please pay attention when using returned data to construct Points
    async fn forward(&self, address: &str) -> Result<Vec<Point<T>>, GeocodingError>;
}
