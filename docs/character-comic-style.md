# Character comic shading

The native toon material uses a comic treatment: three flat lighting bands, cool violet shadows,
and a thin indigo outline. There is no halftone, stippling, or procedural print texture. The
toon lighting is an intentional native art-direction change rather than legacy shader parity.

The shared toon family covers players, NPCs, Nanos, and toon equipment/attachments, including
their preview scenes. Other consumers of that same toon family receive the same style. Terrain,
ordinary static-world materials, particles, holograms, and Fusion effect programs retain their
separate lighting paths. Authored texture colors, player skin/hair tints, texture alpha, cutouts,
fog, animation, skinning, and material pass ownership are retained. No new texture, material
allocation, or render pass is required.

`crates/ffone-client/src/rendering/legacy_model_material/legacy_model_base.wgsl` contains the palette and
lighting in `comic_surface`. Lighting uses the interpolated skinned normal and the material's
existing world-space light direction. Screen derivatives antialias the boundaries between the
three color regions. Texture-authored details and shading remain visible; this shader does not
repaint the assets.

The common sky rim is a restrained cel highlight. Material-owned special rims keep their
authored parameters. The ramp texture still supplies its authored alpha, but its RGB no longer
controls toon lighting.

`legacy_model_outline.wgsl` reuses the existing expanded hull and skinning. Its projected width
is held between 1.1 and 2 framebuffer pixels where the projected normal has a direction.
Authored zero-width outlines remain disabled. The line uses indigo-black as its minimum color;
brighter authored colored outlines retain their color. The original hull depth and fat-factor
deformation remain in effect.

For GPU inspection, build the native preview and capture ordinary production models:

```powershell
cargo build -p ffone-client --features diagnostics --example logical_model_gpu_preview --locked
target/debug/examples/logical_model_gpu_preview.exe --asset-root assets/game --model characters/npcs/npc_dexter/npc_dexter.glb --screenshot target/performance/comic-style/dexter.png --report target/performance/comic-style/dexter.json
```

Keep comparison images and diagnostic reports under ignored `target/performance/comic-style`.
Rebuild the client to pick up edits: these WGSL files are embedded in the executable.

## GPU verification

The September 4, 2026 captures under `target/performance/comic-style` include
`dexter-comic.png`, `nano-comic.png`, and `player-comic.png`. The NPC and Nano captures were
visually inspected with textured surfaces and no dot pattern. The shared player rig reached
`ReadyAnimated` with 133 bones, five parts, seven skin palettes, and seven skinned surfaces;
bone motion was observed. This rig-only harness does not bind a complete player look's skin,
face, and hair texture overrides. The separate tutorial-player appearance harness is currently
blocked by an existing item-catalog length mismatch (expected 1,152,578 bytes, actual 1,156,470).
The runtime catalog was not changed for this shader work.

The NPC capture reported zero shader errors and zero material errors. On the first cold run,
the diagnostic captured only the outline before surface pipeline preparation finished; a
repeat with the same primary camera produced the inspected final frame. A successful probe
status alone should therefore not be used as proof of a complete character image.

The development client and three preview executables built successfully. The material suite
passed all 68 tests with `cargo test -p ffone-client --lib legacy_model_material:: --locked`.
The read-only native duplicate audit also completed; no resource-count-based FPS claim is made.
