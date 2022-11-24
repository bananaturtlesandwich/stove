fn main() {
    #[cfg(windows)]
    {
        winres::WindowsResource::new()
            .set_icon("assets/pot.ico")
            .set_manifest_file("assets/stove.manifest")
            .compile()
            .expect("failed to change icon")
    }
}
