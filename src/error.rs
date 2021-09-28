#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    Http(reqwest::StatusCode),
    Parameter(ParameterError),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<ParameterError> for Error {
    fn from(e: ParameterError) -> Self {
        Error::Parameter(e)
    }
}

#[derive(Debug)]
pub enum ParameterError {
    InvalidSize(u32, String),
    InvalidZoom(u32, String),
    XOutOfRange(u32, String),
    YOutOfRange(u32, String),
}
