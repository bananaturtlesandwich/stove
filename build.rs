fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS") == Ok("windows".to_string()) {
        winres::WindowsResource::new()
            .set_icon("assets/pot.ico")
            .set_manifest_file("assets/manifest.xml")
            .compile()
            .expect("failed to change icon");
        println!("cargo:rerun-if-env-changed=OODLE");
        println!(
            "cargo:rustc-link-search={}",
            std::env::var("OODLE").unwrap_or(
                "C:/Program Files/Epic Games/UE_5.1/Engine/Source/Runtime/OodleDataCompression/Sdks/2.9.8/lib/Win64".to_string()
            )
        );
    }
}
