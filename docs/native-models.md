# Native logical models

## Identity and layout

One playable model is one ownership-proven serialized `GameObject` root. Its exact `m_Name` is the
model identity. A route basename, AssetBundle name, PathID, generated hash, slug or transliteration
must never replace that name.

FusionForge resolves the raw source and writes a validated native GLB/dependencies directly
into the semantic paths in [asset workflow](native-asset-workflow.md). No candidate tree,
source JSON, publication sidecar or GPU receipt is a runtime/authoring prerequisite.
Source identity and reversible filename protection belong to the offline converter;
accepted native IDs/remaps remain authoritative when they differ from source names.

The GLB owns the complete Transform hierarchy, mesh bindings, rigid parts, skinned parts, joint
palettes, inverse-bind matrices and representable animation data. Adjacent PNGs are external glTF
textures and exact source mip levels; they are not separate model identities. Internal node, mesh,
skin, material and animation names remain the exact serialized names, even when a genuine source
name contains spaces, punctuation or a historical typo.

Only filesystem-invalid/control characters, trailing dot/space and reserved DOS device filenames
may be minimally protected in a filename. That protection must be reversible through exact GLB
metadata. Case-insensitive, Unicode-normalized and sidecar output collisions fail before any file
is written. Distinct objects that share one name are never merged by name alone.

For example, route `mob/npc_iceking.kfm` has serialized root `m_Name = npc_simon`; its native model
must therefore be `npc_simon.glb`, not `npc_iceking.glb`. A route-to-name mismatch is evidence, not
permission to invent a filename.

## Coordinate, scale and origin contract

Published model artifacts use one explicit Unity-to-native basis change and no other geometric
normalization:

- `H = diag(-1, 1, 1)`;
- positions, normals and translations are `[-unity.x, unity.y, unity.z]`;
- rotations are `[unity.x, -unity.y, -unity.z, unity.w]`;
- scale is unchanged and one Unity unit equals one Bevy unit;
- UV is `[unity.u, 1 - unity.v]`;
- inverse-bind matrices are `H * unity * H`;
- the publisher swaps triangle indices 1 and 2 exactly once for glTF counter-clockwise winding.

The GLB must retain the serialized root and every child Transform exactly after the declared basis
conversion. Auto-centering, fitting to a unit box, height normalization and implicit rescaling are
forbidden. A gameplay model container applies one explicit `Ry(180 degrees)` child rotation so the
authored model's legacy `+Z` forward agrees with the Bevy gameplay root's `-Z` forward.

The supplied historical native full-root audit for the Retrobution catalog resolves 195/195 logical model
roots with zero errors. This set includes character routes plus two unresolved prop-like `mob/`
routes, so the audit does not itself assign character spawn semantics. It found 191 unit-scale
roots, four positive uniform non-unit roots, five
non-zero authored origins, and no non-uniform, non-positive, non-finite or unexpectedly rotated
root:

- `npc_mandroidglasses = 1.2999999523162842`;
- `nano_coco = 1.350000023841858`;
- `nano_holonano = 0.8500000238418579`;
- `nano_johnnybravo = 1.2999999523162842`;
- authored Unity origins: `npc_smallturo = [0.740397, 0, 0]`,
  `npc_billybillyvonbilly = [-0.7833316, -1.793362, -2.641933]`,
  `npc_grubbygrouper = [-0.7605714, 0, 0]`, `npc_penguin = [1.8869622, 0, 0]`, and
  `npc_tuddrussel = [1.3079498, 0, 0]`.

Artifact TRS preservation is deliberately separate from the old character-spawn behavior:

- NPC: replace root position/rotation with the owning gameplay transform and replace scale with the
  selected `m_pNpcData.m_fScale` value;
- Nano: replace root position/rotation but retain the authored prefab scale;
- assembled player: replace root position/rotation and force scale to one.

For example, the current OpenFusion TableData uses runtime NPC scale `1.5` for
`npc_mandroid_disguise`, `1.149999976158142` for `npc_tuddrussel`, `1.2999999523162842` for
`npc_bigbilly`, `1.0` for `npc_arturo`, `1.399999976158142` for `npc_grubber`, and
`0.800000011920929` for all three `npc_penguin` rows. These are per-row gameplay values and must not
be baked into the shared GLB. The semantic table audit finds 3,021 NPC rows with at least one model
route and no null, zero, or negative `m_fScale` among those rows.

