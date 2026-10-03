# Editable content: accepted target

Incremental design, not a claim that every consumer has migrated. Current native files
and validation remain as described in [native assets](native-asset-workflow.md).
Do not relocate `assets/game` or regenerate its multi-version TableData implicitly.

## Target package layout

~~~text
assets/
  packages/
    ffone.core/
      package.ffpkg.json
      npcs/<id>/npc.ffdef.json
      nanos/<id>/nano.ffdef.json
      items/<kind>/<id>/item.ffdef.json
      localization/{en,ru}.json
      ui/...
      audio/...
    community.example/
      package.ffpkg.json
      npcs/...
~~~

An optional external package root, normally `mods/` or a command-line path, uses the same layout.
A package contains editable source assets, not a generated complete file inventory.

### Package manifest

~~~json
{
  "schema": "ffone.package.v1",
  "id": "community.example",
  "version": "1.0.0",
  "protocol": "native-v1",
  "requires": [
    { "id": "ffone.core", "version": "1.0.0" }
  ]
}
~~~

The manifest deliberately has no per-file hashes, source-build provenance, Unity identity, or
machine path. A release lock file may contain hashes, but it is derived output.

### NPC definition

~~~json
{
  "schema": "ffone.npc.v1",
  "id": "my_npc",
  "networkId": 20001,
  "assets": {
    "model": "model.glb",
    "collision": "collision.json",
    "portrait": "portrait.png"
  },
  "data": {
    "nameKey": "npc.my_npc.name",
    "animations": {
        "idle": "stand1",
        "walk": "walk",
        "run": "run"
    },
    "gameplay": {
      "team": 1,
      "service": null
    }
  }
}
~~~

### Nano definition

~~~json
{
  "schema": "ffone.nano.v1",
  "id": "my_nano",
  "networkId": 201,
  "assets": {
    "model": "model.glb",
    "portrait": "portrait.png"
  },
  "data": {
    "nameKey": "nano.my_nano.name",
    "skills": [
      { "slot": 0, "skill": "community.example:skill/basic_damage" }
    ]
  }
}
~~~

### Item definition (`items/weapon/example_blaster/item.ffdef.json`)

~~~json
{
  "schema": "ffone.item.v1",
  "id": "example_blaster",
  "networkId": { "type": 0, "number": 60001 },
  "assets": {
    "icon": "icon.png",
    "maleModel": "male/model.glb",
    "femaleModel": "female/model.glb"
  },
  "data": {
    "nameKey": "item.example_blaster.name",
    "equipSlot": "hand",
    "stats": { "level": 1, "rarity": 1, "damage": 10 }
  }
}
~~~

Numeric protocol identifiers stay explicit because saves, databases, packets, and the server share
them. Automatic allocation is forbidden.

The runtime canonical identity is derived, not authored:

~~~text
<package-id>:npc/<local-id>
<package-id>:nano/<local-id>
<package-id>:item/<category-directory>/<local-id>
~~~

The item category is derived from its directory and is not authored in the leaf JSON.

## Discovery and conflict rules

1. Scan only direct child directories of each configured package root.
2. Treat a direct child as a package only when it contains `package.ffpkg.json`; ignore direct
   children without a manifest and never discover nested manifests through them.
3. Reject package IDs that collide case-insensitively.
4. Resolve dependencies as a directed acyclic graph. Missing dependencies and cycles are fatal.
5. Use topological order with lexical package ID as the deterministic tie break.
6. Scan only known leaf routes such as `npcs/*/npc.ffdef.json`,
   `nanos/*/nano.ffdef.json`, and `items/*/*/item.ffdef.json`.
7. Resolve asset paths such as `model.glb` relative to the owning definition leaf. Reject `./`,
   `..`, absolute paths, backslashes, and every path escape.
8. Validate the complete candidate registry before atomically replacing the active registry.
9. Reject duplicate canonical IDs and typed network IDs.
10. Do not use hidden priority or last-writer-wins behavior.

A replacement must be explicit:

~~~json
{
  "schema": "ffone.npc.v1",
  "id": "custom_dexter",
  "networkId": 123,
  "replaces": "ffone.core:npc/dexter"
}
~~~

The first implementation supports full-definition replacement only. The target must come from a
declared dependency and the replacement preserves its protocol identity. Generic JSON merge patches
are intentionally excluded because array semantics are ambiguous.


## Reload, server and capability boundary

Minimum reload guarantee: restart without Rust compilation/cook. Live reload is optional
and transactional; server-shared IDs, collision/topology changes may require reconnect.
Packages configure registered capabilities; genuinely new mechanics need Rust or a future
sandboxed API, never arbitrary native libraries.

Client/server must select the same package set and content-set fingerprint. Native packages
project to the server's compatibility XDT through FusionForge; XDT is not long-term authored
truth. Preserve the current TableData baseline while overlaying typed definitions in memory.
Remove it only after every consumer migrates and all combined rows are represented losslessly.

Migration order: package registry beside the existing tree → localization/audio → NPC/Nano/items
→ matching server projection → editable UI → maps last. Retire catalogs only after their last
consumer; preserve local ignored user data. Cache/release products remain optional, disposable
and separate from editable truth.
