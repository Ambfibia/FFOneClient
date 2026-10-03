# Gameplay HUD and targeting

Owners: `gameplay_ui.rs` (persistent HUD), `tutorial_overlay_ui.rs` (independent tutorial),
`group_ui.rs` (group panels). Tutorial presenters must not replace the persistent HUD.

Damage trails hold for 0.4 seconds then smooth independently for player, enemy, group,
escort, active Nano and Nano wheel. F1 denial is a keyed two-second combat notice.
Target HP/identity comes from decoded authority; TableData supplies maximum HP, name,
level, class, team and authored height. Hide contradictory/missing targets instead of
inventing data. Resolve target name keys before `Localization::Apply`.

The avatar portrait uses an assembled 3D render target, not a static bitmap. Target
portraits use their own admitted actor/camera; do not reuse the avatar surface.
NPC subtarget camera uses ground root Y + 0.6 × authored height and row radius; avoid
a second one-unit spawn lift. Hide HUD ownership while NpcIcon/Vendor owns the service.

## Icons and effects

Retry attachment after renderer readiness; classify HNPC roots before attachment.
Do not condition attachment on the local player or a one-second mission refresh.
Priority: unique GameIcon attachment → Bip01Head with Unity offset (-0.6,0,0) →
root at 0.9 × authored height with Unity X rotation 90°. Quest symbols replace ordinary
game icons and restore them afterward. ES0/106 are inactive/empty, not substitute art.
Ordinary nonzero `m_iEffect` excludes classes 0/6; race icons 13/14 depend on race state;
17 uses registered ES446 → ES811. Bank owners 686/1186/1677/2215 retain scoped
ES679/683 exceptions, not a global nonzero-effect fallback.

NPC speech is emitted semantically before balloon-visibility gating. Chat drains after
NpcSpeech, then applies the NPC-message option. Group panels do not own player bubbles.
Target-localization and rendering caches must clear on owner loss, not merely reuse IDs.

## Fusion Matter and tutorial

FM ring angle is 360 × clamped fraction. Draw base, rotated right black mask, then left
black at ≤180° or right regular at >180°. Ring Rect=(12,3,176,176), pivot=(99,91),
local pivot=(87/176,88/176). Empty is black, not transparent.
Tutorial player kills grant 30 FM each through the local virtual-server path,
including the first three Spawn and Kerber; scripted deaths grant none.
Tutorial handoff holds 220/220 for 27 seconds then resets to zero; proximity does not
cancel it. Tutorial overlay requires Tutorial + live player + no completion request.
Exit clears overlay and auxiliary prompts immediately. Keep the existing HUD/cursor
owner ordering; stale FM cursors cannot override mission prompts.
Tutorial combat follows incoming/outgoing activity with a five-second inactivity
timeout, independently of chapter/proximity. The green frame uses the live event-scene
flag to hide during cinematics; DANGER yields its slot to the current target panel.

For Nano channels use [Nano HUD](nano-hud.md); mission widgets use [journal](mission-journal.md),
map markers use [minimap](minimap.md), and chat uses [chat](chat.md).
