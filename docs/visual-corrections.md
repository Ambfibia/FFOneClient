# Visual and display corrections — 2026-09-18

## Runtime changes

- Mordecai and Titan use the explicit native `nativeSurfaceStyle: "cel"` material option.
  It changes RGB lighting while preserving each surface's alpha, depth, culling, ordering,
  texture/mip/sampler contract and pass count. Existing material sharing compares the complete
  GPU uniform, including this option.
- Native bilinear model textures now participate in the anisotropic-filtering option.
  Enabling it promotes mip sampling to linear as required by WebGPU. Disabling it restores
  the authored mip filter and anisotropy. Point-filtered textures and UI/render targets are
  excluded. Original settings are carried by native sampler metadata; no permanent image
  handle cache or per-frame asset scan is added. Existing terrain anisotropic sampling remains.
- Each accepted melee swing receives a separate generation, even when it reuses the same
  attack clip. The sword trail restarts after its previous emission expires without requiring
  an idle animation frame. Closing/holding a UI pointer press remains isolated from combat.
- NPC skill-hit recipient results now drive Coco egg skill effects. Nano healing uses the
  skill table's target projectile; passive buff activation emits the table's one-shot effect
  once per authoritative condition edge. Removal/death/despawn clears owned effects.
- Healing-over-time packet type 24 is decoded with strict size/owner/HP validation. Local and
  remote absolute HP are applied, and native healing effect 54 is emitted at the recipient.
- Local condition timeouts now replace the local absolute condition mask, clearing mob/shiny
  radar together with its authoritative removal while preserving unrelated buffs.
- Programmatic window resize is applied by Winit in `Last`, after Bevy's normal camera update.
  The new schedule after `Last` refreshes camera targets only when window dimensions or scale
  change, before render extraction. This removes mismatched color/depth attachments.

## Native model and effect assets

Darwin's head now has continuous area-weighted normals across equal position/skin records,
retaining creases above 60 degrees. Only head normal accessor bytes change: geometry, UVs,
textures, skin weights and animations remain intact. The reported cheek triangles remain in
an untextured diagnostic before repair and disappear in the matching GPU view after repair.

Native healing/Coco assets include effects 54, 407, 438, 500, 502, 827 and 828, with projectile
contracts 88, 92, 97 and 105. Publication and exact source evidence belong to FusionForge.
No old client, conversion tool or extraction cache is required by FFOne at runtime.

Reproduction commands, source ownership and publication hashes are recorded in the sibling
Editor evidence `docs/reference/evidence/legacy/ffone/visual-bugs-20260918.json` and `docs/native-cli.md`.

## Verification

Runtime reports and GPU captures are below ignored `target/performance/visual-bugs`.
The production offline fixtures use `FFONE_PERF_RESOLUTION` and `FFONE_PERF_SKILL_HITS`
alongside `FFONE_PERF_OUTPUT`; they neither connect to a server nor persist user settings.
Resolution coverage includes 1280x720, 1600x900, 1024x768 and 1920x1080, with both windowed
and borderless-fullscreen transitions. `FFONE_PERF_ENTRIES=2` also checks the visible
fullscreen/windowed button after returning to character selection.

The resolution crash was reproduced before the fix (1920x1080 depth with 1280x720 color).
These mixed-resolution and effect captures are functional acceptance, not matched FPS
benchmarks. No performance gain is inferred from the read-only native duplicate audit.

Verified results: all six resize transitions and return to character selection passed in
`resolution-verified`; the captured fullscreen button is visible and within the window.
Darwin, Mordecai and Titan GPU previews and the healing/Coco/radar fixture passed. All 106
native publication files match the Editor evidence hashes.

The full current-checkout test suite is not green: library 1904 passed / 36 failed / 7 ignored;
application 414 passed / 5 failed. Remaining failures include asset/locale expectations,
migrated fixture paths, world validation and unrelated portrait/combat/mission assertions.
They are not automatically accepted as new golden values. Detailed results and limitations
are recorded in `target/performance/visual-bugs/verification.md`.
