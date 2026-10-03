# FreeChat and menus

The accepted August primary always shows ALL/GROUP/BUDDY, including solo play.
Default 440×150, clamp 325×135..550×250, bottom-left anchored; top-right resize grip
is 25×25. For the default size preserve:

| Element | Rect x,y,w,h |
|---|---|
| Background / log | 0,14,440,113 / 24,18,416,97 |
| Entry / MENU / EMOTE | -8,117,462,40 / 6,124,31,19 / 41,124,31,19 |
| Input / SEND / grip | 75,124,295,23 / 365,124,69,23 / 411,18,25,25 |
| Scrollbar | 3,18,16,97 |

Input intentionally overpaints SEND by five pixels. Tabs are 135×14 at x=-40/95/230
(ALL/BUDDY/GROUP); below width 390, Buddy/Group x=71/182. Active controls include
MENU, EMOTE, field, SEND and grip; inactive retains MENU and the ten-pixel-left hint.
Active padding is 5 on every side. Empty Back padding=(50,5,7,0); cyan
(0.8,1,1,1) Rect=(27.5,33.5,385,60). Buddy/Group icons are 29 pixels.

Scrollbar arrows are 12 high; thumb width 15 and height ≥15. Wheel and thumb change
actual history scroll. Cancel capture on release, focus loss, channel/owner changes,
hide or disable. MENU starts x=40, EMOTE x=72; rows are 18 high, bottom padding 14.
Emote root has 19 entries Hello..Extras; next arrow uses 344. Use reachable normal/
hover states, not unused `*_sel` textures. Active Buddy pane is 293×150, x=current
chat width, bottom aligned; inactive has none.

History is decoded/server-owned, not optimistic echo. NPC semantic speech is emitted
before balloon gating; drain after NpcSpeech and apply the NPC-message option.
Mandatory NPC quest lines replace the speaker's current and queued bubbles immediately.
Their overhead bubbles use the yellow quick-chat box and tail; ordinary NPC speech keeps
the green barker box and tail. The line kind must survive queueing and bubble reuse.
Autonomous background lines share a 600-second cooldown by whitespace-normalized source
text across NPCs and TableData rows, independent of EN/RU selection. Suppression applies
before both history and balloon publication; greetings, quest and combat lines bypass it.
ALL keeps 50 lines; received translation is retained. Buddy history routing is in
[social](social.md). Enter and SEND retain distinct menu-close behavior and the
UTF-16 limit. See [text editing](text-editing.md) for caret/selection; no separate
ASCII-only editor or font shrinking for long input.

## Receive ownership

Normal/ALL GROUP use their exact 0104 packet families. ALL and GROUP each retain
50 rows: Local uses [Local]; GROUP stores unprefixed and mirrors [Group] into ALL.
Resolve group senders through the current roster; blank names fall back to
`Player <pc_uid>`. Only local special-state bit64 gates local FreeChat. Without a
multi-PC roster, selecting/sending GROUP returns routing to ALL but leaves all tabs
visible. Unread uses the 14×14 alert route off the selected tab and clears on selection.

Only accepted Local success creates a player bubble: replace that player's prior
bubble, project from the validated 1.6-unit controller at height×0.9 and expire after
max(7 seconds, rendered height/5). Use BubbleChatSkin fbBox/fbLabel, not NPC green
Barker art. Buddy/Group never create these bubbles. Retain message-type palettes:
General/Buddy/Group use persisted cnTextOption, NPC white, Receive red/yellow midpoint,
Damage/system/default red, Attack blue.
