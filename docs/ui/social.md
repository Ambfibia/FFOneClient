# Buddy, NanoCom and group UI

Buddy has 50 slots; names and PlayerPCUID are distinct. Preserve ten-second refresh,
two presence states, invitation policy, remove/block/warp confirmation and 60-second
warp cooldown. Only server success adds Buddy history: aggregate plus personal PCUID
history, mirrored as [Buddy] in ALL. Incoming messages mark unread only off the current
tab. Apply the exact block roster before history in every channel. SocialBuddy off
auto-declines invitations. Appearance names come from live identity, not guessed rows.
After successful same-shard warp, send zeroed group-leave only when the target is not
already a member; never clear group state optimistically.

NanoCom type 13 interactive requests outrank passive FIFO; preserve FIFO within each
class and correlate the ID/payload. Accept=1; decline/timeout=0. Direct requests and
lookup invitations differ. Compact root=372×122; frame=(51,3,321,119), icon=
(60,15,64,64), half-second sin² reveal. Expanded uses a 50% overlay and 520×164
dialog, message=(70,30,415,87), Decline=(44,124,150,25), Accept=(334,124,150,25).
Twenty-second lifetime expires when negative; a button emits now and removes next
tick. Timeout does not play No_Button. Resolve comm_slidein/out and yes/no through
semantic audio, not old shared-SFX paths.

Passive type 9 remains behind HUD/minimap. Body uses Chalet 11 / line 12.07199955,
Rect=(130,29,164,60), left compensation 13. Uppercase titles after localization.
EnterReadyWorld chat is one transition. Tutorial fallback yields to other popups.
Group panels consume group_info/freechat/nanohp/npc_co_op authority and do not own
player speech bubbles.

## Enter menu

Seven rows: first six blue, EXIT red; pitch=height+3, border=(8,8,5,5).
Font 14, glyph Y scale 0.70, line 13.71000004. Normal/active use the normal texture,
not hover; preserve serialized colors. Close background local=(153,42,23,23), with
no second parent-group offset.

Recorded unresolved scope includes BuddyMenuChat, other-shard warp, wider failure/
audio coverage and detached type-9 consolidation. Native fixtures did not establish
live two-client transactions or normalized Unity pixel parity.

The group snapshot specifically lacks invitations/accept-decline/leader-kick/outbound
leave/block controls/sounds/MenuChat integration. Verify code before scheduling or
claiming these; separate inbound roster/leave and GROUP history are already described.
