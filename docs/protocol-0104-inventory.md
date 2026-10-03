# OpenFusion protocol 0104 registered packet inventory

This inventory is pinned to OpenFusion commit `7c26741acda5687f71092646af4ea4271940b80d`.
Its CMake default is `PROTOCOL_VERSION=104`. IDs come from
`OpenFusion/src/core/Defines.hpp`; shard membership comes only from real
`REGISTER_SHARD_PACKET(...)` calls. Login membership comes only from the
`CNLoginServer::onStep()` switch. Merely having a descriptor in `Packets.cpp`
does not make a client request handled.

The snapshot contains **10 login cases** and **116 shard registrations**.
All 116 shard names are unique and every one resolves to an ID.

## Login server cases

| ID | Request |
| --- | --- |
| `0x12000001` | `P_CL2LS_REQ_LOGIN` |
| `0x1200000c` | `P_CL2LS_REP_LIVE_CHECK` |
| `0x12000002` | `P_CL2LS_REQ_CHECK_CHAR_NAME` |
| `0x12000003` | `P_CL2LS_REQ_SAVE_CHAR_NAME` |
| `0x12000004` | `P_CL2LS_REQ_CHAR_CREATE` |
| `0x12000006` | `P_CL2LS_REQ_CHAR_DELETE` |
| `0x12000005` | `P_CL2LS_REQ_CHAR_SELECT` |
| `0x1200000a` | `P_CL2LS_REQ_SAVE_CHAR_TUTOR` |
| `0x1200000d` | `P_CL2LS_REQ_CHANGE_CHAR_NAME` |
| `0x1200000b` | `P_CL2LS_REQ_PC_EXIT_DUPLICATE` |

OpenFusion explicitly leaves `P_CL2LS_REQ_SHARD_SELECT`,
`P_CL2LS_CHECK_NAME_LIST`, `P_CL2LS_REQ_SERVER_SELECT`, and
`P_CL2LS_REQ_SHARD_LIST_INFO` unimplemented; they are not counted above.

## Native implementation status

The client-side protocol boundary covers the complete request surface implemented by this pinned
server snapshot:

- all 10 login switch cases have exact protocol-0104 codecs or an internal heartbeat route;
- all 116 registered shard requests are represented by a checked registry entry and an exact
  fixed or counted body layout;
- the generic shard command accepts only a `RegisteredGameplayRequest0104`, so an unknown packet
  ID, an oversized body, a wrong fixed size, a negative count, a count above the server cap, or a
  count/trailer mismatch cannot reach the socket;
- feature-owned typed codecs cover Email, PC-to-PC trade, Racing, Transportation, and mission
  start/end requests without weakening that registry gate;
- fixed-layout login and shard replies are checked at the reader boundary. A size mismatch becomes
  a lossless `MalformedFrame0104` diagnostic and is never exposed as a normal state-mutating frame;
- server-side slash commands use the ordinary FreeChat request path. OpenFusion command output is
  decoded from the exact pack(2), 1026-byte `P_FE2CL_PC_MOTD_LOGIN` payload and follows the clean
  client's `CnGuiChat.ReceiveMOTD` route into the system-colored ALL and GROUP histories. The
  clean-client-owned `/speed <value>` command is intercepted separately and uses the exact
  `P_CL2FE_GM_REQ_PC_SET_VALUE`/`P_FE2CL_GM_REP_PC_SET_VALUE` type-6 exchange;
- unknown and variable-layout server frames remain lossless. Implemented gameplay owners use their
  stricter family decoder before applying authoritative state.

This is wire/transport coverage, not a claim that every possible server reply owns a visible UI
reducer. Presentation modules consume only the reply families whose native gameplay owner exists;
the remaining frames stay available losslessly instead of being guessed or discarded.

OpenFusion does not register the legacy street-stall/UserStore request family in this snapshot.
Those requests therefore cannot be made functional by client transport code and are intentionally
rejected by the registered-request boundary rather than sent to an absent server handler.

## Clean-client wire mirror

