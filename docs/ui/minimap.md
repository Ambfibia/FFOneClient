# Minimap and mission markers

Preserve 16-tile (4×4) numbering, PNG vertical flip and stitching across up to four
tiles. Zoom range is 4..16, default 8; Enter-menu buttons control the same owner.
The player portrait is an independent assembled 3D surface.

Ordinary NPC discovery retains the source `!bViewMob` exclusion of
`npcType == 0 && sound != 2`, strict distance < radius, regular-icon size and
advance-icon 16 priority before regular icon 15. Use the declared 34-icon catalog,
not team-based guesses. Tutorial markers are a separate branch. Global NPC rows
support distant mission targets; do not limit discovery to nearby runtime entities.

Mail check starts 60 seconds after ready and repeats every 600 seconds. Positive
count plays Email_Arrived twice and latches the signal. The 19×14 icon sits at Nano
local (145.5,158), uses two seconds of abs(cos(2πTime)) alpha, and hides with the menu.
New NPC/invitation letters reuse this alarm without polling/resetting it every frame;
see [email](email.md).
