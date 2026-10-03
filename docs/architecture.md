# Runtime architecture

Bevy owns rendering/input/audio/ECS, not packet bytes, IDs or server authority.
One `AssetLocator` opens the native `assets/game` tree; content ownership and validation
are specified once in [native assets](native-asset-workflow.md).

## Crate ownership

| Crate | Responsibility |
| --- | --- |
| `ffone-runtime-contracts` | Serialized creation/player-rig/tutorial-effect schemas; no filesystem mutation/import |
| `ffone-packaging`, `xtask` | Release dependency graph, loose-file synchronization, validation |
| `ffone-protocol` | 0104 layouts, framing/encryption, golden fixtures |
| `ffone-net` | Login/shard sessions, bootstrap and gameplay transport |
| `ffone-client-foundation` | Project-asset validation, coordinates, semantic audio |
| `ffone-client-network` | Background network worker and Bevy bridge |
| `ffone-tutorial-core` | Engine-independent tutorial state/choreography |
| `ffone-client` | Bevy composition and coupled presentation/gameplay |

FusionForge owns offline `ffone-content` / `ffone-asset-pipeline`; they are not runtime
workspace dependencies. The boundary is final typed native files, not staged projects.

## Network and scheduling

Blocking TCP stays off the render thread. A command worker and dedicated shard reader
share one synchronized outgoing encoder, including heartbeats. Preserve packet order,
initial PC/NPC/transportation/shiny buckets and malformed payloads. Passwords are
redacted from Debug and discarded after login.

`Bootstrap → Login → CharacterSelect → World` follows network events; world admission
requires accepted loading-complete. Gameplay ordering remains:
`input/network → target/animation → remote simulation → local movement/collision/packet intent → camera`.
Remote movement preserves extrapolation, `dt * speed` correction, `dt * 8` rotation
and the 0.7-second lag stop. Reference collision/traversal completeness is a separate gate.

## Content and builds

Audio/character routes use TableData `native_asset_routes`; `NativeAudioCatalog` is an
in-memory exact-path index, not a generated runtime inventory. Maps use `map/catalog.json`.
Text and voice language are independent; [localization](localization-and-voice.md) owns
fallback rules. Code-only builds do not copy/cook the asset tree. Release synchronization
invalidates its receipt before writing and writes it last; an interrupted copy must not
appear verified. Development and release consume the same loose-file layout.

A recovered per-Mesh GLB does not prove hierarchy, materials, skinning, animation,
colliders or LOD. Only validated logical models belong in the runtime. Tests distinguish
protocol/structural correctness, native loader acceptance and actual visual/interaction parity.
