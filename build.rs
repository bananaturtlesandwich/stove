fn main() {
    #[cfg(target_os = "windows")]
    winres::WindowsResource::new()
        .set_icon("assets/pot.ico")
        .compile()
        .expect("failed to change icon")
}
