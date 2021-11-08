//! Rust bindings to the free Rain Viewer API <https://www.rainviewer.com/weather-radar-map-live.html>
//!
//! Provides easy access to satellite-imagery-style precipitation radar imagery
//! for the entire world.
//!
//! # Example
//!
//! ```
//! #[tokio::main]
//! async fn main() {
//!     //Create requester for issuing requests
//!     let req = rain_viewer::WeatherRequester::new();
//!     // Query what data is available
//!     let maps = req.available().await.unwrap();
//!
//!     // Pick the first past entry in the past to sample
//!     let frame = &maps.past_radar[0];
//!
//!     // Setup the arguments for the tile we want to access
//!     // Parameters are x, y and zoom following the satellite imagery style
//!     let mut args = rain_viewer::RequestArguments::new_tile(4, 7, 6).unwrap();
//!     // Use this pretty color scheme
//!     args.set_color(rain_viewer::ColorKind::Titan);
//!     // Enable showing snow in addition to rain
//!     args.set_snow(true);
//!     // Smooth out the tile image (looks nicer from tile to tile)
//!     args.set_smooth(false);
//!
//!     // Make an API call to get the time image data using our parameters
//!     let png = req.get_tile(&maps, frame, args)
//!         .await
//!         .unwrap();
//!
//!     //Check for PNG magic to make sure we got an image
//!     assert_eq!(&png[0..4], &[0x89, 0x50, 0x4e, 0x47]);
//! }
//! ```
//!
//! [`available`] is the entry point to obtaining radar imagery. This returns
//! historical data and forecast data that is available.
//!
//! From there, most users call [`get_tile`] to download a PNG of a specific satellite tile.

mod error;

pub use error::*;

use serde::Deserialize;

/// The kinds of colors supported by rainviewer
/// All have different visual attributes. See <https://www.rainviewer.com/api/color-schemes.html>
/// for more information
#[derive(Copy, Clone, Debug)]
pub enum ColorKind {
    BlackAndWhite,
    Original,
    UniversalBlue,
    Titan,
    TheWeatherChannel,
    Meteored,
    NexradLevelIII,
    RainbowSelexIS,
    DarkSky,
}

