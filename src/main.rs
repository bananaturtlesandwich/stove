#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use stove::Stove;

fn main() {
    miniquad::start(
        miniquad::conf::Conf {
            window_title: "stove".to_string(),
            sample_count: 32,
            // remind me to add a cool icon
            ..Default::default()
        },
        |ctx| Box::new(Stove::new(ctx)),
    );
}