The Bevy runtime keeps a character GLB hidden until `SceneInstanceReady`, finds exactly one topmost
descendant whose `Name` equals the proven parentless root `m_Name`, validates finite
translation/rotation/scale and positive authored scale, and only then applies the typed
NPC/Nano/Player replacement above. Bevy's intermediary scene wrappers are permitted only when every
one has identity TRS, so they cannot smuggle in a second origin, scale, or facing rotation.
Descendants of the matched logical root do not compete: `nano_johnnybravo` legitimately has a
nested mesh transform with the same `m_Name`. Two topmost sibling candidates remain an error.
Missing or ambiguous topmost exact names, a non-identity intermediary wrapper, a non-finite,
degenerate, or materially non-unit quaternion, an invalid authored scale, or an invalid NPC row
scale leaves the scene blocked and hidden. The stable `Ry(180 degrees)` character container is outside that imported root and has
identity translation and scale.

`logical_model_gpu_preview` also has a reusable runtime-root acceptance mode:
`--character-kind npc|nano|player --true-root <m_Name>`, plus mandatory `--npc-scale` for NPCs and
an optional standalone `--report`. It instantiates the real scene through the gate above, waits for
transform propagation and a named animation sample, and records authored/applied/actual local TRS,
gameplay/container/root world TRS and skinned world bounds. The aggregate
`target/character-runtime-gpu-v2-identity-wrapper.audit.json` was regenerated from scratch after
the identity-wrapper and unit-quaternion gate was enabled. It binds nine new per-model JSON reports
and nine new GPU PNGs by SHA-256 and passes 9/9 with zero violations:

- NPC table scales: `npc_smallturo = 1`, `npc_billybillyvonbilly = 1.3`,
  `npc_grubbygrouper = 1.4`, `npc_mandroidglasses = 1.5`, `npc_penguin = 0.8`, and
  `npc_tuddrussel = 1.15`;
- preserved Nano authored scales: `nano_coco = 1.35`, `nano_holonano = 0.85`, and
  `nano_johnnybravo = 1.3`;
- every gameplay root is identity, every applied/actual imported root has zero translation and
  identity rotation, and the only character-facing half-turn is the unit-scale visual container;
- every intermediary Bevy loader wrapper passed the identity-TRS production gate and every
  authored root quaternion passed the finite unit-quaternion gate;
- every final `stand1` sample has finite non-degenerate skinned bounds, a non-empty GPU screenshot,
  and zero material or shader errors.

The Player branch is covered by the same acceptance contract at unit level: it discards authored
root translation/rotation/scale, applies zero/identity/one, and must not introduce a second
half-turn. No assembled-player GLB is included in this nine-model exception set.

This production gate is exercised by the native GPU harness and feeds the same accepted semantic
character contracts used by the runtime. World discovery now uses the domain-owned
`ffone.runtime-world.v1` registry and native terrain/scene payloads. The evidence above and the
completed registry migration are not claims of
full character rendering, world collision, traversal or gameplay parity.

World scenes, static models, attachments, colliders and terrain do not inherit this character-root
replacement policy. Their parent-relative Transform chain and pivot are separate acceptance gates.
The world exporter serializes native local position/quaternion with unchanged scale, mirrors only a
collider center's X component (dimensions remain dimensions), and transforms baked normals with the
inverse transpose. A singular normal transform omits normals with an explicit diagnostic instead of
inventing a direction. The `Map_00_01` golden world origin is `[0, -300, 512]` in native axes and has
no character-facing half-turn.

## Player equipment and scripted attachments

The 195-root logical-model audit is not equipment coverage. The current semantic plan has 3,424
equipment rows and 1,695 unique `wear/*.nif` routes, but zero ownership-proven equipment model
proposals. The old flat diagnostic GLBs reduce a prefab to one mesh node; they cannot be promoted
because skinned clothes lose their hierarchy, weights, bind poses, and shared-skeleton mapping.

The native pipeline must keep four distinct policies:

1. Standard Hat, Glasses, LeftPistol, RightPistol, Back, and Zipline attachments bind to the exact
   `helmet01`, `glass01`, `Lweapon01`, `Rweapon01`, and `back01` skeleton paths. Their item-local
   position is zero, rotation is Unity `Euler(90,0,0)` converted through `H`, item scale is one, and
   the socket scale is forced to one. Zipline shares `Rweapon01`. They already inherit the player
   character container, so another `Ry(180 degrees)` is forbidden.
2. Head/mask/shirt/pants/shoes and equip-type-one backs participate in the combined skinned player
   mesh. They require exact bone-index remapping to the common player skeleton; spawning them as
   independent GLBs is incorrect. Height/shape are sampled additive skeleton animations, not a
   root-scale shortcut.
