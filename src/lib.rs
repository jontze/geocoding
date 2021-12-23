//! This crate provides forward– and reverse-geocoding functionality for Rust.
//! Over time, a variety of providers will be added. Each provider may implement one or both
//! of the `Forward` and `Reverse` traits, which provide forward– and reverse-geocoding methods.
//!
//! Note that for the `reverse` method, the return type is simply `Option<String>`,
//! as this is the lowest common denominator reverse-geocoding result.
//! Individual providers may implement additional methods, which return more
//! finely-structured and/or extensive data, and enable more specific query tuning.
//! Coordinate data are specified using the [`Point`](struct.Point.html) struct, which has several
//! convenient `From` implementations to allow for easy construction using primitive types.
//!
//! ### A note on Coordinate Order
//! While individual providers may specify coordinates in either `[Longitude, Latitude]` **or**
//! `[Latitude, Longitude`] order,
//! `Geocoding` **always** requires [`Point`](struct.Point.html) data in `[Longitude, Latitude]` (`x, y`) order,
//! and returns data in that order.
//!
//! ### Usage of rustls
//!
//! If you like to use [rustls](https://github.com/ctz/rustls) instead of OpenSSL
//! you can enable the `rustls-tls` feature in your `Cargo.toml`:
//!
//!```toml
//![dependencies]
//!geocoding = { version = "*", default-features = false, features = ["rustls-tls"] }
//!```

static UA_STRING: &str = "Rust-Geocoding";

use chrono;
pub use geo_types::{Coordinate, Point};
use num_traits::Float;
use reqwest::header::ToStrError;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;
use thiserror::Error;

#[cfg(feature = "async")]
pub mod async_impl;
#[cfg(feature = "blocking")]
pub mod blocking;

// The OpenCage geocoding provider
pub mod opencage;
#[cfg(feature = "blocking")]
pub use crate::blocking::opencage::Opencage;

// The OpenStreetMap Nominatim geocoding provider
pub mod openstreetmap;
#[cfg(feature = "blocking")]
pub use crate::blocking::openstreetmap::Openstreetmap;

// The GeoAdmin geocoding provider
pub mod geoadmin;
#[cfg(feature = "blocking")]
pub use crate::blocking::geoadmin::GeoAdmin;

/// Errors that can occur during geocoding operations
#[derive(Error, Debug)]
pub enum GeocodingError {
    #[error("Forward geocoding failed")]
    Forward,
    #[error("Reverse geocoding failed")]
    Reverse,
    #[error("HTTP request error")]
    Request(#[from] reqwest::Error),
    #[error("Error converting headers to String")]
    HeaderConversion(#[from] ToStrError),
    #[error("Error converting int to String")]
    ParseInt(#[from] ParseIntError),
}

/// Used to specify a bounding box to search within when forward-geocoding
///
/// - `minimum` refers to the **bottom-left** or **south-west** corner of the bounding box
/// - `maximum` refers to the **top-right** or **north-east** corner of the bounding box.
#[derive(Copy, Clone, Debug)]
pub struct InputBounds<T>
where
    T: Float,
{
    pub minimum_lonlat: Point<T>,
    pub maximum_lonlat: Point<T>,
}

impl<T> InputBounds<T>
where
    T: Float,
{
    /// Create a new `InputBounds` struct by passing 2 `Point`s defining:
    /// - minimum (bottom-left) longitude and latitude coordinates
    /// - maximum (top-right) longitude and latitude coordinates
    pub fn new<U>(minimum_lonlat: U, maximum_lonlat: U) -> InputBounds<T>
    where
        U: Into<Point<T>>,
    {
        InputBounds {
            minimum_lonlat: minimum_lonlat.into(),
            maximum_lonlat: maximum_lonlat.into(),
        }
    }
}

/// Convert borrowed input bounds into the correct String representation
impl<T> From<InputBounds<T>> for String
where
    T: Float,
{
    fn from(ip: InputBounds<T>) -> String {
        // Return in lon, lat order
        format!(
            "{},{},{},{}",
            ip.minimum_lonlat.x().to_f64().unwrap().to_string(),
            ip.minimum_lonlat.y().to_f64().unwrap().to_string(),
            ip.maximum_lonlat.x().to_f64().unwrap().to_string(),
            ip.maximum_lonlat.y().to_f64().unwrap().to_string()
        )
    }
}
