# Building and running

The repository pins Rust 1.98.1 and Bevy 0.19.1. Cargo.lock preserves dependency versions;
use --locked for reproducibility. The local rodio patch retains spatial pan behavior
specified in vendor/rodio/FFONE-PATCH.md. Unity and FusionForge are not runtime inputs.

Windows requires an MSVC Rust host, Visual Studio C++ build tools and Windows SDK.
The configured rust-lld linker ships with the toolchain. Linux requires build-essential,
pkg-config, ALSA, udev, Wayland and X11 development libraries. Build native releases on
the corresponding OS; do not assume an unconfigured cross-linker works.

## Commands from the repository root

```powershell
cargo dev
cargo dev -- --language ru
cargo editor
cargo ui-editor
cargo check-all --locked
cargo test-all --locked
cargo release --locked
```

Aliases are canonical in .cargo/config.toml. cargo dev selects ffone-client directly;
it does not cook or rewrite assets. The release alias builds the client and native editor.
Output binaries are target/debug or target/release. Ship ordinary editable assets/game
beside the executable; compilation alone does not package those files.

FFONE_LANGUAGE selects the startup locale; F9 toggles it. --asset-root selects a native
asset tree. --validate-assets performs the client's contract smoke check. For headless
network smoke use --network-smoke with FFONE_USERNAME/FFONE_PASSWORD set locally;
do not put credentials in commands or documents. Runtime legacy-loader flags are rejected.

Run focused tests before the complete workspace suite. For filesystem-sharing failures
under incremental output, identify the process holding the file; do not disable incremental
compilation as a workaround. Review actual CI configuration when changing packaging.