3. Vehicle models attach to the `vehicle` socket with the socket rotation followed by X+90 degrees,
   but retain their authored root scale because the old vehicle path does not force local scale one.
4. `NpcCustomization`/event cosmetics have explicit local offsets. Real source values include the
   non-uniform scale `[2.55,2.25,2.65]`, negative scale such as `[-0.25,-0.25,-0.25]`, and a separate
   zero-scale tail-hiding operation. These are typed scripted exceptions, not permission to accept a
   negative, zero, or non-uniform ordinary model root.

Internal attachment origins remain authored. For example, representative glasses and weapon
prefabs contain X-90-degree children and non-zero local offsets; the attachment-root X+90-degree
rotation compensates them. Auto-centering or flattening that hierarchy breaks the original fit.

The central character preview and selection portraits sample the existing shared-skeleton
`height_Add`, `shape_Add`, `height`, and `shape` clips through `NativePlayerBodyShape`.
Height selects `1 - index / 4`, build selects `index / 2`. The static poses remain layered over
the equipped idle, and creator changes update the same skeleton in place. Recovery of a preview
animation graph restores the body layers as well as idle. The gameplay animation adapter keeps
its own existing locomotion/body composition. See the Editor-owned investigation in
`../FusionForge/docs/legacy/ffone/player-body-shape-20260912.md`.

### Historical player-cooker checkpoint

The early Test Ser cooker is not the current direct-model workflow. Its partial skeleton
inventory did not establish a completed player GLB. Source-specific limitations are retained
in FusionForge [checkpoint](../../FusionForge/docs/legacy/ffone/player-cooker-checkpoint.md);
read them only when repairing that converter. Do not recreate its manual JSON inputs.

## ShaderLab texture defaults and NPC runtime overrides

Native material metadata preserves every ShaderLab `2D` property in exact `Properties` source
order, even when that slot is absent from Unity `m_SavedProperties`. The typed values distinguish
`"white" {}`, `"black" {}`, `"gray" {}`, `"bump" {}`, `"red" {}` and `"" {}`. Duplicate properties,
unknown built-in names and non-canonical right-hand sides block publication. Saved texture
assignments always override the declared default.

The Bevy material loader has audited fallback equivalence for exactly `builtinWhite` and
`builtinBlack`. An absent/null saved slot with one of those exact defaults receives an opaque 1x1
RGBA texture; `builtinBlack` is explicit rather than accidentally reusing Bevy's white fallback.
`_MainTex` is sampled as sRGB, while `_ShaderMap` and `_BumpMap` (including FusionEffect slots) use
linear data. A saved texture assignment always wins. `blank`, `gray`, `bump`, `red`, empty and
unknown defaults remain provenance only and never acquire an invented color, so a required
unresolved slot still blocks publication.

Legacy `NpcMoveController` can later replace NPC textures from table-driven runtime customization.
That dynamic assignment is a separate pending gameplay policy: the static GLB records the exact
prefab material/default state, and this fallback work does not pretend to implement or bake those
per-NPC table overrides. The historical offline XDT gallery applies them diagnostically: for visible
non-HNPC rows it assigns Texture1 to exact material names containing `main` and Texture2 to names
containing `sub`, matching `NpcMoveController.SetupNPC` without mutating the published GLB.

## No-loss gates

For a changed converted model, validate these properties in memory and through the native
consumer. Reports are optional requested evidence, not required intermediate files:

1. Offline ownership proof finds one self-contained root and complete pointer closure.
2. Publication matches source/published feature counts and leaves no unresolved source feature.
3. Independent GLB re-decoding matches canonical SHA-256 digests for hierarchy, geometry and
   weights, skins and inverse binds, standard animation values/tangents, and animation metadata.
4. The tree audit rejects extra model formats, orphan PNGs, generated identities, broken links or
   stale reports.
5. Native Bevy GPU evidence binds the live GLB and screenshot hashes to exact material, mip,
   skin/joint/IBM and fixed named-animation runtime facts.
6. Controlled reference-client visual review remains a separate 1:1 gate.

Recorded controlled coverage was `npc_dexter`, `fusion_dexter` and `npc_max`: 3 GLBs, 207 hierarchy
nodes, 8 mesh parts, 5 skins, 176 joint references and inverse-bind matrices, 5,133 weighted
vertices and 47 clips including 2 metadata-only clips. Both structural and automated GPU evidence
audits pass 3/3; this is not the complete catalog.

