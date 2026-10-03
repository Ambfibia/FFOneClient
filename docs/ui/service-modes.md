# Smaller service modes

Read only the affected subsection. Shared dialogs, portrait leases, help and item
controls are defined in [item cards](item-cards.md), not reimplemented per mode.

## Pc2Pc trade

`pc2pc_ui.rs` has five local/five remote slots, Taros registration, shared Item panel,
Accept/Confirm, cancellation/failure and an embedded-chat boundary. Base inventory
stays immutable under an authoritative overlay. Only correlated register/unregister/
cash replies update it; final confirmation atomically commits server items, target
slots and Taros. Portraits stay transparent until cameras bind.
The supplied notes describe a passive plugin: screen gestures and production
transport/ownership remain fail-closed. OpenFusion0104 does not register clean trade
chat or outbound confirm-abort handlers. Do not advertise preview controls as working.

## Combi / Croc Pot / Enchant

Combi uses the strict 38-row recipe table, equipment refusal260, confirmation254/256,
wait>4seconds and packet sizes16/36/20. Preserve distinct primary/waiting live-NPC
portrait framing. Use real item names, descriptions, icons and result ratings; retain
image handles while loading, fixed Rect containers with intrinsic Text, and startup
asset-resource gates. Captured inventory drag dispatches once on release over an
attachment/trash control, respects opening/modals and cancels on focus reset. Existing
click selection and inventory validation remain intact. Seven published GLB action clips
per gender are required by the native catalog; do not disable validation to admit a rig.

Optional `FFONE_ENCHANT_POINTER=drag|close` and `FFONE_ENCHANT_ASSET_ROOT` exercise
preview/installed assets. Full-client `FFONE_PERF_ENCHANT=close` tests focus, drag,
Clear and Close under offline authority, not a live enchant transaction.

## Nano Free Tuning

Preserve zero44/request168/success8/failure protocol lengths and exact phase behavior,
including unreachable Sad. Use the native external camera owner, not a fabricated 2D
substitute. Earlier production-provider integration was partial; check current code.

## Race

32 locations, four tabs, five-item pagination and top ten; keep recovered inclusive
boundary behavior and two-step entrance. Help is ignored in the recovered path.
Do not declare missing production-provider integration complete from preview output.

## Rule

Only pages1 Vehicles and2 Combining. No Prev/Next and no invented third page.
Back/Close exit; Escape retains the source help/system-modal asymmetry.
