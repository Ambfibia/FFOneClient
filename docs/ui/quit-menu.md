# Quit menu

`quit_menu_ui.rs` owns presentation/input and `quit_menu_runtime.rs` destination/reply state.
The editable `assets/game/ui/en/gameplay/quit-menu/menu.ffquit.json` uses
`ffone-ui-layout::quit_menu::QuitMenuDocument`; its AssetLoader validates version, paths, metrics,
unique typed actions and styles before binding the production hierarchy. Missing/invalid documents
hide the menu and report an asset error. Runtime needs neither FusionForge nor Unity.

The accepted native menu has Change Character, Quit Game and Cancel; redundant logout is omitted.
At 1264×681, scale=max(1,height/768×1.05)=1; integer-centered dialog=(529,246,206,188).
Three 175×45 controls start at(13,16),(13,69),(13,122). The source background's 241 px height does
not override accepted 188 px geometry. Recipe, source identity and conversion live in FusionForge's
`recipes/native/ui/quit-menu-direct.json`; run `convert-native-ui <raw-build-root> quit-menu
<native-asset-root>` there. Six image routes and JEFFE replacement font are dependency fields in
the native document. Standard border 6/6/6/4 and Cancel border 5 retain nine-slice corners.

All labels use semantic EN/RU keys. Preserve `fonts/jeffe.otf`,12 px font and 13.71 px line height.
Style `contentOffset` retains source semantics; optional `fontCompensation` is a separate
measured translation applied once to the label, without moving the control/hit rectangle.
Old documents default it to zero. Optional `active` supplies the pressed image; old
documents retain their accepted Standard=Normal/Cancel=Hover fallback. FusionForge can
bind final EN/RU metric captures through `--text-metrics` and preflight with `--check`.
No guessed global JEFFE offset is part of this contract.
Button images use Bevy's `VisualBox::BorderBox`: source padding only constrains text.
The converted style declares `backgroundBox: "border-box"`; older documents default
to that policy. ContentBox is rejected by the native format to prevent shrinking.
The default ContentBox would shrink Standard backgrounds by 12×6 px while leaving
the hit rectangle unchanged. Live Retrobution comparison at 1264×681 also shows
`CHANGE CHARACTER` on two lines: Font 903 advances total 174 px for a 163 px content
width, while the approved replacement's native text layout measures 145 px and fits
one line. Paired `CANCEL` raster bounds share their lower edge but differ in height;
an ink top edge must not be treated as a baseline measurement.
Vertical compensation alone cannot reproduce that wrap, so text parity remains unresolved.
The source's reachable styles explicitly select Font 903; its unused root-font external 1/10102
remains unresolved provenance, not a runtime dependency. Unknown UI adapters/source revisions fail.

Visible menu blocks lower pointer/gameplay input and requests the pointer. Disabled controls emit
no actions. Escape dismissal is distinct from Cancel; open/close use shared open_screen/close_screen
OGG cues. Button cues deterministically choose mouse_click 01..05 at 0.7 gain and the production shell
plays despawning one-shots. Button actions retain their existing typed handlers.

Change Character and Quit Game request PC exit; four-byte PcExitRequest 0104 is sent through
GameplaySender and NetworkCommand::ExitWorld. Keep destination latched, validate PC identity and
transition only on exitCode 1. Rejection releases input and returns to Login; fixed non-1
disconnect strings come from primary TableData. Character change keeps the authenticated login
socket and requests the retained roster after the confirmed shard exit;
Quit Game closes after confirmation. Escape must not reopen in the same frame. Lower movement,
combat and QuickSlot sends remain gated.

Tests cover schema rejection, production EN/RU/font/action contracts, input/disabled/cancel/escape,
geometry, scale and audio. `quit_menu_gpu_preview` exercises actual native loading and text/image
rendering in both locales. `--interactive` leaves the native window open after capture
and preserves real pointer input for hover/click/Escape checks; default captures remain deterministic.
The preview also prints the alpha-mask bounds of each label's glyphs in physical pixels
relative to its text layout. Transparent atlas margins and adjacent glyphs are excluded;
these bounds precede GPU sampling and do not measure the original GUI baseline.
`client-pen-y-by-line` inverts atlas placement to recover the positioned Parley glyph
origin in client pixels. Its Y includes any shaping offset from the line baseline;
it is not derived from ink or decoration bounds. Current JEFFE EN/RU captions have
equal pen Y within each line (289,342,395 at 1264×681 with zero compensation).
WebPlayer PageOut/HomePage IPC and the clean EndGame callback sequence
remain unsupported; non-1 disconnects return to Login with the exact status. No general managed-code
translation or full legacy visual equivalence is implied by this adapter.
