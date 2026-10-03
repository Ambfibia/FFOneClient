# Nano HUD

The active strip is 154×29 with a 104×4 stamina bar; the wheel is a separate owner.
Animated wheel portraits use 72×108 rectangles at y=-24 and 144×216 render targets.
The off-axis camera extends the former 72×72 view upward by 36 logical pixels,
preserving model scale and the bottom edge at y=84. Only the three HUD rigs bypass
CPU frustum culling, which does not account for Bevy's SubCameraView. Journal
portraits retain their square view. Skills, stamina, affinity overlays and battery
counters retain their rectangles and draw above the portraits.
Inactive backgrounds are 42×6 at x=22/99/176,y=17; fills are 40×4 at
x=23/100/177,y=18. Clamp stamina / `m_iNanoBattery1`; preserve the source active-slot
and 0.99 visibility rules. Stretch the complete gradient to 40×fraction, not a UV crop.

Cooldown overlays are 26×26, shifted one pixel left. Remaining fraction runs 1→0,
starting with the coarse completed overlay followed by five-degree ray sectors.
Start on an accepted local request, not SUCC. Buttercup skill 1 cool=80 means eight
seconds. Keep per-equipped-slot timers across active-Nano switching; clear only on
skill identity changes/session end. Decode full pack-4 SUCC and viewer USE eST tails
before applying any post-state. A reply must not restart the timer. Same-map moves
snap; cross-map moves follow normal teleport ownership.

Preserve exact skill 1/2/3 tune selection and accepted startup Call skill. Rebuild the
native graph only for a real model change. The ten-path sampler override retains
sector/ray/stamina bilinear repeat without mips. Do not globally change image sampling.
Potion/Boost counts remain four digits (`0000`), UpperCenter, font 9, line height
13.71, +1 Y inside the same Rect; preserve low-count color and icon ownership.