`ffone_protocol::wire_0104` is generated from the decompiled primary `Assembly-CSharp` by
`../FusionForge/tools/legacy-sources/generate-wire-0104.py`. It carries every one of the 515
packet structs and 57 nested records with the exact Mono marshaled layout, every `csDefines` ID
(upper-cased into `packet::`), `declared_struct_size`, and `frame_kind`, which classifies frames
with OpenFusion's own `core/Packets.cpp` descriptor table: 485 fixed, 15 counted, 6 manual.
`fixed_payload_size` falls back to that table only for client-to-server IDs without a hand-written
codec; server-to-client frames without a hand-written codec stay unclassified, because the pinned
server sends several `eST`-dependent packets through its unvalidated legacy path while its table
still calls them fixed.

The generator also records where the clean client and the pinned server disagree
(`OPENFUSION_SIZE_DIVERGENCES_0104`; ledger in
`../FusionForge/docs/reference/evidence/legacy/ffone/managed-code/wire-0104-openfusion-divergence.json`).
The retrobution-20260821 client's `sPCLoadData2CL` is 2552 bytes (`iUnlockedFeatureFlag`, no
`iFatigue*`/`aiPCSkill`) against the server's 2688; the hand-written codecs follow the pinned
server for every divergent packet and `wire_0104::tests::hand_written_sizes_agree_with_mirror`
rejects any third variant.

## Shard server registrations

| Source family | Count |
| --- | ---: |
| `Buddies.cpp` | 9 |
| `BuiltinCommands.cpp` | 11 |
| `Chat.cpp` | 9 |
| `Combat.cpp` | 8 |
| `Eggs.cpp` | 1 |
| `Email.cpp` | 8 |
| `Groups.cpp` | 4 |
| `Items.cpp` | 5 |
| `Missions.cpp` | 4 |
| `Nanos.cpp` | 9 |
| `NPCManager.cpp` | 6 |
| `PlayerManager.cpp` | 10 |
| `PlayerMovement.cpp` | 9 |
| `Racing.cpp` | 4 |
| `Trading.cpp` | 10 |
| `Transport.cpp` | 2 |
| `Vendors.cpp` | 7 |
| **Total** | **116** |

### Buddies.cpp (9)

| ID | Request |
| --- | --- |
| `0x13000036` | `P_CL2FE_REQ_ACCEPT_MAKE_BUDDY` |
| `0x1300003c` | `P_CL2FE_REQ_GET_BUDDY_STATE` |
| `0x13000051` | `P_CL2FE_REQ_PC_BUDDY_WARP` |
| `0x1300008f` | `P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY` |
| `0x1300008e` | `P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY` |
| `0x1300003b` | `P_CL2FE_REQ_REMOVE_BUDDY` |
| `0x13000035` | `P_CL2FE_REQ_REQUEST_MAKE_BUDDY` |
| `0x1300003a` | `P_CL2FE_REQ_SET_BUDDY_BLOCK` |
| `0x13000070` | `P_CL2FE_REQ_SET_PC_BLOCK` |

### BuiltinCommands.cpp (11)

| ID | Request |
| --- | --- |
| `0x1300006c` | `P_CL2FE_GM_REQ_KICK_PLAYER` |
| `0x1300006e` | `P_CL2FE_GM_REQ_PC_LOCATION` |
| `0x1300006b` | `P_CL2FE_GM_REQ_PC_SET_VALUE` |
| `0x1300006a` | `P_CL2FE_GM_REQ_PC_SPECIAL_STATE_SWITCH` |
| `0x130000a3` | `P_CL2FE_GM_REQ_REWARD_RATE` |
| `0x13000082` | `P_CL2FE_GM_REQ_TARGET_PC_SPECIAL_STATE_ONOFF` |
| `0x1300006d` | `P_CL2FE_GM_REQ_TARGET_PC_TELEPORT` |
| `0x1300001a` | `P_CL2FE_REQ_PC_GIVE_ITEM` |
| `0x13000044` | `P_CL2FE_REQ_PC_GIVE_NANO` |
| `0x13000014` | `P_CL2FE_REQ_PC_GOTO` |
| `0x13000057` | `P_CL2FE_REQ_PC_WARP_TO_PC` |

### Chat.cpp (9)

