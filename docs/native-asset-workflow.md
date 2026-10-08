# Native assets

## Ownership and layout

Edit the files the client actually loads under `assets/game/`. No parallel `content/`
authoring copy, Unity bundles, migration cache or FusionForge runtime dependency.
`assets/editor/` is editor-only. JSON requires a named schema and consumer; unstructured
settings bags are not a substitute. Native definitions use semantic IDs and safe relative
references, never machine paths, build aliases, Unity IDs or conversion commands.

| Content | Current owner |
| --- | --- |
| Models, rigs, wardrobe | `characters/`; HNPC textures: `characters/hnpc/textures` |
| Reusable world objects | `objects/<category>/<resource-set>/` |
| Tiles/terrain/placements/environment | `map/` via `map/catalog.json` |
| UI/images/fonts | `ui/en/`, optional matching `ui/ru/`; approved native fonts |
| Text | `localization/{en,ru}.json` |
| Audio | `audio/`, semantic rows in TableData |
| Character/audio routes | `data/tables/xdt.json`, `_ffone.tables` / `native_asset_routes` |
| Shared effect mip chains | `effects/shared/textures` |

Preserve domain ownership and GLB-relative material/mip URIs. Do not restore a flat
`models/` or `textures/` dumping ground or make per-model copies of shared payloads.
Character models use `npcs`, `mobs`, `fusions`, `nanos` and `shinies`; shared
textures may remain under `characters/shared`. Route `category` owns the physical
directory, while accepted route IDs keep their original namespace and package slug.
`_runtime/audio.json` and `_runtime/characters.json` are retired, not runtime inputs.
Existing other catalogs, hashes and compatibility fields remain until their consumers
migrate; do not drop validation or regenerate the combined TableData from one legacy XDT.
`map/catalog.json` is a reference registry: IDs and guarded file references, without
source-build evidence, repeated object metadata or summary counts. Its v1 reader contract
and all path/byte/BLAKE3 bindings remain intact; see [map assets](map-assets.md).

Commit editable JSON/GLB/PNG/OGG/TTF/OTF/WGSL and explicit terrain height/layer sources.
Retain active native BIN contracts where required. Optimization/release output is
optional and disposable, not an authoring prerequisite. Plain Cargo compilation is
code-only; deleting a cache must not delete authored data. New supported content should
be discoverable without a hand-maintained Rust asset list.

## Identity and changes

`xdt.json` exposes gameplay tables at the root for the RustyFusion loader.
The `_ffone` extension owns the `ffone.xdt.v1` schema, table identities, native
routes and other client metadata. Preserve it when editing or distributing the
shared XDT. Client loaders reconstruct a named table view in memory; this is not
a second authored file. Do not overwrite client content with an older server copy.

Mission destinations in `data/missions/client-npc-waypoints.json` use
`ffone.client-npc-waypoint-catalog.v2`: `schema` and ordered `rows`, each with
`npcType` and `clientPosition` in the existing client-world units. Row indices and
counts are derived; duplicate types retain first-match order. The loader still
validates v1 provenance/counts/indices when reading older files. The mission editor
publishes v2 without legacy provenance and refreshes appended destinations from
the selected server's `NPCs.json`, preserving the accepted baseline prefix.
Existing authored positions remain intact when the selected server has no matching
placement; they do not block unrelated edits. Newly assigned mission destinations
still require a matching placement before Rewrite publishes gameplay files.

Keep explicit network IDs, accepted variants/remaps and approved replacement fonts.
Duplicate semantic/network identities fail; replacement must be explicit and preserve
protocol identity. New text keys and placeholders enter EN/RU together.
Native authoring needs no legacy provenance. For an explicitly requested import use
[FusionForge's direct converter](../../FusionForge/docs/conversion-workflow.md), not
manual post-export calculations/copy/repair steps.

Current `assets/game` is the runtime root. Package discovery is an incremental accepted
target, not permission to move production files; its schema/conflict rules are in
[editable-content-architecture](editable-content-architecture.md).

## Acceptance by changed domain

Logical-model material `extras.ffone.nativeSurfaceStyle: "cel"` retains the
validated normal surface pass, including its blend and depth state. Optional
`nativeOutline: true` adds the existing skinned ink pass through
`PendingLegacyModelMaterial`; it requires the cel style and uses `_Outline` and
`_OutlineColor` (defaults: 0.005 and black). Keep transparent outer shells without
this pass when the opaque ink hull would obscure their interior; outline the
depth-writing body instead. Source `passes` continue to describe the validated
surface contract, before these native additions.

Opaque cel materials may opt into `nativeInkRim: true` for rounded inset parts
such as eyes. This draws an antialiased black grazing-angle rim in the surface
shader using skinned normals and the camera direction; the center retains its
texture and cel shading. Scope it to a separate primitive/material so the head
and body keep their existing surface and silhouette passes. It does not change
blend, depth, texture/mip ownership or the GPU uniform layout.

| Domain | Preserve/check |
| --- | --- |
| Model/world | Hierarchy, slots, topology, basis, transforms, bounds, normals, UV, skin, colliders |
| Animation | Rig, duration, wrap, targets/interpolation/events, transitions/root motion; asymmetric pose samples |
| Texture/material | Pixels, color/alpha, sampler, full mip chain, ordered passes, mutable ownership |
| Terrain | Identity, scale, heights/seams/holes, layer tiling, placements/collider samples |
| UI | Geometry/anchors/clipping, state/input/draw order, fonts/localization |
| Mechanics | Initialization/update order, clocks, packet fields/authority, retries/side effects |

Use affected schema/loader tests and the real client path. Visual/timing changes need
matched captures/interactions; a JSON without a loader or structure-only test is not
runtime acceptance. `cargo run -p xtask -- assets` is fast domain validation;
`--full` checks the release dependency graph and hashes and is for dependency/release
work, not every label edit. Use the command's actual help when changing flags.
