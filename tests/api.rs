#[tokio::test]
async fn api() {
    let maps = rain_viewer::available().await.unwrap();
    let frame = &maps.past_radar[0];
    let mut args = rain_viewer::RequestArguments::new_tile(4, 7, 6).unwrap();
    args.set_color(rain_viewer::ColorKind::Titan);
    args.set_snow(true);
    args.set_smooth(false);
    let png = rain_viewer::get_tile(&maps, frame, args).await.unwrap();

    //Check for PNG magic
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4e, 0x47]);
}

#[should_panic]
#[tokio::test]
async fn bad_x() {
    let _ = rain_viewer::RequestArguments::new_tile(40, 1, 2).unwrap();
}

#[should_panic]
#[tokio::test]
async fn bad_y() {
    let _ = rain_viewer::RequestArguments::new_tile(0, 4, 2).unwrap();
}

#[should_panic]
#[tokio::test]
async fn bad_size() {
    let _ = rain_viewer::RequestArguments::new_tile(0, 4, 2)
        .unwrap()
        .set_size(100);
}
