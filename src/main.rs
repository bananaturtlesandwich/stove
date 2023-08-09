#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "stove",
        eframe::NativeOptions {
            icon_data: Some(eframe::IconData {
                rgba: include_bytes!("../assets/pot.rgba").to_vec(),
                width: 64,
                height: 64,
            }),
            initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
            ..Default::default()
        },
        Box::new(|ctx| Box::new(stove::Stove::new(ctx))),
    )
}
