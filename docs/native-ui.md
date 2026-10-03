# Native UI: choose the affected screen

This is a task index, not a reading checklist. Open one relevant page/section, then its
code/tests. Do not preload every UI document or legacy evidence file. These contracts
condense the supplied implementation notes through 2026-09-18; implementation status
must still be checked in code. Historical test counts/capture diaries are not current
acceptance. Native previews do not establish live-server or running-Unity parity.

The design reference is 1280×720; many client fixtures use a 1264×681 surface. A decorated
1280×720 window is not a pixel peer until normalized. Apply root/viewport scaling once,
not again after layout. Preserve typed ownership, network authority, approved native
fonts and semantic routes; common text/voice rules live in
[localization](localization-and-voice.md). For an actual Unity migration use FusionForge's
UI converter contract, not manual reconstruction instructions inside this client.

Control backgrounds must cover the full control rect (`VisualBox::BorderBox`), including
padding reserved for captions. Shared sliced backgrounds enforce this; screen-specific
GUIStyle backgrounds must preserve it when replacing images for hover/pressed states.
Content images such as portraits retain their own content-box policy.

- [Gameplay HUD and targeting](ui/hud.md)
- [Nano HUD](ui/nano-hud.md)
- [Mission journal and tutorial prompts](ui/mission-journal.md)
- [Minimap and mission markers](ui/minimap.md)
- [FreeChat and menus](ui/chat.md)
- [Buddy, NanoCom and group UI](ui/social.md)
- [UserEquip / inventory](ui/inventory.md)
- [Shared item cards and service controls](ui/item-cards.md)
- [Vendor](ui/vendor.md)
- [Bank](ui/bank.md)
- [Transportation](ui/transportation.md)
- [Quit menu](ui/quit-menu.md)
- [Quick Slots](ui/quick-slots.md)
- [Buffs and overheat](ui/buffs.md)
- [System messages](ui/system-message.md)
- [Resurrect](ui/resurrect.md)
- [WorldMap](ui/world-map.md)
- [AssetLoader and scene admission](ui/loading.md)
- [Shared text editing](ui/text-editing.md)
- [Email](ui/email.md)
- [Smaller service modes](ui/service-modes.md)
- [Guide Changer](ui/guide-changer.md)
- [Upsell](ui/upsell.md)
- [Options](ui/options.md)
- [Login, character selection and diagnostic UI](ui/front-end.md)
