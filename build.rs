fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS") == Ok("windows".to_string()) {
        winres::WindowsResource::new()
            .set_icon("assets/pot.ico")
            .set_manifest_file("assets/manifest.xml")
            .compile()
            .expect("failed to change icon")
    }
}
