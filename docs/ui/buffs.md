# Buffs and overheat

Icons are 26×26 with fixed debuff/buff order for local, cash and target owners.
Do not add hover/stack/ordinary expiry UI absent from the recovered contract. Cash
duration uses the shared server-reset tick, once per second without catch-up; preserve
the recovered hours/days division-by-60 behavior until an explicit correction is chosen.

Stim state drives local Nanos only; target presentation uses the recovered local-style
lookup. Declared row zero means empty, unlike an invalid unknown row. Clear cached
conditions on target despawn/ownership loss, including reused IDs. Friendly targets
require talk range; hostile targets show immediately. Remaining combat/equip/regeneration
coverage is a historical gap, not authorization to fabricate state.

Overheat remains owned by the weapon runtime and existing decoded state. Keep its
visual timer separate from Nano cooldown and server inventory authority.

Regular/cash/time-buff packets are typed; seed local flags from
`PcLoadData2CL.iConditionBitFlag`. Resolve `m_pSkillBuffData` → `m_pSkillIconData`
→ semantic `icons/skills/skillicon_##.png`, with TopLeft/TopRight/CenterTop pivots.
Preserve newer protected-infection/timeout state while off-target; NPC/player numeric
ID collisions must not leak into local HP.

`overheat_ui.rs` uses `ui/en/gameplay/overheat/{background,maximum,fill}.png`.
The primary initializes `localized.WpnOverheat=eNone`, so installation is hidden by
default; only explicit preview exposes it. The supplied snapshot has no live equipment/
table bridges. Do not enable it without authoritative eUse evidence.
