mod error;
pub use error::*;

use serde::Deserialize;

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

struct TileArguments {
    /// The size of the
    size: u32,
    x: u32,
    y: u32,
    zoom: u32,
    color: ColorKind,
    smooth: bool,
    snow: bool,
}

enum RequestArgumentsInner {
    Tile(TileArguments),
}

pub struct RequestArguments {
    inner: RequestArgumentsInner,
}

impl RequestArguments {
    pub fn new_tile(x: u32, y: u32, zoom: u32) -> Self {
        Self {
            inner: RequestArgumentsInner::Tile(TileArguments {
                size: 256,
                x,
                y,
                zoom,
                color: ColorKind::UniversalBlue,
                smooth: true,
                snow: true,
            }),
        }
    }

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

    pub fn set_smooth(&mut self, smooth: bool) -> &mut Self {
        match &mut self.inner {
            RequestArgumentsInner::Tile(tile) => {
                tile.smooth = smooth;
            }
        };
        self
    }

    pub fn set_snow(&mut self, snow: bool) -> &mut Self {
        match &mut self.inner {
            RequestArgumentsInner::Tile(tile) => {
                tile.snow = snow;
            }
        };
        self
    }

    pub fn set_color(&mut self, color: ColorKind) -> &mut Self {
        match &mut self.inner {
            RequestArgumentsInner::Tile(tile) => {
                tile.color = color;
            }
        };
        self
    }
}

/// Hits the rainviewer API to obtain a single tile of rain for the world
///
///
///See `https://www.rainviewer.com/api/weather-maps-api.html` for more details
pub async fn get_tile(
    host: &str,
    time: u64,
    args: RequestArguments,
) -> Result<Vec<u8>, error::Error> {
    match args.inner {
        RequestArgumentsInner::Tile(args) => {
            let options = format!("{}_{}", args.smooth as u8, args.snow as u8);
            let color_val: u32 = args.color.into();
            let url = format!(
                "{}/v2/radar/{}/{}/{}/{}/{}/{}/{}.png",
                host, time, args.size, args.zoom, args.x, args.y, color_val, options,
            );
            println!("Requesting: {}", url);
            let res = reqwest::get(url).await?;
            match res.status() {
                reqwest::StatusCode::OK => Ok(res.bytes().await?.to_vec()),
                status => Err(Error::Http(status)),
            }
        }
    }
}

pub async fn weather_maps() -> Result<WeatherMaps, error::Error> {
    let res = reqwest::get("https://api.rainviewer.com/public/weather-maps.json").await?;
    Ok(serde_json::from_str(res.text().await?.as_str())?)
}

#[derive(Deserialize)]
pub struct WeatherMaps {
    pub version: String,
    pub generated: u64,
    pub host: String,
    pub radar: Radar,
    pub satellite: Satellite,
}

#[derive(Deserialize)]
pub struct Frame {
    pub time: u64,
    pub path: String,
}

#[derive(Deserialize)]
pub struct Radar {
    pub past: Vec<Frame>,
    pub nowcast: Vec<Frame>,
}

#[derive(Deserialize)]
pub struct Satellite {
    pub infrared: Vec<Frame>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let maps = weather_maps().await.unwrap();
        let frame = &maps.radar.past[0];
        let args = RequestArgumentsInner::Tile(TileArguments {
            size: 256,
            x: 26,
            y: 12,
            zoom: 6,
            color: ColorKind::UniversalBlue,
            smooth: true,
            snow: true,
        });
        let png = get_tile(
            maps.host.as_str(),
            frame.time,
            RequestArguments { inner: args },
        )
        .await
        .unwrap();

        //Check for PNG magic
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4e, 0x47]);
    }
}
