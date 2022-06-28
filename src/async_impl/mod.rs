pub use crate::GeocodingError;
pub use geo_types::{Coordinate, Point};
use num_traits::Float;
use std::fmt::Debug;

pub mod openstreetmap;

pub trait Reverse<T>
where
    T: Float + Debug,
{
    // NOTE TO IMPLEMENTERS: Point coordinates are lon, lat (x, y)
    // You may have to provide these coordinates in reverse order,
    // depending on the provider's requirements (see e.g. OpenCage)
    fn reverse(
        &self,
        point: &Point<T>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>, GeocodingError>>>>;
}

pub trait Forward<T>
where
    T: Float + Debug,
{
    fn forward(
        &self,
        address: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<Point<T>>, GeocodingError>> + Send + '_>,
    >;
}
