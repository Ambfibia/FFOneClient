# Mission journal and tutorial prompts

Selected-objective copy uses the direct 50-pixel label with MiddleLeft alignment;
the description is top-aligned GUILayout in its viewport, not vertically centered.
Offer Accept Rect=(347,562,218,85), zero padding, MiddleCenter. Completion
Rect=(327,562,233,70); parent and label share it. Offer Cancel is based on
(16,578,190,25). Nano group stays 555×245 with a 128×128 portrait.

Chapter 05/06 `SubText2` begins at y=0, unlike quarter-height `SubText`. Auxiliary
prompts must not appear during the first 4.5 seconds. Cursor=(3/4 width−246,
1/2 height−41); stale Fusion Matter cursor ownership cannot override it.
Accepted native task 2250 handoff enters OFFER, then ACCEPT sends zero-choice
TASK_END, not Reward. Later talk continues automatically; this is an owner extension.

Draw unselected tabs before selected. Tab1 overflow=(0,20,0,2), visual 120×32;
Tab2=(30,30,0,0), visual 160×30; hit regions stay 100×30. Preserve cyan/black states.
The black scroll mask is Shadow 412, not bright 470. Selected rows are 86 high with
68×68 portrait at (10,y+10); unselected rows are 56 high, no portrait, title
Rect=(15,y+5,280,25).

Portrait lookup is NPC → declared icon row, never inferred identity. Example:
task 451 / NPC 2555 / icon 50 → type 4, number 49 (`npcicon_49`). The accepted Active
row-click extension updates viewed and tracked mission; pending requests block it,
completed rows remain read-only, and the bottom button remains available.
Task 451 reward 106 (type 5,id 24) is fixed/server-granted with choice 0/0: it must
not block selection. Only declared choice rewards use bitmasks; completion return is 3.

Mission targeting must use the global client NPC table rather than only streamed
entities, retaining source order and owner predicates. For map display use
[minimap](minimap.md) and [WorldMap](world-map.md). Mail invitations have a separate
[email](email.md) owner. No journal action invents rewards or player-level changes.