| ID | Request |
| --- | --- |
| `0x1300006f` | `P_CL2FE_GM_REQ_PC_ANNOUNCE` |
| `0x13000050` | `P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT` |
| `0x1300002d` | `P_CL2FE_REQ_PC_TRADE_EMOTES_CHAT` |
| `0x13000063` | `P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE` |
| `0x13000066` | `P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE` |
| `0x13000037` | `P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE` |
| `0x13000038` | `P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE` |
| `0x13000007` | `P_CL2FE_REQ_SEND_FREECHAT_MESSAGE` |
| `0x13000008` | `P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE` |

### Combat.cpp (8)

| ID | Request |
| --- | --- |
| `0x13000049` | `P_CL2FE_DOT_DAMAGE_ONOFF` |
| `0x13000090` | `P_CL2FE_REQ_PC_ATTACK_CHARs` |
| `0x13000006` | `P_CL2FE_REQ_PC_ATTACK_NPCs` |
| `0x13000033` | `P_CL2FE_REQ_PC_COMBAT_BEGIN` |
| `0x13000034` | `P_CL2FE_REQ_PC_COMBAT_END` |
| `0x1300001f` | `P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE` |
| `0x1300001c` | `P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE` |
| `0x1300001d` | `P_CL2FE_REQ_PC_ROCKET_STYLE_HIT` |

### Eggs.cpp (1)

| ID | Request |
| --- | --- |
| `0x13000060` | `P_CL2FE_REQ_SHINY_PICKUP` |

### Email.cpp (8)

| ID | Request |
| --- | --- |
| `0x1300007e` | `P_CL2FE_REQ_PC_DELETE_EMAIL` |
| `0x1300007b` | `P_CL2FE_REQ_PC_EMAIL_UPDATE_CHECK` |
| `0x1300007c` | `P_CL2FE_REQ_PC_READ_EMAIL` |
| `0x13000081` | `P_CL2FE_REQ_PC_RECV_EMAIL_CANDY` |
| `0x13000080` | `P_CL2FE_REQ_PC_RECV_EMAIL_ITEM` |
| `0x13000089` | `P_CL2FE_REQ_PC_RECV_EMAIL_ITEM_ALL` |
| `0x1300007d` | `P_CL2FE_REQ_PC_RECV_EMAIL_PAGE_LIST` |
| `0x1300007f` | `P_CL2FE_REQ_PC_SEND_EMAIL` |

### Groups.cpp (4)

| ID | Request |
| --- | --- |
| `0x1300004c` | `P_CL2FE_REQ_PC_GROUP_INVITE` |
| `0x1300004d` | `P_CL2FE_REQ_PC_GROUP_INVITE_REFUSE` |
| `0x1300004e` | `P_CL2FE_REQ_PC_GROUP_JOIN` |
| `0x1300004f` | `P_CL2FE_REQ_PC_GROUP_LEAVE` |

### Items.cpp (5)

| ID | Request |
| --- | --- |
| `0x13000047` | `P_CL2FE_REQ_ITEM_CHEST_OPEN` |
| `0x1300000a` | `P_CL2FE_REQ_ITEM_MOVE` |
| `0x13000073` | `P_CL2FE_REQ_ITEM_USE` |
| `0x1300002e` | `P_CL2FE_REQ_PC_BANK_OPEN` |
| `0x13000019` | `P_CL2FE_REQ_PC_ITEM_DELETE` |

### Missions.cpp (4)

| ID | Request |
| --- | --- |
| `0x13000083` | `P_CL2FE_REQ_PC_SET_CURRENT_MISSION_ID` |
| `0x1300000c` | `P_CL2FE_REQ_PC_TASK_END` |
| `0x1300000b` | `P_CL2FE_REQ_PC_TASK_START` |
| `0x13000012` | `P_CL2FE_REQ_PC_TASK_STOP` |

### Nanos.cpp (9)

| ID | Request |
| --- | --- |
| `0x13000015` | `P_CL2FE_REQ_CHARGE_NANO_STAMINA` |
| `0x1300000f` | `P_CL2FE_REQ_NANO_ACTIVE` |
| `0x1300000d` | `P_CL2FE_REQ_NANO_EQUIP` |
| `0x13000011` | `P_CL2FE_REQ_NANO_SKILL_USE` |
| `0x13000010` | `P_CL2FE_REQ_NANO_TUNE` |
| `0x1300000e` | `P_CL2FE_REQ_NANO_UNEQUIP` |
| `0x13000048` | `P_CL2FE_REQ_PC_GIVE_NANO_SKILL` |
| `0x13000071` | `P_CL2FE_REQ_REGIST_RXCOM` |
| `0x13000074` | `P_CL2FE_REQ_WARP_USE_RECALL` |

