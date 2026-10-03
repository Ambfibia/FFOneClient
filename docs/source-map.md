# Source ownership

`crates/ffone-client/src/lib.rs` is the compatibility facade, not the implementation.
Choose one owner; old public module names remain reexports.

| Area | Source |
| --- | --- |
| Startup, schedules, network ingress | `crates/ffone-client/src/app/` |
| UI screens and shared UI primitives | `crates/ffone-client/src/ui/` |
| Character assembly, appearance, animation | `crates/ffone-client/src/characters/` |
| Gameplay state and services | `crates/ffone-client/src/gameplay/` |
| Scene loading and placement | `crates/ffone-client/src/world/` |
| Live world systems, authority, targeting | `crates/ffone-client/src/world_systems/` |
| Materials, terrain rendering, glow | `crates/ffone-client/src/rendering/` |
| Tutorial orchestration and presentation | `crates/ffone-client/src/tutorial_runtime/` |
| Protocol and generated wire records | `crates/ffone-protocol/src/` |
| Shared foundation, network, tutorial core, schemas, editors | Matching workspace crate |

Tests live beside their owner in `tests.rs` or `tests/`; integration fixtures remain
under each crate's `tests/`. They are regression coverage, not obsolete bug reports.
Examples are isolated opt-in diagnostics under `examples/<domain>/<scenario>/`.
They retain separate crate roots because several use different ECS fixture types.

```powershell
rg -n -m 20 "symbol" crates/ffone-client/src/ui/option
rg --no-ignore -n "regression" crates/ffone-client/src/ui/option/tests
cargo test -p ffone-client --lib ui::option::
cargo probe option_ui_gpu_preview -- <original-arguments>
cargo probe-tests option_ui_gpu_preview
```

The example alias builds only the selected scenario; `diagnostics` is off by default.
Existing `cargo ... --example NAME` commands need `--features diagnostics`.
For source changes spanning a public boundary, run the affected dependent crates too.
Full workspace and GPU acceptance are for cross-cutting changes, not every local edit.

`.ignore` limits broad searches; it does not disable Cargo tests or remove assets.
Use `rg --no-ignore` only with an explicit excluded path. Generated protocol files are
owned by FusionForge's `tools/legacy-sources/generate-wire-0104.py`, not hand edits.
