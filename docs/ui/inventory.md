# UserEquip / inventory

Root=1020×638; one-second sin(πt/2) opening. Inventory is 5×10; equipment wire→visual
order is [4,5,6,1,2,3,0,7,8]. Taros occupies nine digits. Combined high-word appearance
is not the base wire identity; retain Quest's early-null rule and declared icon routes.
Move/use/delete are typed server requests, with a ten-second timeout and correlated
post-state. Preserve protocol reply-label reversal; do not “correct” wire semantics.

Drag supports reorder/equip/first-empty unequip/two weapons/trash confirmation. No
optimistic inventory updates. Clear preview models outside World. Avatar preview is
independent logical 500×564 at local (0,-50), clipped to the left; mounted Y offset +100.
Its render target follows the window/UI pixel density and uses linear sampling, so
Scale UI and high-DPI displays do not magnify a fixed-resolution avatar image.
Turn controls are 43×78 at (120,440)/(340,440).

Controller focus uses a cyan border and translucent fill above slot icons. A opens
an occupied item without starting a mouse drag; B closes its card before the main
window. LB/RB select Items/Nanos. Item cards, Nano viewer and help capture focus
within their own popup until dismissed. Held A remains pressed for continuous
controls, while a release and a new A produce a fresh activation.

Nano gallery identity is stored `Nano.id`, not array position. Gallery preview and
station/current skill have different owners. Resolve semantic nanoicon/nanoready
routes; accepted IDs/donors are canonical in FusionForge's source contract, never
re-infer the current Unstable/Van Kleiss mapping from older snapshots. Five columns,
13 rows; Nano scroll 397 versus item scroll 190; source sensitivity=0.1×200.
Nano icons use nearest filtering. Cheese tunes 211..213 retain the documented fallback
when the journal lacks the extension. The viewer is full-color even for unowned Nanos;
locked cells use the ready art.

General subtype 3 Nano giveaway uses `m_iStimPackAttri == 4 || style + 1` matching.
Subtypes 4/5/7/8/9/11 use compact user dialog; other gum uses the gum calculation,
chests the user dialog. Preserve occupied strip (0.6,1,0,0.8), empty (1,1,1,0.8)
and localized uppercase EQUIPPED. The footer redeem control opens the shared
character-owned input owner, not a new transport. Item details are [item cards](item-cards.md).
