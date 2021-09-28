mod error;
pub use error::*;

use serde::Deserialize;

pub enum RequestArguments {
    Tile {
        /// The size of the
        size: u32,
        x: u32,
        y: u32,
        zoom: u32,
    },
}

/// Hits the rainviewer API to obtain a single tile of rain for the world
/// This function does no verification of the parameters
///
/// time: The unix time of the radar image. The caller must know that radar images for this time
///     otherwise this function will return a not found error when HTTP gives a 404.
/// size: The width and height of the image, either 256 or 512. Images are always square
/// z: The zoom of the tile within the world. See https://www.maptiler.com/google-maps-coordinates-tile-bounds-projection/
///     for a more visual explanation of what this is.
/// x: The x coordinate of the radar image. Must be in the range 0..2^z-1
/// y: The y coordinate of the radar image. Must be in the range 0..2^z-1
/// color: A color scheme id in the range 0..8. Variants listed here: https://www.rainviewer.com/api/color-schemes.html
/// options: A options string in the format `{smooth}_{snow}`. Where smooth and snow are booleans
///     spelled out as 0 or 1.
///
///See `https://www.rainviewer.com/api/weather-maps-api.html` for more details
pub(crate) async fn get_tile_unchecked(
    host: &str,
    time: u64,
    size: u32,
    z: u32,
    x: u32,
    y: u32,
    color: u32,
    options: &str,
) -> Result<Vec<u8>, error::Error> {
    let url = format!(
        "{}/v2/radar/{}/{}/{}/{}/{}/{}/{}.png",
        host, time, size, z, y, x, color, options,
    );
    println!("Requesting: {}", url);
    let res = reqwest::get(url).await?;
    match res.status() {
        reqwest::StatusCode::OK => Ok(res.bytes().await?.to_vec()),
        status => Err(Error::Http(status)),
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
        let png = get_tile_unchecked(maps.host.as_str(), frame.time, 256, 6, 12, 26, 2, "1_1")
            .await
            .unwrap();

        //Check for PNG magic
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4e, 0x47]);
    }
}
