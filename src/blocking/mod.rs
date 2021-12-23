pub mod geoadmin;
pub mod opencage;
pub mod openstreetmap;

use crate::{GeocodingError, Point};
use num_traits::Float;

/// Reverse-geocode a coordinate.
///
/// This trait represents the most simple and minimal implementation
/// available from a given geocoding provider: some address formatted as Option<String>.
///
/// Examples
///
/// ```
/// use geocoding::{Opencage, Point, Reverse};
///
/// let p = Point::new(2.12870, 41.40139);
/// let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
/// let res = oc.reverse(&p).unwrap();
/// assert_eq!(
///     res,
///     Some("Carrer de Calatrava, 68, 08017 Barcelona, Spain".to_string())
/// );
/// ```
pub trait Reverse<T>
where
    T: Float,
{
    // NOTE TO IMPLEMENTERS: Point coordinates are lon, lat (x, y)
    // You may have to provide these coordinates in reverse order,
    // depending on the provider's requirements (see e.g. OpenCage)
    fn reverse(&self, point: &Point<T>) -> Result<Option<String>, GeocodingError>;
}

/// Forward-geocode a coordinate.
///
/// This trait represents the most simple and minimal implementation available
/// from a given geocoding provider: It returns a `Vec` of zero or more `Points`.
///
/// Examples
///
/// ```
/// use geocoding::{Coordinate, Forward, Opencage, Point};
///
/// let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
/// let address = "Schwabing, München";
/// let res: Vec<Point<f64>> = oc.forward(address).unwrap();
/// assert_eq!(
///     res,
///     vec![Point(Coordinate { x: 11.5884858, y: 48.1700887 })]
/// );
/// ```
pub trait Forward<T>
where
    T: Float,
{
    // NOTE TO IMPLEMENTERS: while returned provider point data may not be in
    // lon, lat (x, y) order, Geocoding requires this order in its output Point
    // data. Please pay attention when using returned data to construct Points
    fn forward(&self, address: &str) -> Result<Vec<Point<T>>, GeocodingError>;
}
