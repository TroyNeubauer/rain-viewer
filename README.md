# rain_viewer

Rust bindings to the free Rain Viewer API <https://www.rainviewer.com/weather-radar-map-live.html>

Provides easy access to satellite-imagery-style precipitation radar imagery
for the entire world.

## Example

```rust
#[tokio::main]
async fn main() {
    // Query what data is available
    let maps = rain_viewer::available().await.unwrap();

    // Pick the first past entry in the past to sample
    let frame = &maps.past_radar[0];

    // Setup the arguments for the tile we want to access
    // Parameters are x, y and zoom following the satellite imagery style
    let mut args = rain_viewer::RequestArguments::new_tile(4, 7, 6).unwrap();
    // Use this pretty color scheme
    args.set_color(rain_viewer::ColorKind::Titan);
    // Enable showing snow in addition to rain
    args.set_snow(true);
    // Smooth out the tile image (looks nicer from tile to tile)
    args.set_smooth(false);

    // Make an API call to get the time image data using our parameters
    let png = rain_viewer::get_tile(&maps, frame, args)
        .await
        .unwrap();

    //Check for PNG magic to make sure we got an image
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4e, 0x47]);
}
```

[`available`] is the entry point to obtaining radar imagery. This returns
historical data and forecast data that is available.

From there, most users call [`get_tile`] to download a PNG of a specific satellite tile.

License: MIT