impl From<ColorKind> for u32 {
    fn from(color: ColorKind) -> Self {
        // Values obtained from: https://www.rainviewer.com/api/color-schemes.html
        match color {
            ColorKind::BlackAndWhite => 0,
            ColorKind::Original => 1,
            ColorKind::UniversalBlue => 2,
            ColorKind::Titan => 3,
            ColorKind::TheWeatherChannel => 4,
            ColorKind::Meteored => 5,
            ColorKind::NexradLevelIII => 6,
            ColorKind::RainbowSelexIS => 7,
            ColorKind::DarkSky => 8,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct TileArguments {
    size: u32,
    x: u32,
    y: u32,
    zoom: u32,
    color: ColorKind,
    smooth: bool,
    snow: bool,
}

#[derive(Copy, Clone, Debug)]
enum RequestArgumentsInner {
    Tile(TileArguments),
}

/// Arguments needed to pull a rain tile from rainviewer
#[derive(Copy, Clone)]
pub struct RequestArguments {
    inner: RequestArgumentsInner,
}

impl RequestArguments {
    /// Creates arguments struct suitable for making a radar image request for a single tile
    ///
    /// `x` and `x` must be less than `2^zoom`, or Err(...) is returned
    pub fn new_tile(x: u32, y: u32, zoom: u32) -> Result<Self, error::ParameterError> {
        let max_coord = 2u32.pow(zoom);
        if x >= max_coord {
            Err(ParameterError::XOutOfRange(
                x,
                format!(
                    "With a zoom of {}, the max value for x is {}",
                    zoom,
                    max_coord - 1
                ),
            ))
        } else if y >= max_coord {
            Err(ParameterError::YOutOfRange(
                y,
                format!(
                    "With a zoom of {}, the max value for y is {}",
                    zoom,
                    max_coord - 1
                ),
            ))
        } else {
            Ok(Self {
                inner: RequestArgumentsInner::Tile(TileArguments {
                    size: 256,
                    x,
                    y,
                    zoom,
                    color: ColorKind::UniversalBlue,
                    smooth: true,
                    snow: true,
                }),
            })
        }
    }

    /// Sets the size of the resulting image when the API call is made.
    ///
    /// `size` must be 256 or 512 else Err(...) is returned
    pub fn set_size(&mut self, size: u32) -> Result<&mut Self, error::ParameterError> {
        if size == 256 || size == 512 {
            match &mut self.inner {
                RequestArgumentsInner::Tile(tile) => {
                    tile.size = size;
                }
            };
            Ok(self)
        } else {
            Err(ParameterError::InvalidSize(
                size,
                "Image size must be either 256 or 512".to_owned(),
            ))
        }
    }

    /// Sets the size of the resulting tile image when the API call is made
    pub fn set_smooth(&mut self, smooth: bool) -> &mut Self {
        match &mut self.inner {
            RequestArgumentsInner::Tile(tile) => {
                tile.smooth = smooth;
            }
        };
        self
    }

    /// Sets weather or not the resulting tile should show snow
    pub fn set_snow(&mut self, snow: bool) -> &mut Self {
        match &mut self.inner {
            RequestArgumentsInner::Tile(tile) => {
                tile.snow = snow;
            }
        };
        self
    }

    /// Sets the color scheme for the tile
    pub fn set_color(&mut self, color: ColorKind) -> &mut Self {
        match &mut self.inner {
            RequestArgumentsInner::Tile(tile) => {
                tile.color = color;
            }
        };
        self
    }
}

pub struct WeatherRequester {
    client: reqwest::Client,
}

impl WeatherRequester {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
    /// Queries the Rain Viewer API for what current and historical data is available.
    /// This function should serve as the entry point so that the caller has the correct path and time
    /// information to call [`get_tile`]
    pub async fn available(&self) -> Result<AvailableData, error::Error> {
        let res = self
            .client
            .get("https://api.rainviewer.com/public/weather-maps.json")
            .send()
            .await?;
        let raw: RawAvailableData = serde_json::from_str(res.text().await?.as_str())?;

        Ok(AvailableData {
            host: raw.host,
            past_radar: raw.radar.past.into_iter().map(|r| r.into()).collect(),
            nowcast_radar: raw.radar.nowcast.into_iter().map(|r| r.into()).collect(),
            infrared_satellite: raw
                .satellite
                .infrared
                .into_iter()
                .map(|r| r.into())
                .collect(),
        })
    }

    /// Hits the Rain Viewer API to obtain a single tile of rain for the world
    ///
    /// `maps` is the struct returned from [`available`]
    ///
    /// `frame` is the data frame indicating the moment in time to pull from
    ///
    /// See <https://www.rainviewer.com/api/weather-maps-api.html> for more details
    pub async fn get_tile(
        &self,
        maps: &AvailableData,
        frame: &Frame,
        args: RequestArguments,
    ) -> Result<Vec<u8>, error::Error> {
        match args.inner {
            RequestArgumentsInner::Tile(args) => {
                let options = format!("{}_{}", args.smooth as u8, args.snow as u8);
                let color_val: u32 = args.color.into();
                let url = format!(
                    "{}/{}/{}/{}/{}/{}/{}/{}.png",
                    maps.host, frame.path, args.size, args.zoom, args.x, args.y, color_val, options,
                );
                let res = self.client.get(url).send().await?;
                match res.status() {
                    reqwest::StatusCode::OK => Ok(res.bytes().await?.to_vec()),
                    status => Err(Error::Http(status)),
                }
            }
        }
    }
}

/// Indicates that radar or satellite data is available for the time given at path [`path`]
#[derive(Debug, Clone)]
pub struct Frame {
    /// The timestamp when this data was generated
    pub time: chrono::NaiveDateTime,

    /// The path where this data can be accessed
    pub path: String,
}

/// Contains the kinds of imagery that are available
#[derive(Debug, Clone)]
pub struct AvailableData {
    host: String,
    pub past_radar: Vec<Frame>,
    pub nowcast_radar: Vec<Frame>,
    pub infrared_satellite: Vec<Frame>,
}

/// Base API information returned by [`available`]
///
/// `radar` and `satellite` contain frame objects that can be used in conjunction with [`get_tile`]
/// to obtain a tile of imagery.
#[derive(Deserialize)]
#[allow(dead_code)]
struct RawAvailableData {
    /// The version of Rain Viewer
    pub version: String,
    /// The unix timestamp when this response was generated
    pub generated: u64,

    /// The tile host. Pass this value to [`get_tile`] so that it contacts the correct mirror
    pub host: String,

    /// What radar information is available
    pub radar: Radar,

    /// What satellite information is available
    pub satellite: Satellite,
}

#[derive(Deserialize)]
struct Radar {
    past: Vec<RawFrame>,
    nowcast: Vec<RawFrame>,
}

#[derive(Deserialize)]
struct Satellite {
    infrared: Vec<RawFrame>,
}

#[derive(Deserialize, Debug, Clone)]
struct RawFrame {
    /// The unix timestamp when this data was generated
    pub time: u64,

    /// The path where this data can be accessed
    pub path: String,
}

impl From<RawFrame> for Frame {
    fn from(raw: RawFrame) -> Self {
        use chrono::TimeZone;

        Self {
            time: chrono::Utc.timestamp(raw.time as i64, 0).naive_utc(),
            path: raw.path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let req = WeatherRequester::new();
        let maps = req.available().await.unwrap();
        let frame = &maps.past_radar[0];
        let args = RequestArgumentsInner::Tile(TileArguments {
            size: 256,
            x: 26,
            y: 12,
            zoom: 6,
            color: ColorKind::UniversalBlue,
            smooth: true,
            snow: true,
        });
        let png = req
            .get_tile(&maps, frame, RequestArguments { inner: args })
            .await
            .unwrap();

        //Check for PNG magic
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4e, 0x47]);
    }
}
