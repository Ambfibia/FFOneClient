# Bank

Root=1036×653, shell=495×638. Vault is 6×34 with 200 slots; unless `iExtraBank == 1`,
60..199 remain visible but locked. Open and both move post-values are authoritative
and applied atomically. Localized search does not renumber server slots.

Right-click transfers to first empty opposite slot. Left drag captures full item and
owner until release to an explicit bank/inventory slot; cancel on focus/modal/authority
change and while replies are pending. Same-cell release opens inspection: equipment/
general large panel, chests short panel. Transfer preserves the full stack. Changed
PC/NPC/slot/item, closed service or pending transaction invalidates the draft. Escape
closes the popup but preserves the Bank input gate through the current frame.

Inventory-side trash uses an icon/stack confirmation and UserEquipProductionRuntime.
Revalidate live inventory; cancellation/stale replies do nothing. Lock until correlated
reply or timeout, retaining bounded late-reply handling without deleting locally.
Vault cards are transfer-only. Drag feedback never commits item state.

Help opens FirstUse 3 (Gear); capacity refusals use shared localized OK. Redeem uses
the shared PC/NPC/mode owner and existing free-chat path. Common card, scrollbar,
text/image and pointer rules are in [item cards](item-cards.md).

Optional `FFONE_BANK_POINTER` cases: help, redeem, popup-general, popup-equip,
popup-chest, popup-inventory, popup-transfer, popup-delete, popup-delete-accept,
popup-delete-cancel. Their native GPU fixtures do not establish a live round trip.
