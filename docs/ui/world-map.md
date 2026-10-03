# WorldMap

Preserve three world zooms plus local map, Future/Past/Other-instance selection,
center scale=max(1,height/768×1.05), inverse input transform, scan bands0.15/0.5 and
transactional finite-coordinate guards. Local map viewport=950×624; apply player
Rect after panning. Named world locations retain their source visibility gates.

Text styles: 9 Manrope, offset(0,+5), source1119; 8.4 RussianRail uppercase after
locale, offset(−1,+5),1066; 11 Manrope,offset(+2,+4),1018. Unity DrawTexCol neutral
gray maps to native white, not sRGB0.5 darkening. Do not change geometry for translation.

Presence request 0x130000A7 is eight bytes with clock. Reply 0x31000137 has eight-byte
header and int32 count≤1020; any nonzero bClear is true. Reply has no timestamp: retain
the independent server clock, not a fabricated echoed one. Global client NPC rows
support distant mission targets; do not rely on streamed entities. M/action22 resets
mode lifecycle. Twelve WorldView filters differ from MyView/minimap's eligible-all rule.
Open only for server map 0 or an InstanceTable EP map; reject other instances on
both the map key and NanoCom route. Ordinary mob rows require the authoritative
0x1000 radar condition even in local Type4 view. Warp/regen clears old markers.
The shard's placement snapshot contains friendly NPCs only; radar mobs use
authored overworld placements plus observed live mobs, still gated by the
instance-scoped present-type reply.

Transport hover enlarges/raises routes: green SCAMPER, purple Skyway, peach Woosh;
locked routes are gray/dashed. Active XCom uses the spiral. Six-color right-click
waypoints persist independently of quest targets. Drag must not multiply displacement
by delta time. Contextual Help now uses FirstUse15 via the shared Game Guide; old
“unconnected help” notes are superseded, not another implementation task.
