fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS") == Ok("windows".to_string()) {
        winres::WindowsResource::new()
            .set_icon("assets/pot.ico")
            .compile()
            .expect("failed to change icon")
    }
}
