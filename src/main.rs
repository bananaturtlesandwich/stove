#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use stove::Stove;

fn main() {
    // println!("{:#?}",glam::Mat4::perspective_infinite_lh(45.0, 1920.0/1080.0, 1.0));
    miniquad::start(
        miniquad::conf::Conf {
            window_title: "stove".to_string(),
            sample_count: 8,
            fullscreen: true,
            icon: Some(miniquad::conf::Icon {
                small: *include_bytes!("../assets/pot_16.rgba"),
                medium: *include_bytes!("../assets/pot_32.rgba"),
                big: *include_bytes!("../assets/pot_64.rgba"),
            }),
            ..Default::default()
        },
        |ctx| Box::new(Stove::new(ctx)),
    );
}