### NPCManager.cpp (6)

| ID | Request |
| --- | --- |
| `0x13000065` | `P_CL2FE_REQ_BARKER` |
| `0x13000045` | `P_CL2FE_REQ_NPC_SUMMON` |
| `0x13000046` | `P_CL2FE_REQ_NPC_UNSUMMON` |
| `0x13000088` | `P_CL2FE_REQ_PC_TIME_TO_GO_WARP` |
| `0x1300004b` | `P_CL2FE_REQ_PC_WARP_USE_NPC` |
| `0x130000a7` | `P_CL2FE_REQ_PRESENT_NPC_TYPES` |

### PlayerManager.cpp (10)

| ID | Request |
| --- | --- |
| `0x13000075` | `P_CL2FE_REP_LIVE_CHECK` |
| `0x13000054` | `P_CL2FE_REQ_PC_CHANGE_MENTOR` |
| `0x13000001` | `P_CL2FE_REQ_PC_ENTER` |
| `0x13000002` | `P_CL2FE_REQ_PC_EXIT` |
| `0x13000086` | `P_CL2FE_REQ_PC_FIRST_USE_FLAG_SET` |
| `0x1300008d` | `P_CL2FE_REQ_PC_LOADING_COMPLETE` |
| `0x13000009` | `P_CL2FE_REQ_PC_REGEN` |
| `0x1300007a` | `P_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH` |
| `0x130000a0` | `P_CL2FE_REQ_PC_VEHICLE_OFF` |
| `0x1300009f` | `P_CL2FE_REQ_PC_VEHICLE_ON` |

### PlayerMovement.cpp (9)

| ID | Request |
| --- | --- |
| `0x13000005` | `P_CL2FE_REQ_PC_JUMP` |
| `0x1300003d` | `P_CL2FE_REQ_PC_JUMPPAD` |
| `0x1300003e` | `P_CL2FE_REQ_PC_LAUNCHER` |
| `0x13000003` | `P_CL2FE_REQ_PC_MOVE` |
| `0x13000040` | `P_CL2FE_REQ_PC_MOVEPLATFORM` |
| `0x13000062` | `P_CL2FE_REQ_PC_MOVETRANSPORTATION` |
| `0x13000041` | `P_CL2FE_REQ_PC_SLOPE` |
| `0x13000004` | `P_CL2FE_REQ_PC_STOP` |
| `0x1300003f` | `P_CL2FE_REQ_PC_ZIPLINE` |

### Racing.cpp (4)

| ID | Request |
| --- | --- |
| `0x1300005e` | `P_CL2FE_REQ_EP_GET_RING` |
| `0x1300005d` | `P_CL2FE_REQ_EP_RACE_CANCEL` |
| `0x1300005c` | `P_CL2FE_REQ_EP_RACE_END` |
| `0x1300005b` | `P_CL2FE_REQ_EP_RACE_START` |

### Trading.cpp (10)

| ID | Request |
| --- | --- |
| `0x1300002c` | `P_CL2FE_REQ_PC_TRADE_CASH_REGISTER` |
| `0x13000027` | `P_CL2FE_REQ_PC_TRADE_CONFIRM` |
| `0x13000028` | `P_CL2FE_REQ_PC_TRADE_CONFIRM_CANCEL` |
| `0x1300002a` | `P_CL2FE_REQ_PC_TRADE_ITEM_REGISTER` |
| `0x1300002b` | `P_CL2FE_REQ_PC_TRADE_ITEM_UNREGISTER` |
| `0x13000022` | `P_CL2FE_REQ_PC_TRADE_OFFER` |
| `0x13000026` | `P_CL2FE_REQ_PC_TRADE_OFFER_ABORT` |
| `0x13000024` | `P_CL2FE_REQ_PC_TRADE_OFFER_ACCEPT` |
| `0x13000023` | `P_CL2FE_REQ_PC_TRADE_OFFER_CANCEL` |
| `0x13000025` | `P_CL2FE_REQ_PC_TRADE_OFFER_REFUSAL` |

