# Gameplay corrections — 2026-09-18

The client now connects race authority to streamed ring visibility and collection.
The instance-map packet supplies course bounds. Rings remain hidden until race
start succeeds, use stable identities across streamed reloads, and disappear on
collection. Pending requests suppress duplicate pickups; server replies supply
the count. Runtime-managed visibility prevents ordinary model admission from
revealing inactive pods. Pickup tests use the authored ring sphere and player
capsule.

Weapon cycling moves only the two equipped hand slots, including holstering a
single weapon. It no longer searches the bag. Cannons read configured movement
keys independently of the walking input gate, while holding/releasing the
configured jump action continues to charge/fire the launcher.

Rockets and grenades now collide with native terrain and authored solid meshes.
Grenades bounce and expire; rockets explode on contact. Swept NPC tests include
the NPC's authored height and radius. Only the firing client sends the validated
bullet slot, impact position, and NPC list. Empty explosions also release the
server bullet slot. RustyFusion retains damage authority. Both rocket-hit and
grenade-hit replies update NPC health and hit/death animation, including the
grenade-hit reply RustyFusion uses for rockets.

Player health loss plays the native `woundupper` clamp animation on the upper
body. Both genders have the clip. Healing, initialization, and lethal damage do
not start this reaction; the upper layer releases after completion.

Closing vendor, bank, transportation, and Croc Pot services queues the NPC's
localized farewell through the existing semantic voice catalog. Hoverboard
engine gain is 0.5; other vehicle engine gains remain 1.0 before the user's SFX
setting is applied.

Verification logs and renderer captures are under ignored
`target/performance/gameplay-bugs`. The opt-in full-client fixture is
`FFONE_PERF_GAMEPLAY=1` with `FFONE_PERF_OUTPUT` set to an ignored report directory.
It verifies actual damage playback and release without connecting an account or
writing user settings. These offline checks do not constitute a live server playtest.
