fn main() {
    println!("cargo:rerun-if-changed=assets/fusionfall.ico");

    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/fusionfall.ico")
        .compile()
        .expect("failed to embed FusionFall icon into ffone-client.exe");
}
