fn main() {
    #[cfg(windows)]
    winres::WindowsResource::new()
        .set_icon("assets/pot.ico")
        .compile()
        .expect("failed to change icon")
}
