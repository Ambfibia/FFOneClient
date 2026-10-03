# Protocol 0104 character creation

FFOneClient follows the native login-server sequence implemented by OpenFusion:

1. `P_CL2LS_REQ_CHECK_CHAR_NAME` (`0x12000002`, 64 bytes)
2. `P_LS2CL_REP_CHECK_CHAR_NAME_SUCC/FAIL`
3. `P_CL2LS_REQ_SAVE_CHAR_NAME` (`0x12000003`, 68 bytes)
4. `P_LS2CL_REP_SAVE_CHAR_NAME_SUCC` (64 bytes), which assigns the account-owned PC UID
5. `P_CL2LS_REQ_CHAR_CREATE` (`0x12000004`, 100 bytes)
6. `P_LS2CL_REP_CHAR_CREATE_SUCC/FAIL`

All packets remain on the existing login TCP connection and use its post-login E key and
continuous legacy client sequence counter. Heartbeats may occur between any two responses and are
answered before the operation continues.

OpenFusion-specific behavior that the client handles explicitly:

- `CHECK_CHAR_NAME` currently echoes success. Authoritative availability/content validation occurs
  in `SAVE_CHAR_NAME`.
- A rejected save can arrive as either `CHECK_CHAR_NAME_FAIL`, `SAVE_CHAR_NAME_FAIL`, or
  `SHARD_SELECT_FAIL` through OpenFusion's shared `invalidCharacter()` path.
- Invalid create/delete requests can likewise arrive as their declared failure packet or
  `SHARD_SELECT_FAIL`.
- Creating appearance is allowed only after a matching successful name save. A reconnect may
  resume an account roster entry whose `appearance_flag` is zero.
- OpenFusion validates gender-dependent face/hair ranges and requires the upper-body, lower-body,
  and foot starter IDs to be valid level-one items of types 1, 2, and 3.

Deletion is `P_CL2LS_REQ_CHAR_DELETE` (`0x12000006`) with one 64-bit PC UID. Protocol 0104 has no
password, PIN, or authorization-code field for this operation. OpenFusion authorizes the UID
against the account bound to the current authenticated login socket.

There is no protocol-0104 request that asks the login server to resend the character list.
`RefreshCharacters` therefore republishes the session's typed local roster. Create/delete
responses update that roster first. A create response does not carry world coordinates; an
existing unfinished entry keeps its coordinates, while a newly created local selection entry uses
zero coordinates until shard entry or the next login supplies authoritative coordinates.
The network worker keeps the authenticated `LoginSession` while gameplay runs, services
its heartbeats, and updates the local roster position from each successful shard-entry
snapshot. Returning to selection therefore retains creation/deletion capability and the
new character's known location without reconnecting or resetting the packet sequence.
