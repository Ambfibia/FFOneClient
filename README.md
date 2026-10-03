# FFOneClient

Native Rust/Bevy reconstruction of FusionFall without Unity Web Player, with native
editors for extending the game. Editable assets are ordinary files, not Unity bundles.
The server is RustyFusion (FFOne / Retrobution 0104). Legacy inspection/conversion is
owned by sibling FusionForge; neither it nor old builds is needed at runtime.

Install Git LFS, then clone the client together with its editable assets submodule:

```powershell
git lfs install
git clone --recurse-submodules https://github.com/Ambfibia/FFOneClient
cd FFOneClient
```

For an existing checkout, run `git submodule update --init --recursive` before
starting the client. Binary resources in `assets/` use Git LFS; if they were cloned
with LFS downloads disabled, run `git -C assets lfs pull`.

```powershell
cargo dev
cargo editor
cargo ui-editor
cargo check-all --locked
cargo release --locked
```

`crates/` owns runtime, schemas and editors; `assets/game/` contains game content;
`assets/editor/` holds editor resources; `target/` is disposable build output.
Commands are defined in `.cargo/config.toml`.

[Build/platform details](docs/building.md) · [Task map](docs/navigation.md).
Read only the relevant reference. Historical verification is not a current completion claim.

[Source ownership and focused checks](docs/source-map.md).
