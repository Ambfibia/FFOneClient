# Text, artwork and voice

Read the affected section only. `localization/**`, `audio/**` and TableData are editable;
syntax, paths/identities and required files remain validated, not locked by content hashes.

## UI text

Add EN/RU keys and identical placeholders on the first pass. Every visible `Text` or
`LegacyBitmapText` uses `LocalizedText::new("semantic.key", "Exact English fallback")`;
dynamic/player/server values enter keyed templates through `.with_arg(...)`. English
source text is not identity; compatibility adoption does not excuse missing new keys.

Production truth: `assets/game/localization/{en,ru}.json`. Test actual bundles, static
label key ownership and repeated placeholders. Terminology edits use
`node --test ../FusionForge/tools/native/test-localization-terminology.mjs`.
Preserve capitalization and contextual Russian inflection; do not globally replace
ordinary words that also occur as names (common/lance/ship/cheese).

For another locale, copy EN, set lowercase `locale`, translate `entries`, preserve
placeholders, and add `{ "id": "de", "text": "localization/de.json" }` to `catalog.json`.
Additional locales may be partial and fall back to EN; maintained EN/RU must match exactly.
Locale discovery/reload is at next launch. No second authored bundle tree.

Mission/NPC/quest-item/world text uses semantic aliases, e.g.
`content.mission.task.<task_id>.*`, `content.npc.<npc_type>.name`,
`content.quest_item.<item_id>.name`. Translate base text before composing live progress.
Legacy TableData text import belongs in FusionForge; old dump-based importer commands
are not a required authoring step or proven direct adapter.

## Geometry, fonts and fitting

Localization must not move/resize/overlap accepted controls. Preserve approved native
replacement fonts. OptionMode uses `OPTION_JEFFE_FONT_PATH`, `OPTION_COMIC_FONT_PATH`
and `OPTION_CHALET_FONT_PATH`; legacy bitmap fonts are metric evidence, not replacements.

`UiTextAutoFit` covers fixed pixel labels or the sole in-flow label of a fixed parent,
subtracting padding/borders. Content-sized/shared regions need explicit fit ownership;
explicit regions win. Shrink font and absolute line height together down to 6 logical
pixels; restore metrics for shorter text and changed rectangles. Short source Rects
retain intentional one-line vertical overflow and fit width. Unresolved overflow needs
a shorter translation or explicit design decision, not moved geometry.

Wait for a nonempty computed node. Measure physical shaped width/summed line heights
divided by layout scale, not GPU allocation rounding or selection decorations. Editable
fields keep font size and scroll; they are excluded from automatic fit.

## Localized artwork

Canonical layouts/fonts/images: `ui/en`; optional RU PNGs mirror paths in `ui/ru`.
Index once at startup via `AssetLocator`; missing RU uses EN. `Language.effective`
switches images independently of voice, including after hover/toggle writes; unchanged
handles are not dirty. Preserve dimensions, alpha and image mode. Selection music
`musicon` / `musicoff` remains 86×30; supplied RU images were assigned by visible action,
not misleading source filenames. Fitting/translation is an extension, not parity proof.

## Voice identity and SFX

Own-voice dialogue/greeting/grunt/scream/wound/death is voice. Footsteps, weapons and
constructed creature sound design are SFX, organized by owner:
`audio/sfx/creatures/<monster>`, `characters/<name>`, or the actual ui/combat/movement/
vehicles/environment/world_events/nano/nano_skills/character_creation bucket. No catch-all
shared owner. Fusion clones are separate speakers; spelling aliases of one speaker unify.

Bundle scope does not define a voice line. Keep creature `/zone_local/` and
`/tutorial_audio/` scope where runtime routing uses it. Cutscene cue policy controls
replacement/overlap, while row category controls bus/language. Deduplication requires
same speaker, `trueName` and audio packet payloads, not whole OGG bytes (serial/CRC vary).
Male/female avatar and world/vendor semantic rows remain independent even with shared audio.

## Voice routing and replacement

`NativeAudioCatalog` is an in-memory exact-path index of TableData
`native_asset_routes.value.m_pAudioData`. A row path such as `ben/ben_tut01.ogg` is relative
to `audio/voice/<locale>/`. Every localized player carries `LocalizedVoice` with semantic
`true_name`, follows `VoiceLanguage`, and never hardcodes an EN/RU route in gameplay/UI.
Text/voice settings are independent. Voice selector lists actual locale directories.

Select the declared variant first, independently of language. Resolve requested locale
→ EN → silence; EN never falls back to RU. Missing files do not trigger another variant
draw or remove a row; undeclared files are ignored. Languages may have independent take
counts, and English files are optional. Check both locales and every new voice spawn.

Replace an OGG in place; no catalog hash update. A localized take mirrors the row path
under another locale. A new variant requires a TableData row, not a filesystem scan.
Retired `_runtime/audio.json` / `characters.json` remain historical FusionForge snapshots.
Mixer/spatial behavior: [audio channels](audio-channels.md), only for those changes.
