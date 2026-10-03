# Options

`crates/ffone-client/src/ui/option/mod.rs` types the exact four-tab Option shell,
all four reconstructed pages, five persisted option groups, 39 clean input
actions, key-capture/modal rules, sound boundaries and the original Apply
versus Save/Cancel asymmetry. The Graphics/Sound page preserves the exact
two-panel geometry, Browser/resolution dropdown, five detail presets,
visibility and particle track/thumb sliders, five advanced flags, the two-value
PlayerOnly/AllCharacters shadow enum, High/Medium/Low texture order, and
Master/SFX/Voice/Ambient/Music. Sound is the sole eager-commit exception.

Game UI renders all 13 reachable display rows, three exact 18-color palettes,
Animated NanoCom and the inverted `OLD CHAT -> new_chat` restart-modal
boundary. Controls renders mouse invert-Y/sensitivity and separate persisted gamepad
invert-Y/sensitivity 1..10, gamepad selection,
four scroll groups, the 27 clean reachable actions and three mapping columns
with source-sized scrollbars, arrow/wheel scrolling and eight-second
keyboard/mouse capture, duplicate rejection and defaults. Backspace or Delete
while capturing clears a binding so its key can be assigned elsewhere. Social retains the
complete request flags, bounded 50-slot blocked roster and the clean
always-enabled `UNIGNORE` surface.
The Social language section uses the newer 450x180 panel above the request
controls. Two independently applied selectors replace its single language
selector: text and voice, with explicit localized labels and existing native
Cyrillic-capable fonts. Request rows begin at y=280 with a 77px pitch; the
blocked list, footer and Controls geometry remain unchanged. Language choices
are rejected outside the visible Social page or their active dropdown.

The preview accepts `FFONE_OPTION_PREVIEW_LOCALE=en|ru` and
`FFONE_OPTION_PREVIEW_DROPDOWN=text|voice` for localized acceptance frames.
The native language change remains immediate
and persists through the existing independent locale settings.


Language panels use local z=70 above both z=60 selector buttons, so choices cannot be
occluded by the other field. 

Language labels and selectors share the row vertical center. Selected values
are centered within the text well, excluding the arrow button.

The two language values can differ (the RU capture uses Russian text and English
voice). RU request labels are shortened to fit the original control bounds.

The source contract covers 41 semantic PNGs, including the sliced clean
`set_back` outer frame, the serialized scrollbar/dropdown overflow and the
original paint order where normal tabs sit under the frame while the selected
tab sits above it. Every nonzero `FusionFallOptionSkin` border used by this
window is now an explicit nine-slice: frame `10/10/10/10`, button and close
`8/8/5/5`, Graphics normal/hover tab `0/0/0/3`, toggle `17/0/0/0`, dark panels
`10/10/10/10`, connector `6/35/0/0`, text field `2/2/2/2`, and scrollbar/thumb
`2/2/4/4`. The clean serialized `SkyTex` reference resolves to Texture2D
pathId 82, `equipbar`: a uniform 5x7 cyan source drawn with Stretch. The former
194x120 `setting_back_tmp` semantic route was incorrect and caused both the
pixel-smearing and the missing light title backing. Zero-border
`darkbox2`, `darkbox2_inter`, pulldown and horizontal-slider styles remain
Stretch. Text applies the serialized UpperLeft/UpperRight/MiddleLeft/MiddleRight
anchors through the actual flex container (including Start/End), plus the clean
button padding and Font-object line metrics. Style-specific replacement-font
sizes retain the clean title/button/whitelabel proportions, and the Graphics
tab uses a 10px native inset because Bevy cannot represent Unity's negative
left RectOffset without placing the glyphs outside its left rail. The three
runtime fonts are the validated native replacement files with Cyrillic coverage;
clean Unity Font objects remain metric evidence only and are deliberately not
runtime Option assets. Eight audio routes remain independent from the selected
text language. Production installs
the plugin, seeds it once from the real Window, orbit-camera and glow owners,
routes the clean configurable Key 4 and Key 23
bindings, and applies committed resolution/window mode, glow, UI scale,
waypoint, master volume, camera sensitivity/invert-Y and buddy-request policy.
Committed movement, jump and camera-turn mappings feed the native readers.
Controller camera speed is independent of the mouse setting (90 degrees/s yaw,
60 degrees/s pitch at pad sensitivity 5). New default pad bindings use separate
LT/RT, D-pad zoom/journal/weapon, LB free camera, RB Nano boost, face Nano slots,
L3 vehicle and R3 auto-run. Start goes through the Enter/NanoCom reducer; A/B
own confirmation/cancel in an open UI and A interacts with a usable world NPC
or trigger before jumping. Existing saved mappings are retained; Default Key
Mapping installs the new defaults. D-pad focus uses visible, unclipped,
pickable buttons in the foremost UI window and feeds their normal Interaction
handlers. Controller focus preserves a simultaneous pointer click. The
physical H key (Russian Р) is no longer a default Help binding or an active
Help route from older settings snapshots; KeyP remains the Email binding.
Game Guide still opens from NanoCom, and a separately assigned Help key can
open it with the normal modal cursor.

User-owned settings are durable at
`%LOCALAPPDATA%\FusionFallOne\settings.json`. The stable JSON document uses
schema `ffone.user-settings.v1`; every change replaces one atomic full
snapshot instead of writing independent per-control fragments. That snapshot
contains committed `options`, `input`, independent `text_locale` and
`voice_locale`, and `character_selection_music`. Login credentials, selected
server/session state, account or character data, and all other
server-authoritative gameplay state are deliberately excluded.

The production Scale UI adapter is window-owned rather than a second
`UiTransform` on each already-laid-out panel. It maps the physical client area
with the exact clean height formula (`1.05` at 768p and `1.4765625` at 1080p),
caps that scale to the measured `1264x681` reference surface on narrow or small
windows, and leaves extra aspect-ratio space available to the source anchors. Login, OptionMode,
character flow, HUD, clipping, pointer hit-testing and replacement-font layout
therefore consume the same logical coordinates. The clean per-mode scale
formulas remain available to isolated parity harnesses, but production passes
`1.0` to those post-layout matrices so maximizing an HD/FHD window cannot
double-scale or crop text fields.

Option also participates in the world-map, movement, cursor, chat, F1,
avatar-action and Quit modal gates and resets only its transient session state
when leaving World. 


The NanoCom SETTINGS entry, category-aware updates for already playing dynamic
audio sinks, gamepad capture and the server-side buddy-unblock route remain
explicit integration gates.
