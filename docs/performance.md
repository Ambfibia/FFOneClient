# Rendering performance

Preserve world coverage, camera range, geometry, texture/mip quality, shader passes, collider
ownership and animation timing. Resource sharing never authorizes deleting scene placements.
This is the canonical optimization/measurement contract; dated observations below are not fresh
results and are not requirements to load old captures for unrelated work.

## Resource ownership

Static admission interns immutable mesh/image/material values after complete equality, with hashes
as lookup accelerators only. Normals, UVs, indices, full mip chains, color spaces, sampler and
alpha/depth/cull/sort state remain distinct. Weak handles do not pin unloaded maps. Asset events
invalidate revision fingerprints and opacity; inspect with immutable Assets::get, never iter_mut.
Exclude storage images/render targets. Existing scene reload semantics remain unchanged.

Model/effect admission occurs after final renderer/pass sort bias. Allocation state is unadmitted,
shared or private and is non-semantic. Explicit clones are private. Appearance/portrait/player/Nano,
tutorial dome and effect-animation writers detach shared handles synchronously before mutation;
writers before admission mark private. Outlines retain their ownership. Equal animated snapshots
must not be re-interned. Flattened static-world float curves may defer detachment until the first
actual value change; generic models and analytic GPU UV tracks retain eager private ownership.
Binding identity includes player, target and occurrence, preserving overlapping assignment order.

Animated numeric materials retain one persistent 208-byte GPU uniform buffer per material ID.
Stage extraction after asset extraction, apply at RenderSystems::ExtractCommands, before material
preparation. Up to 64 updates use direct writes; larger sets use one reusable upload buffer and
range copies. Preserve WGSL ABI, float values, texture bindings and identity. Animated _Cutoff uses
normal asset events because AlphaMode::Mask caches it. Pending final poses survive texture loading;
unused assets release buffers. Immutable cache equality excludes GPU ownership.

Exact mip assembly shares only equal completed images. Weak caches plus temporary strong leases
retain completed slots while other slots load, releasing on admission/cancellation/source unload.
Native file deduplication additionally preserves GLB BIN bytes, geometry, animation, material values
and all placements; terrain density sharing remains within its native terrain domain. Byte-identical
icons or individual mips with independent routing are not automatically interchangeable.

## UI and lifecycle

Localization skips unchanged labels but responds to language/bundle changes, semantic text/arguments,
casing changes/removal and external rendered-text edits. EN/RU keys/placeholders/fonts/layout stay
unchanged. HUD/chat/minimap writers compare final fields before mutable access; real changes and
external corrections still produce normal events. Keep guards at producers. A broad layout/stack
cache was measured and rejected; Bevy traversal remains a cost.

All persistent gameplay UI constructors run once through NativeGameplayUiStartup. Deferred commands
must complete before the first gameplay update; phase/session bindings continue normally. Reentry
must preserve entity IDs, not merely camera/root counts. Selection resources become resident on
first return, so compare later entries for steady counts.

## Rig and ground queries

Rig validation indexes source/pass queries once per update with a dense, generation-checked epoch
index. Descendant membership and original query ranks preserve source/pass/error order, nested rigs,
same-update reparenting/removals, metadata grace and readiness. Do not stop checking after Ready.
The index retains entities/ranks, not render assets; a slower hash-map version was rejected.

Repeated NPC/remote point-ground queries use an 8×8 world-space triangle grid. Outward-rounded
barycentric intervals conservatively select candidates; final arithmetic and triangle/tie order
remain exact. Preparation is bounded to 512 source triangles and 4,096 cell tests per update;
queries use the original routine until complete. Matrix-bit or immutable geometry changes restart
preparation; unused previous-update entries and removed colliders retire. Player capsule/sweep/support,
triggers, broad phase, gravity and timing are unchanged.

## Streaming and visibility

Retain nine physical roots and 356/420 load/unload bands. Reserve a root before I/O. At most two
terrain decodes overlap and one completed tile installs per frame; retirement cancels its own task
without waiting for unrelated decoding. Backpressure counts actual outstanding components across
all roots. Preserve admission/finalization limits and nearest-distance tie/source order.

