fn main() {
    #[cfg(target_os = "windows")]
    winres::WindowsResource::new()
        .set_icon("assets/pot.ico")
        .set(
            "FileDescription",
            "An editor for cooked unreal engine 4 map files",
        )
        .compile()
        .expect("failed to change icon")
}