The separate scale/origin exception candidate covers nine additional exact roots: the four
non-unit-scale roots and the five non-zero-origin roots listed above. Its structural audit passes
9/9 with 504 nodes, 14 mesh parts, 10 skins, 465 joint/inverse-bind
references, 15,255 weighted vertices and 97 clips. The refreshed native GPU evidence also passes
9/9, while deliberately retaining `visualParityPending: true` and `publishable: false`.

`npc_mandroidglasses` exposed a preview-only bounds bug without changing the source or GLB. Its
skinned `shirt` mesh node has an authored native X translation near `-5.9600935`; applying that node
transform to its static AABB double-counted a transform already represented by the inverse-bind
matrices and produced a false bound near X `-8.25`. Preview framing now evaluates every skinned
vertex with the same `jointGlobal * inverseBind * vertex` matrices used by Bevy 0.17.3 and reserves
transformed AABBs for rigid meshes. At `stand1` 50% the resulting finite bounds are
X `[-0.5451987, 0.5626266]`, Y `[-0.0110608, 1.2614073]`, and
Z `[-0.5042557, 0.2777811]`. This correction changes only diagnostic framing; authored hierarchy,
origin, scale, skin weights, joints and inverse-bind matrices remain byte-bound to the same GLB.

## Runtime character paths in TableData

Accepted character closures are installed below physical plural taxonomies:

```text
assets/game/characters/
  nanos/<id>/
  npcs/<id>/
  mobs/<id>/
  fusions/<id>/
  shared/textures/
  shared/runtime-textures/
assets/game/data/tables/xdt.json
```

The native_asset_routes table, m_pCharacterModelData array, maps semantic model IDs
to exact GLB paths, animations and optional collision paths. Migration inventories and
hashes are archived in FusionForge docs; the client does not open them.
The category chooses the physical directory; accepted semantic IDs retain their
original namespace when a model is reclassified. Sharing a model between gameplay
roles does not move it into `shared`: NPC models belong to `npcs`, ordinary mobs to
`mobs`, and Fusion character counterparts to `fusions`. Reusable texture payloads
remain in `shared`; GLBs reference them without copies. Existing variants and
logical-name aliases remain distinct, including alternate NPC packages nested
under the same stable package slug. Exporter family and source-owner aliases do
not choose the runtime folder.

Old `characters/catalog.json`, `characters/registry.json`,
`characters/schema-upgrade-violations.json`, and singular
`characters/{nano,npc,mob}` layouts are not current runtime contracts. Structural/GPU checks operate on native output. Requested reference reports belong in
FusionForge documentation, not runtime data or a required intermediate workspace.

The old flat `assets/game/models` dump has been removed. Its generated-suffix
per-Mesh GLBs did not prove hierarchy, materials, skinning or animation
ownership; historical recovery copies must never be used as the runtime registry or required by
the direct converter.

Static map geometry now lives only in reusable packages below `assets/game/objects`. Each
package owns its visual and collision geometry, while tiles store placement matrices. These files
are validated through `map/catalog.json`; a second `models/world` tree must not be recreated.

The recorded catalog contained 3,474 normalized KFM/NIF routes. Physical identity is accepted
only when every occurrence has the same serialized-asset SHA-256, PathID and object type. This has
proved 83 real multi-owner aliases and rejected 175 same-route groups with distinct physical
targets. Unresolved or conflicting routes remain blocked instead of being silently collapsed.


## Focused checks and diagnostic classifications

Use `logical_model_gpu_preview` for a changed native model; full NPC/Nano galleries are
explicit audits, not ordinary build steps. Run legacy resolution through FusionForge's
[documented converter](../../FusionForge/docs/conversion-workflow.md), without intermediate
projects or manual metadata assembly. Unsupported gallery/repair coverage stays explicit.

Preserve these recovered distinctions: no matching material slot is
`no_matching_material_unity_noop`; non-HNPC classes >=100 are `hidden_location_marker`
(model/name/damage hidden). Visible scale-0.01 `ObjectNPC1` rows are
`primary_interaction_placeholder`, not the named Time Machine/Magic Tree/Guide Changer art.
Missing/ambiguous models/textures remain diagnostics, never guessed donors. Nano default
appearance without a table row requires both canonical `<model>` / `<model>_face` and
proven `main` / `sub` roles. Deduplicate a requested gallery only by complete native
appearance identity and exact samplers; keep runtime extensions distinct from source parity.
