# Vendor

Main panel=498×638, left of center; rows=80 pixels, catalog≤50 and buyback FIFO≤15.
Preserve source signed-byte behavior: 9001→41 and negative GumPopup maximum; do not
silently normalize protocol values. Checked buy/sell/restore/delete intents are
server-authoritative; quantities and common controls are in [item cards](item-cards.md).

Start has a documented OpenFusion compatibility mapping: use table NPC identity in
both wire fields, while retaining placed NPC identity for UI/camera ownership. Mixing
them can admit Start then reject the table request, leaving close/escape pending.
Keep this profile distinct from primary Unity behavior; do not remove transaction or
pending-close guards to hide an ID error. Retain image handles while cold-loading icons.

Inventory chest OPEN captures PC/session/slot/full item and uses the shared UserEquip
chest owner. Lock until correlated completion/timeout; closing retires bounded late
replies. Neither request nor acknowledgement invents rewards.

## Try-on and portrait

TRY ON is catalog equipment 0..6 only: no inventory/buyback/consumables/vehicles.
Eligibility uses appearance gender/guide, including combined appearance, without a
new level gate. Clone the character summary; do not mutate inventory. Transparent
preview=210×375, camera distance 2.2/height 0.7; panel=215×375 shifts the card left
200. Held arrows rotate 100°/second. Closing preview retains the card. New card resets
permission/rotation; authority loss invalidates it. A pending generation cannot show
previous-character pixels.

Vendor portrait is a stable 200×150 live HNPC target: FOV70, near0.2/far2,
distance0.4, world height1.1, converted yaw20. Only admitted HNPC vendors show it;
other source-hidden portraits stay hidden. Release camera/image/layer leases on close
or target loss; repair leased layer bits after rig reparenting, not unrelated layers.

Focused replay: `FFONE_VENDOR_POPUP=buy|quantity|sell|buyback|chest|chest-open|try-on|try-on-close|localized-weapon|localized-general`.
Full-client `FFONE_PERF_VENDOR_REGRESSION=close|escape` exercises cold icons and actual
close-command result; run with advancing time, not `FFONE_PERF_FREEZE`. Offline
transport reset is not successful close. Vendor dragging remains a separately recorded
scope; inspect current code before assuming its historical status.
