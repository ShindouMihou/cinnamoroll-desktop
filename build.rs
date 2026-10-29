fn main() {
    println!("cargo:rerun-if-changed=packaging/update-channel.json");
    let channel: serde_json::Value = serde_json::from_slice(
        &std::fs::read("packaging/update-channel.json").expect("Cannot read update channel"),
    )
    .expect("Invalid update channel JSON");
    for (environment, field) in [
        ("CINNAMOROLL_UPDATE_REPOSITORY", "repository"),
        ("CINNAMOROLL_UPDATE_PUBLIC_KEY", "public_key"),
    ] {
        let value = std::env::var(environment).unwrap_or_else(|_| {
            channel[field]
                .as_str()
                .expect("Missing update channel field")
                .to_owned()
        });
        assert!(
            !value.contains(['\n', '\r']),
            "Invalid update channel value"
        );
        println!("cargo:rustc-env={environment}={value}");
    }
    println!("cargo:rerun-if-env-changed=CINNAMOROLL_UPDATE_REPOSITORY");
    println!("cargo:rerun-if-env-changed=CINNAMOROLL_UPDATE_PUBLIC_KEY");
    #[cfg(target_os = "macos")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rerun-if-changed=src/native/tray_macos.m");
        println!("cargo:rerun-if-changed=src/native/browser_audio.m");
        println!("cargo:rerun-if-changed=src/native/stacking_macos.m");
        println!("cargo:rerun-if-changed=src/native/startup_macos.m");
        cc::Build::new()
            .file("src/native/tray_macos.m")
            .file("src/native/browser_audio.m")
            .file("src/native/stacking_macos.m")
            .file("src/native/startup_macos.m")
            .flag("-fobjc-arc")
            .compile("cinnamoroll_tray");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=ServiceManagement");
    }
    println!("cargo:rerun-if-env-changed=SDL3_LIB_DIR");
    if std::env::var_os("CARGO_FEATURE_BUNDLED_SDL").is_none()
        && let Ok(directory) = std::env::var("SDL3_LIB_DIR")
    {
        println!("cargo:rustc-link-search=native={directory}");
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
        }
    }
}