Leaf-first retirement has a 1 ms soft budget and 256-entity hard cap, including relationship hooks
and handle releases; one operation can exceed the soft deadline. Bound descent visits, retain progress
on repeated cancellation and handle late children/reparenting. Idle retirement schedules do no work.

Range admission tests complete world bounds against the packed/decoded center contract and 32-unit
fade band. Large geometry that cannot fit uses ordinary AABB/frustum/far-plane culling without custom
center fade; late passes inherit this. Small objects retain their ranges. City Station rails have
radii about 349.4/334.8, exceeding the 308–340 band; preserve the geometry-based guard, camera far 340,
nine-tile cap and actual rail/collider geometry. Do not replace it with a model-name exception.

## Reproduction

```powershell
cargo build -p ffone-client --features diagnostics --example world_performance_probe --locked
target/debug/examples/world_performance_probe.exe baseline target/performance/baseline --freeze
target/debug/examples/world_performance_probe.exe shared target/performance/shared --freeze
node ../FusionForge/tools/native/audit-render-duplicates.mjs assets/game target/performance/duplicates.json
cargo test -p ffone-client --lib localization:: --locked
cargo test -p ffone-client --test ui_reentry_standalone --locked
```

The world probe waits for readiness, warms three seconds and measures 360 intervals at 1280×720.
Baseline disables content interning only. Optional X Y Z after output selects location; omit freeze
for timing. Run sequentially, reverse order, same adapter/settings, no competing compilation/game.
Frozen pixels establish appearance; animated scenes need matched moving runs. CPU bytes are not VRAM;
visible entities are not draw calls; reuse counters are operations, not allocations saved.

The full-client diagnostic uses production plugins,1920×1080,far 340, readiness,five-second warmup,
600 intervals, fixed 1/60 simulation. Set FFONE_PERF_OUTPUT; add FFONE_PERF_NPCS=1 and
FFONE_PERF_ORBIT=1 for published NPCs and the same 300-degree orbit. FFONE_PERF_POSITION takes XYZ;
FFONE_PERF_FREEZE=1 fixes time. This bypasses normal login/settings only for this explicit fixture;
it is not server movement/combat/crowd acceptance. FFONE_PERF_TRANSPORT_NPC_TYPE=964 at
-3750.5798 -56.9 4484.4097 opens a real transport portrait with fixture route unlocks/Taros;
wait for a bound loaded image and capture EN/RU separately.

FFONE_PERF_ENTRIES=3 alternates two UIDs with equal appearance, disconnects the typed NPC epoch,
removes the world slice, visits selection for at least 500 ms and runs production entry/exit.
FFONE_PERF_UI_REENTRY_BASELINE=1 restores old fixture construction; normal execution uses the fix.
Keyboard/mouse are discarded in performance fixtures. The dedicated input fixture owns deliberate
input. Entry reports include entity/UI/camera/world counts and player/camera poses.

FFONE_PROBE_TILE_STEP='512 0 0' measures a world-probe transition. FFONE_PERF_CPU_QUERY_BASELINE=1
with FFONE_PERF_OUTPUT restores old rig/ground scans. FFONE_PERF_MODEL_MATERIAL_BASELINE=1 with
FFONE_PERF_OUTPUT or FFONE_PERF_ASSET_AUDIT restores pre-admission material ownership while keeping
static sharing enabled. FFONE_PERF_ASSET_AUDIT=1 adds a current-value material census after timing;
it is not proof of immutable ownership and never rewrites content.

Use the profiling feature and TRACE_CHROME for attribution; FFONE_PERF_GPU=1 enables pass diagnostics.
Detailed tracing perturbs timing. Exclude final report/screenshot/shutdown from the 600 intervals;
span midpoints correlate work but pipelined/inclusive CPU/GPU durations are not additive. Preserve
normal untraced comparison runs and reject runs overlapping compilation, without stopping others' work.

Historical measurements are optional: [dated baseline tables](reference/performance-baselines.md).
Do not read/replay them for unrelated UI work. Choose the affected fixture only; no FPS claim
from duplicate counts, traced spans, synthetic CPU savings or runs overlapping compilation.
