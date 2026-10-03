# Map assets

Native map catalogs, tiles and shared payloads are published below `assets/game/map`; reusable
object packages are the top-level sibling `assets/game/objects`. The runtime has one tile
namespace: `map_XX_YY`. The nine tiles that were historically labelled `tile_XX_YY` are ordinary
map tiles; their old identifiers are retained only in `legacyAliases` for migration. Tutorial flow,
barriers and other scripted interactions are behaviours attached to a map tile, not separate map
assets.

## Layout

```text
assets/game/
  objects/<category>/<resource-set>/
    set.json
    textures/<atlas>.png
    models/<object>/
      object.json
      visual.glb
      collision.glb
  map/
    catalog.json
    shared/
      terrain/layers/<name>/mips/...
      terrain/layers/<name>_variant_02/mips/...
      effects/...
      projectiles/...
      environment/skyboxes/{past,future}/...
    tiles/map_XX_YY/
      tile.json
      scene.json
      objects.json
      behaviour.json
      terrain/...
```

`catalog.json` is the map entry point. Its `resourceSets`, `geometry`, `objects` and `tiles` arrays
describe the complete reference graph. There is no physical tutorial/world split, no `Prefab`
directory and no global map-model texture dump.

Each logical map object remains one indivisible package: its `object.json` owns every visual and
collision part, so collision cannot be separated from the reusable object accidentally. Objects
that use the same texture atlas are placed in one resource set. The set owns exactly one copy of
the atlas and may contain several independent object packages and models. An object may contain
only a visual, only collision, or both. A tile stores object IDs and exact native-world matrices;
it does not copy the object GLB or textures.

Physical package names are readable and never contain content-hash suffixes. The most-used
package owns the plain `<object>` route. If primary evidence proves that several different objects
have the same legacy display name, additional packages use sibling routes
`<object>_variant_0002`, `<object>_variant_0003`, and so on. Exact content identity and source
ownership remain in `object.json` and `catalog.json`, not in the directory name.

Terrain heightmaps, weights, lightmaps, gameplay attributes, details and environment remain owned
by their tile. Reusable terrain layer textures are content-addressed below `map/shared/terrain` and
referenced from every tile that uses them. Model atlases are not terrain resources: they live in
their object set. Identical atlas bytes cannot appear in two sets; the verifier rejects that state.

The logical categories include `attractions`, `collision`, `effects`, `furniture`, `gameplay`,
`infrastructure`, `nature`, `props`, `signs`, `structures`, `unclassified` and `vehicles`.
Recovered prefixes and families are kept as source evidence; notably, `WD` is not rewritten to
the visually similar `DW`. Names without sufficient evidence remain under `unclassified/MISC`.

All transforms are already in FFOne native coordinates: `H = diag(-1, 1, 1)`, with one Unity unit
equal to one Bevy unit. Local object geometry is recovered by applying the inverse of the exact
source world matrix. Normals use the matching inverse-transpose transform, while the audited
winding, UVs and materials are retained.


## Validation

New native tiles and editor placements edit `objects`, `map/tiles` and `map/catalog.json`
directly. Preserve every reference, current byte/BLAKE3 guard, placement and hash binding.
Do not recreate a `world/`, `models/`, `props/` or `tutorial/` runtime tree. Validate the
complete changed reference closure, including URI safety and object/texture ownership.

Legacy map organization remains FusionForge tooling. Historical `organize-map` /
`organize-resource-sets` / `normalize-object-routes` staged commands are not the supported
native authoring workflow and do not prove a direct world adapter exists; check its
[actual conversion coverage](../../FusionForge/docs/conversion-workflow.md).
