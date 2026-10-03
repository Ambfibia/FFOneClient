# Shared item cards and service controls

Inventory, Bank and Vendor resolve semantic content text rows even when they differ
from protocol item IDs. General-item uppercase runs after localization and is removed
when the same entity changes type. Names/descriptions fit fixed rectangles with approved
fonts; text children pass pointer focus to controls. Numeric input remains editable,
not auto-shrunk. Bound text children, not layout parents, receive rating colors:
higher point/group/defense green, lower red, equal normal. Nonweapon range is blue N/A.

Combined inventory uses appearance text/icon, Special rarity, badge and red Not tradable.
Badge=(52,52,26,26) over a 64×64 icon, only for positive signed appearance words on
weapon/shirt/pants/shoes. Vehicle cards hide combat/ratings, show localized Speed and
speed class; changing back restores ordinary fields. Catalog rental time is duration;
owned time is Unix expiry. Preserve the accepted readable numeric date, 24-hour time
and UTC offset, ordinary expiry below level. Vendor price uses the existing Taros icon.

Quantity drafts capture PC, service NPC/session, full item and typed source slot.
Revalidate before submit; authority changes invalidate them. Clamp to authoritative
maximum; zero and duplicate completed presses send nothing. Shared input registers
after DefaultPlugins and explicit Deferred startup; gameplay image owners stay in
NativeUiStartupSet, not login prerequisites. Higher modals/focus loss gate actions.

## Pointer, scroll and text

Arrows, held repeat, track paging and thumb drag use the same proportional thumb
extent as rendering. Track repeat keeps initial direction and stops at the pointer;
drag keeps the grab offset. Cancel on owner/tab/focus/modal change. Bank drag icon
passes focus and drop scale/fade is visual only, not local authority.
Equipment captions are MiddleRight intrinsic Text in fixed Rects; Taros is centered
inside the original nine cells. Bank title uses its information-panel origin and full
width. Black scroll-edge tint with preserved alpha/nine-slice is an owner-requested
native correction, not asserted original Unity tint. Button binders preserve normal,
hover, pressed and disabled states. Hold requested image handles through async loading.
At fractional scale compare rounded local offsets from unrounded sizes, then transform;
do not patch layout based on subtraction of already-rounded widths.

## Help, redeem and portraits

Game Guide owns help. Vendor, mentor and WorldMap use FirstUse→Help routes; a valid
new topic resets scrolling, invalid routes retain selection. Hide unavailable buttons
and hit areas, not just labels. Bank FirstUse 3 is Gear/Your Stuff, not invented Bank
help. Shared redeem drafts are tied to PC/NPC/mode and use the existing free-chat
redeem path; active Enchant is not required.
Service portraits lease layer bits on admitted live NPCs, wait for streamed meshes,
and release on target/mode loss. Preserve unrelated layers and re-check after rig
reparenting. Combi waiting uses its own wide framing.

For server ownership see [Vendor](vendor.md), [Bank](bank.md) and
[service modes](service-modes.md). Native preview and offline full-client fixtures
are not live-shard transaction or running-Unity parity acceptance.
