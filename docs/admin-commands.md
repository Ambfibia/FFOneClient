# Administrator chat commands

Enter commands in world chat with the leading `/`. Commands require an
account level of 50 or below; moderation, announcements, teleports targeting
players, NPC summons, GM flags and reward rates require 30 or below. The server decides whether to accept the request
and owns the resulting values, inventory and position.

| Command | Effect |
| --- | --- |
| `/taro <value>` or `/taros <value>` | Set Taros |
| `/fm <value>` or `/fusionmatter <value>` | Set Fusion Matter |
| `/health <value>` or `/hp <value>` | Set health |
| `/batteryW <value>` | Set weapon battery |
| `/batteryN <value>` | Set Nano battery |
| `/speed <value>` | Set movement speed |
| `/jump <value>` | Set jump attribute |
| `/warp <tile-x> <tile-y>` | Teleport to the centre of a map tile |
| `/goto <x> <y> [z]` | Teleport to world coordinates; default altitude 100 |
| `/item <type> <id> <count> [time-left]` | Give an item in a free inventory slot; `/itemN` is equivalent |
| `/itemQ <id> <count>` | Give a quest item |
| `/nano <nano-id>` | Request a Nano |

For example, `/taro 10000` requests a balance of 10000, `/fm 5000` requests
5000 Fusion Matter, and `/warp 1 -2` requests the centre of tile (1, -2).
The `/item` argument order is **type, ID, count**. Item IDs and types must exist
in the server's tables. The server may cap values or reject unsupported items.
“Command sent” confirms transmission, not server acceptance.

These commands use the normal world chat input independently of the selected
All/Group/Buddy tab. Unknown slash commands continue to the server's chat-command
handler. Syntax and access errors are localized in English and Russian.

RustyFusion handles these forwarded commands itself (accountLevel 50 or below).
Its replies arrive as server system messages in English:

| Command | Effect |
| --- | --- |
| `/level <1-36>` or `/levelx <1-36>` | Set your character level; saved with the character. Grants no Nanos |
| `/whois` | Describe the nearest NPC in view: ID, type, name, HP, position, angle, chunk, map, instance, distance |

## Additional commands

Names are case-sensitive. All 77 names from the privileged primary chat switch
now have a native dispatch path, including the following groups. Native aliases
such as `/taro`, `/fm`, `/item` and `/fly` are additional to that count.