### Transport.cpp (2)

| ID | Request |
| --- | --- |
| `0x13000069` | `P_CL2FE_REQ_PC_WARP_USE_TRANSPORTATION` |
| `0x13000068` | `P_CL2FE_REQ_REGIST_TRANSPORTATION_LOCATION` |

### Vendors.cpp (7)

| ID | Request |
| --- | --- |
| `0x13000098` | `P_CL2FE_REQ_PC_ITEM_COMBINATION` |
| `0x1300004a` | `P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY` |
| `0x13000017` | `P_CL2FE_REQ_PC_VENDOR_ITEM_BUY` |
| `0x13000032` | `P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY` |
| `0x13000018` | `P_CL2FE_REQ_PC_VENDOR_ITEM_SELL` |
| `0x13000030` | `P_CL2FE_REQ_PC_VENDOR_START` |
| `0x13000031` | `P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE` |

## Native NPC lifecycle and basic combat slice

These are the packets implemented by `ffone-protocol` and exposed losslessly by
`ffone-net`. All fixed structures use little-endian protocol-0104 `#pragma pack(4)`
layouts. Counted trailers are length-checked exactly; no trailing bytes are ignored.

| Direction | ID | Packet | Exact payload bytes |
| --- | --- | --- | --- |
| client to shard | `0x13000006` | `P_CL2FE_REQ_PC_ATTACK_NPCs` | `4 + 4*n` (`iNPCCnt`, then IDs); OpenFusion accepts at most 3 targets |
| client to shard | `0x13000033` | `P_CL2FE_REQ_PC_COMBAT_BEGIN` | 4 (`iPC_ID`) |
| client to shard | `0x13000034` | `P_CL2FE_REQ_PC_COMBAT_END` | 4 (`iPC_ID`) |
| shard to client | `0x3100000b` | `P_FE2CL_NPC_ENTER` | 36 (`sNPCAppearanceData`) |
| shard to client | `0x3100000c` | `P_FE2CL_NPC_EXIT` | 4 (`iNPC_ID`) |
| shard to client | `0x3100000d` | `P_FE2CL_NPC_MOVE` | 24 (`id`, XYZ, speed, `int16` style, 2 padding bytes) |
| shard to client | `0x3100000e` | `P_FE2CL_NPC_NEW` | 36 (`sNPCAppearanceData`) |
| shard to client | `0x3100000f` | `P_FE2CL_NPC_AROUND` | OpenFusion effective `36 + 36*n`; count at offset 0, entries at offset 36 |
| shard to client | `0x31000011` | `P_FE2CL_AROUND_DEL_NPC` | `4 + 4*n` (count, then IDs) |
| shard to client | `0x31000014` | `P_FE2CL_PC_ATTACK_NPCs_SUCC` | `8 + 24*n` (battery, count, `sAttackResult[]`) |
| shard to client | `0x31000015` | `P_FE2CL_PC_ATTACK_NPCs` | `8 + 24*n` (PC ID, count, `sAttackResult[]`) |
| shard to client | `0x31000016` | `P_FE2CL_NPC_ATTACK_PCs` | `8 + 24*n` (NPC ID, count, `sAttackResult[]`) |

`sNPCAppearanceData` is nine `int32` fields (36 bytes). `sAttackResult` is five
`int32` fields followed by one `int8` hit flag and three padding bytes (24 bytes).
The AROUND offset of 36 is intentional OpenFusion behavior in `Chunking.cpp`, even
though the declared packet header itself is only four bytes.

The effective ILSpy handler advances multi-result combat entries by the eight-byte
packet-header size, but its own `sAttackResult` declaration is 24 bytes and OpenFusion's
descriptor/server emit 24-byte entries. The native implementation follows the confirmed
server wire ABI and records this discrepancy instead of reproducing a decompiler/client bug.

The complete outbound registry also validates Rocket, grenade, DOT, PvP `PC_ATTACK_CHARs`, NPC
skill, and other registered combat request bodies. Packet-specific gameplay reducers beyond the
table above remain outside this *basic NPC lifecycle slice*; their response layouts are not inferred
from the hitscan types, and their raw frames remain lossless until an owning gameplay system applies
a proven decoder.
