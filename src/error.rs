use reqwest::header::ToStrError;
use std::num::ParseIntError;
use thiserror::Error;

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