| Command | Effect / arguments |
| --- | --- |
| `/nano_equip <nano-id> <slot>` | Equip into slot 0..2 |
| `/nano_unequip <slot>` | Unequip slot 0..2 |
| `/nano_active <slot>` | Activate slot 0..2; -1 dismisses |
| `/nanoskill <nano-id> <skill-id>` | GM skill assignment, server-validated |
| `/nanoArr <level>` | Raise to a greater level, up to 36, grant IDs from old level + 1 through target and assign their first declared skills |
| `/summon <npc-type> [count]` | Spawn 1..100 NPCs |
| `/groupsummon <group-type>` | Spawn the declared five-member NPC group |
| `/summonshiny <type>` | Spawn a temporary shiny, offset 200..599 server units on each horizontal axis |
| `/unsummon` | Remove the selected living NPC |
| `/equipitem <inventory-slot>` | Equip the item from inventory |
| `/mission <mission-id>` | Mark the mission complete and stop its active tasks; no inferred rewards |
| `/task <active-task-id>` | Complete the active task through the normal reward/continuation path |
| `/warptopc <pc-id>` | Warp to a connected player |
| `/unstick` or `/unstick_i`, `_ui`, `_n` | Unstick self or selected player identity |
| `/locate_i`, `/locate_ui`, `/locate_n` | Show PC ID, UID, map and server coordinates |
| `/teleport2me_i`, `_ui`, `_n` | Bring a target to the GM |
| `/teleportXYZ_i`, `_ui` | `<target> <x> <y> <z>` in world units |
| `/teleportXYZ_n` | `<x> <y> <z> <FirstName> <LastName>` |
| `/teleportMapXYZ_i`, `_ui` | `<target> <map> <x> <y> <z>` |
| `/teleportMapXYZ_n` | `<map> <x> <y> <z> <FirstName> <LastName>` |
| `/teleport_i_i`, `/teleport_ui_ui` | `<target> <destination-player>` |
| `/teleport_i_n` | `<target-PC-ID> <destination-FirstName> <destination-LastName>` |
| `/teleport_n_n` | `<First Last>; <Destination First Last>` |
| `/kick_i`, `/kick_ui`, `/kick_n` | Disconnect the target |
| `/mute_i_on`, `/mute_ui_on`, `/mute_n_on` | Mute the target; matching `_off` commands unmute |
| `/invisible`, `/invulnerable`, `/gmmarker` | Toggle the corresponding server flag |
| `/announce`, `/bcast` | `<area\|shard\|world\|global> <type> <seconds> -message` |
| `/motd <type> -message` | Set the login message until server restart |
| `/rateT`, `/rateF` | Query rates, or set `<index:0..4> <percent:0..1000>`; index 0 sets all |
| `/rule <page-index>` | Open the existing help/rule UI |
| `/viewloc` (alias `/viweloc`), `/viewid`, `/viewnetinfo` | Toggle native diagnostics; `/viewloc` shows map, server XYZ and player `iAngle` in degrees ([NPC placement guide](npc-placement.md)); `/viewnetinfo history` toggles bounded packet history |
| `/viewcol` | Toggle nearby collision-bound outlines |
| `/hideui` | Toggle HUD visibility; enter again to restore |
| `/qinven`, `/tasklog` | Print quest inventory / active task IDs to local chat |
| `/chnum`, `/chinfo` | Show channel number / channel population |
| `/chwarp <number>`, `/shwarp <number>`, `/shardwarp <number>` | Validate against the paired server's single-channel/single-shard topology |
| `/Store` | Open a GM street stall |
| `/Store <PC-ID>` | Browse another nearby GM's open stall |

Identity suffix `_i` uses a positive PC ID, `_ui` a positive 64-bit UID, and `_n`
first/last names (the last name may contain two words). For example,
`/locate_n Test Player`, `/teleportXYZ_i 81 -100 200 30`,
`/announce shard 1 10 -Server maintenance in ten minutes`.

## Paired server and transaction behavior

The paired OpenFusion changes are required: 15 handlers were added to the base
116-registration inventory (eight GM requests plus seven street-stall requests).
An older running server will not acquire them until it is rebuilt/restarted.

Item requests are queued, with only one in flight. Slot selection happens after
the preceding authoritative reply is committed. The server never overwrites an
occupied slot, returns the actual quest slot, and sends failures for bad IDs,
full inventories, denied access and count overflow. A timeout retains the lock
until a late reply or reconnect; the client does not retry uncertain grants.

The GM store is an explicit server extension: opening requires no consumable;
there are five listing slots and a five-percent sales tax matching the existing
UI's truncation rule. Both entry and purchase require GM access. Offers retain
their source items in persistent inventory until purchase. The shard validates
the complete original item, available money, empty buyer slot, player distance,
inventory capacity and absence of an active trade before a single transaction.
Disconnect discards offers, never items. Ordinary consumable/player-menu store
entry remains independently gated; `/Store <PC-ID>` is the admin entry route.

OpenFusion has one shard/channel here. Number 1 reports that the player is
already there; other numbers fail. The two shard aliases use that native server
contract, not the primary client's multi-shard reconnect sequence. Diagnostics
and announcements use a native overlay; `/viewcol` shows bounds, not a claim of
pixel-identical Unity collider rendering. `/nanoArr` explicitly sets level because
this server correctly decouples individual Nano IDs from character level.

Validation includes protocol layout tests, all-command parser/access fixtures,
item-queue regression, native store reducer/localization tests, isolated real
server-handler socket tests (including a two-player store transaction), and
EN/RU GPU store captures. No live account was used for these tests.
