# Guide Changer

`guide_ui.rs` owns rendering/domain and `guide_runtime.rs` owns authority/correlation.
1036×654 center-pivot layout with the source scale. Mentor order/wire IDs:
Ben4,Dexter2,Mojo3,Edd1; change cost exactly zero. Preserve portraits, TableData
copy, Past-Warp warning, initial selection, message155 current mentor, confirmation,
success/failure154 and dismissal ownership. PcLoad mentor/count is lossless; sentinel5
and clean versus OpenFusion count-1 reply semantics remain distinct.

NpcIcon StartGuideChage categories18..22 pass IDs1..5; category23 depends on payment.
Target using camera-facing cone, authored radius/height, static-world LOS, strict
per-row sight range and dead/class gates; static AI-type0 Guide NPCs are not excluded.
Revalidate live team1 identity before opening; use its subtarget camera/service row
and Guide special state16. Handle mentor replies transactionally before generic FM.

First change validates warp76/NPC1425 at [373330,442600,−5700], resolves one unique
live NPC, waits1.5seconds, sends24-byte warp, decodes36-byte success or4-byte failure.
Apply authoritative position/map/Candy; failure message173. Close/death/disconnect/
world reset release modal ownership and lower gameplay/social input gates.

Help now routes through shared Game Guide (event44); the old missing-help note is
superseded. Recorded remaining boundaries: Guide-mission events12/30, warp effect394,
Dexbot_Warp/fade/DongReady/movement-send choreography, inventory mutation for eIL≠4,
full NPC visuals and primary-live/failure replays. Category23 News/payment limitations
are in [Upsell](upsell.md); do not fabricate absent LeaveFuture modes.

Draw-order authority: primary `main.unity3d` SHA256
`01b544976b2d54355507cf30fe6dfada2b476b92b209a3d47c1499669ed9b4ef`,
`sharedassets0.assets` component1415 (raw SHA256
`061d3b9441f40dc204c256e514ee6f655385a691520a624db54752c6a78f10e8`),
`Assembly - CSharp.dll`, `cnGuideMode.OnChange`, token `0x06000336`.
IL_0285..0310 paints the card and current frame before the clipped card group;
IL_0317..0366 draws icons at x=-width/2 relative to the card group. This IL
alone does not confirm the old renderer's texture clipping behavior. The accepted
native correction keeps each badge centered on the card edge and paints it outside
the clipped text group, so the full icon is visible. IL_04c2..05a7 paints the
selected effect before the portrait. Texture integer division precedes float conversion
(effect267/2=133, Mojo281/2=140). Confirmation art above the dialog and the 50% black
veil are intentional (IL_08c8..0a38). Static IL establishes this order and arithmetic;
paired running-original captures are still required for complete visual acceptance.
Replay: FusionForge `inspect ../builds/retrobution-20260821/main.unity3d --assembly
"Assembly - CSharp.dll" --class cnGuideMode --method OnChange --limit 1`.
