/// The error type for this library. Wraps api errors and error types from downstream crates
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Json deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Server returned unexpected code: {}", 0)]
    Http(reqwest::StatusCode),

    #[error("Request failed: {0}")]
    Parameter(#[from] ParameterError),
}

/// Indicates that an invalid parameter was passed to a library function
/// Each variant contains the offending value and a string error message
#[derive(thiserror::Error, Debug)]
pub enum ParameterError {
    #[error("Invalid image size: {0} - {1}")]
    InvalidSize(u32, String),

    #[error("Invalid Zoom: {0} - {1}")]
    InvalidZoom(u32, String),

    #[error("X out of range: {0} - {1}")]
    XOutOfRange(u32, String),

    #[error("Y out of range: {0} - {1}")]
    YOutOfRange(u32, String),
}
