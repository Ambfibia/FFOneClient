# FFOneClient Editor

`ffone-editor` is the native Bevy content browser for published NPC, Nano and
player equipment assets. Its NPC/Nano library comes directly from the production `TutorialMissionContent`
XDT projection and stays sorted by numeric NPC/Nano ID. Models resolve through
`NetworkNpcVisualCatalog0104` and `GameplayNanoPortraitCatalog`; ordinary NPCs
are spawned and textured by the same production systems as live world NPCs.

Run it from the repository root:

```powershell
cargo editor
```

Build both the game and the content editor in the release profile with `cargo release`.
On Windows, the editor executable is `target/release/ffone-editor.exe` (on Linux,
`target/release/ffone-editor`). To build only the release editor, use
`cargo build --release --package ffone-client --bin ffone-editor --locked`.

Russian is the default editor language. Select English at startup with:

```powershell
cargo editor -- --language en
```

The editor provides:

- separate, continuously scrollable NPC and Nano XDT lists with no catalog pages;
- exact NPC `m_iIcon1 -> m_pNpcIconData -> AvatarUtil` portraits and Nano icons;
- strict numeric ID ordering without registry-only rows;
- Unicode search by display name, semantic ID, logical root, or network ID;
- XDT row, scale, height, style, level/set, and texture metadata behind **Details**;
- production GLB resolution, Scene0 loading, root policy, material shader, and NPC texture variants;
- the production 45-degree MainCamera projection, no-MSAA profile, and neutral 8,000-lux world light;
- explicit frozen default-pose, geometric T-pose from the native bind rig, and named clip playback modes;
- named animation selection, playback, looping, speed, and timeline controls;
- orbit, zoom, reset-view, and opt-in turntable controls;
- explicit static-model and no-ordinary-native-visual labels for exact XDT rows;
- key-first English/Russian UI with `F9` language switching;
- flat panels and buttons, fixed column widths, and a single-line search field
  whose height and width do not depend on its text or result count;
- an **Equipment** tab with category filters and male/female fitting on the
  production shared player rig, including the exact item material and attachment rules;
- a camera viewport isolated from the header, library, inspector, and playback chrome.

Mouse drag orbits the camera and the mouse wheel zooms. The initial model is
stationary and frozen on frame zero of `stand1`, or the first available clip
when `stand1` does not exist. `DEFAULT POSE` (or `D`) restores that frozen
authored frame. `T-POSE` (or `T`) reloads the scene without animation and aligns
recognized upper-arm/forearm bind chains horizontally. Models without those
native biped bone names remain in their exact published bind pose rather than
receiving a guessed skeleton edit.

`Space` toggles playback, left/right selects the previous/next clip, `R` resets the view, and
`Ctrl+F` focuses search. `↑` / `↓` selects the previous/next visible item in
the active NPC, Nano or equipment list, including while search is focused.
Navigation respects search/category filters and scrolls the selected row into view;
it stops at the ends of the list. **Fit model** / `R` frames the loaded geometry with a
margin using the narrower viewport field of view. Manual zoom reaches 120 metres.

In **Equipment**, select a gender and then a category or search for a name/ID.
Clicking an item replaces its slot while keeping the other slots. Male and
female outfits remain independent for the current editor session. **Reset
outfit** restores the default clothing and removes accessories. The inspector
lists the current outfit; gender-incompatible items and failed native routes
show an explicit error. The fitting view uses the production idle animation;
NPC/Nano clip controls apply only to the NPC/Nano view.

Reproducible rendering checks can start with `--equipment shirt/1 --female`,
`--npc 1`, `--search TEXT`, `--language en`, or `--compact` (1180×720).
`--capture target/performance/editor/example.png` writes an image and adjacent
JSON status including the actual computed search-field size, then exits.

The persistent header has separate NPC, Nano, Equipment, Strings and XDT tabs.
Each table editor keeps its own draft, selection and search when switching tabs;
model tabs retain their selected entry and search. The model browser is read-only.
The **Strings / Строки** tab edits the native
English and Russian text bundles. Both views consume ordinary native runtime assets below
`assets/game` and never open a legacy client, extraction cache, or
FusionForge project.

## String editing

Open **Строки** in the header, or run `cargo editor -- --strings`.
The tab is a continuous two-column table: the English original on the left,
the Russian translation on the right. Click either text column to place the
caret and edit it directly. Find searches English and Russian text without filtering rows.
There are no pages or separate detail cards. Mouse wheel or the scrollbar moves
through all rows; only visible rows are instantiated, even for the
65,000+ entry production bundles. Long rows wrap and scroll with the table.

The toolbar contains a row filter and Save. Ctrl+F opens Find with Previous/Next; Ctrl+H reveals replacement. Autosave runs after one second
without typing, with a quiet status line at the bottom. Ctrl+S saves immediately;
Ctrl+F focuses search. Text editing supports Unicode, Enter, left/right,
Home/End, Shift selection, Ctrl+A/C/X/V and Ctrl+Z/Y. Shift+click extends selection.
Escape ends editing. The caret is one pixel wide; selected text has a colored background.
`--strings --search TEXT` opens the table with a preset search.

External changes are checked every two seconds while this tab is open;
focused drafts are merged on save. English owns the key set: new keys receive
the English fallback in the Russian draft. Conflicting edits, unknown Russian
keys and placeholder mismatches block saving and leave the draft intact.
Synchronization and draft-discard actions appear only when an error needs
attention. Discarding all drafts requires a second click.

Saving writes edited `localization/en.json` and/or `localization/ru.json` under the selected `--asset-root`,
preserves extra document metadata and unrelated external translations, and
validates both locale drafts before writing and atomically replaces each changed JSON using a temporary file in the same directory.
The editor reloads localization after saving. Restart an already running game
to read the updated bundle. Closing saves valid pending edits; a failed save
keeps the editor open. Unsaved drafts remain in memory and do not survive a
forced process termination.

### Export/import selected rows

Use the slim checkbox at the left of each row to select rows. Ctrl+click toggles individual rows; Shift+click selects the range
between the first and last row, including rows outside the viewport. Ctrl+A
selects all rows when neither text nor search is being edited.
Selection persists when searching or scrolling; the footer shows the total.
Escape clears row selection when no text field is active.

Right-click a selected row for **Экспорт… / Импорт…**. Right-clicking an
unselected row selects only that row. A native file dialog chooses the CSV;
exports default to the ignored `target/localization` directory. These actions
are currently backed by Windows native dialogs.

CSV is UTF-8 with a BOM and columns `key,en,ru`. Export contains exactly the
selected semantic keys, their English originals, and current Russian drafts.
Copy the English column into your translation workflow and put the result in
`ru`. Keep `key` and `en` unchanged. Quotes, commas, Cyrillic, and embedded
newlines round-trip without flattening dialogue. Column and row ordering may
change; import matches semantic keys, not positions.

Select the same rows and choose Import. The entire file is validated before any
translation is applied: selected key sets must match exactly, keys must be
unique, English source text must still match, and template placeholders must
be preserved. Changes made while the file dialog is open are checked as well.
A rejected file changes no translations. A successful import changes only the
selected Russian values, is autosaved, and is one undo/redo step with Ctrl+Z/Y.
No content is sent to a translation service by the editor.

### Keyboard navigation, search and replacement

- Ctrl+Enter moves editing to the next row in the active language column in the current
  sorted table and scrolls it into view. The last row stays selected. Enter
  inserts a newline as before; edits keep the existing autosave behavior.
- Ctrl+F focuses search and selects the existing query. Click or drag in the
  field to position/select text. Arrow keys, Home/End, Ctrl+Left/Right,
  Shift selection, Ctrl+A/C/X/V, Backspace and Delete work within the field.
  Long queries scroll horizontally to keep the thin caret visible.
- Ctrl+H opens the replacement row. Tab switches between search and replacement.
  **Replace all in filtered rows** replaces text in the chosen EN/RU column
  only within the current row filter. Keys are unchanged. An empty search
  does nothing; an empty replacement deletes matches. Placeholder validation
  happens before the batch is applied, and Ctrl+Z undoes the whole batch.
- Click English or Russian headers to sort ascending; click again for descending.
  The arrow shows the direction. **#** restores semantic-key order. Shift range
  selection and Ctrl+Enter follow the displayed order, including offscreen rows.
- The last active tab (strings, NPCs, Nanos or equipment) is saved atomically to
  ignored `target/editor/preferences.json` when it changes and restored on launch.
  Explicit `--strings`, `--npc` or `--equipment` arguments override the saved tab.


### Find without filtering

Ctrl+F focuses Find. Typing never removes rows: English and Russian text are
searched without changing the table order, so neighboring dialogue stays visible.
Enter / F3 and Next move to the next match; Shift+Enter / Shift+F3 and Previous
move backward, wrapping at the ends. The matched text is highlighted and the
footer shows the match index and count. Search and replacement ignore case.
Ctrl+H opens Replace with both Replace (current match) and Replace all, plus an
explicit English/Russian target. The active editing column is selected initially.

Tab while editing moves to the next cell in the active language column and selects its entire text for
Ctrl+V. Shift+Tab does the same for the previous row. In the search/replace bar,
Tab switches fields. Drag within either text column to select text; Shift+click and
Shift+arrow keys also extend selection. Selection uses a colored background and
does not insert any visible marker characters into the text.


### Separate filter and Find fields

The permanent top field filters rows by key, English or Russian text, ignoring
case. Clearing it restores all rows. Ctrl+F opens a separate Find bar; Ctrl+H
also opens replacement. Find has its own query and traverses the currently
visible set without changing the filter or hiding neighboring visible rows.
Closing Find keeps the filter intact.


### Filter modes and replacement options

Click the filter-mode button to cycle Normal -> Strict -> Exclusive. Normal
matches substrings (`бег` matches `побег` and `беготня`); Strict requires whole
word boundaries (`бег` within a sentence); Exclusive requires the entire trimmed
cell or key to equal the query. Filtering ignores case in every mode.

Replacement has independent Match case and Whole word checkboxes. With both off
it replaces substrings without regard to case. Match case preserves the distinction
between `БЕГ` and `бег`; Whole word prevents changing `беготня` for a `бег` query.
The current filter is snapshotted before the batch is applied, so changing a value
cannot cause additional rows to enter the same replacement. Both Replace and
Replace all leave hidden rows untouched. Ctrl+Z/Y retains the language of each
history step, including batches. Placeholder mismatches reject the entire batch.

English and Russian edits merge independently with external writers, retaining
metadata and unrelated changes. Save validates EN/RU keys and placeholders before
replacing either file. Each file replacement is atomic; the pair is not a filesystem
transaction. If a write fails after the first file, drafts remain available to retry.

## XDT table editing

Open **XDT таблицы / XDT tables**, or `cargo editor -- --xdt`.
The editor discovers object-row arrays in the actual native
`assets/game/data/tables/xdt.json`, including native asset routes. Gameplay tables
such as `m_pMissionTable` are at the JSON root, matching RustyFusion's XDT loader.
The `_ffone` extension (`ffone.xdt.v1`) preserves native audio/model routes, accepted
table identities and other client metadata; the server ignores this extra section.
Native consumers use a lossless named table view in memory. Saving always writes
the server-shaped document, including `_ffone`, so ordinary editing never strips
client extensions or requires a separate gameplay export.
It preserves table keys, metadata, unknown fields and nested values.
`--xdt-table m_pNpcTable/m_pNpcData --xdt-row 1` selects a table and a
zero-based source row; `--search TEXT` presets its row filter.

The left column shows the main game-data tables by default. **All tables** opens
supporting tables; searching includes all tables, and a table opened through a
reference stays visible. Search accepts readable names and technical paths. The center is a short
record list with ID, resolved name and key parameters; search includes names from
linked text rows. ID and index headers toggle ascending/descending order, with an
arrow showing the active direction. Sorting changes only the displayed row indexes;
indexed XDT references and the stored array order remain intact. Wheel and page buttons
move through records without changing their source indexes.

Select a record to open its parameter card, grouped into identity, appearance,
combat, mission flow and additional settings. Editing stays in the right card,
so the record list keeps its height. Parameter names and values sit side by side;
long names and values wrap within the card. Search,
toolbar and grid cells stay on one line with a fixed readable font size.
The panels keep independent widths and padding in compact and normal windows.
Click a parameter to see its description and edit it;
**Apply / Ctrl+Enter** commits its draft, **Esc** cancels the field.
NPC/Nano, mission, item and reward records have semantic authoring blocks. Mission
blocks separate conditions, objectives, start, success and failure; items separate
requirements, appearance, combat, economy and bonuses. Blocks containing only zeros,
empty strings or recursively empty lists start collapsed unless they contain required
creation parameters. Expanding/collapsing only changes the view; it never deletes,
disables or rewrites content. All fields includes unknown source parameters, and
parameter search also searches collapsed blocks. Custom required parameters remain
visible in the creation form after returning to the basic view.
List fields open individual elements, preserving list length and other elements.
Declared reference elements use the same named picker as scalar references. Optional
ID routes offer No linked record; indexed name/model/icon routes retain their meaning.
Field details show the value type, required status and existing native constraints.
For linked text, the picker shows the number of distinct owners in declared routes:
Edit shared text follows the original row; Create a private text copy clones the
entire text row (including unknown fields and optional prose), retargets just the
chosen owner field, and opens the copy. Copy and retarget are one undo step. Other
owners and existing IDs/indexes stay intact. Back returns to the owner.
In mission tasks, New mission allocates both a new task ID and a new mission-group
ID with its own title. Mission stage adds an independently identified task to the
selected mission, retaining its group and shared title. Show this mission's stages
filters by the actual mission ID; Show all missions returns to the complete list.
Adding a stage inserts it after the selected stage and retains the previous outgoing
transition as the new stage's successor. Identity settings and Journal NPC are copied
from the selected stage, while the task receives its own ID and objective text.
Mission tasks open a three-pane workspace: searchable mission catalog, stage canvas
and inline properties. The blue theme reuses native option buttons, mission card
frames, category icons and NPC portraits; canvas and panel backgrounds remain plain
so game artwork is not stretched across unrelated controls. All property sections
start collapsed. Enum values show their meanings and codes, and accept only the
verified choices. Reference fields search names, IDs, descriptions and EN/RU text;
the adjacent + creates and links a record without leaving the mission workspace.
Waypoint NPC fields resolve NPC names and IDs. Journal references resolve their
summary/description and array index, including EN/RU search. The adjacent + creates a
journal entry with all six text references, each allowing nested text creation.
Nested creation, translations and the final link are one Undo operation.

Stage cards show the authored objective and only populated gameplay goals, including
NPC/item IDs, counts and matched quest-item drop rates. Journal, NanoCom and overhead
speech previews follow in event order: start, completion, then configured failure.
Each preview identifies its event, text/journal ID and actual speaker. They use native
blue journal, dark NanoCom and pale-yellow quest-speech backgrounds. Long preview text
is shortened; clicking a preview opens the full linked field in the inspector.
The header's collapse/expand toggle hides these details while retaining the objective,
task ID and connection ports. Its state is saved with the editor layout; it never alters
XDT, task order or dependencies. Edges follow the current card height in both states.
Card height and automatic layout follow the configured content; arrow endpoints use
each card's dimensions. Existing manual positions remain editable with Auto layout.
Overhead speech fields select mission strings by text/ID and support inline EN/RU
editing or private copies. Journal entries can be edited in place with nested text
editing. Both new and existing indexed drafts show their exact text/journal ID;
zero-valued optional references create a new text instead of editing the neutral row.

Stage flow normally exposes a single Next stage connection. The After completion
section holds the next stage and completion effects. Unconfigured failure sections
stay hidden until All fields is enabled; existing failure routes remain visible and
are preserved. Mission chains use `m_iCSTReqMission` as completed mission IDs;
stage transitions use task IDs. Drag card headers to arrange the canvas, wheel to
zoom around the cursor, or use right-drag, middle-drag / Space-drag to pan. A short
right-click opens the context menu; moving at least five pixels pans without opening
the menu on release. Fit all and automatic
layout are available below the canvas. Right-click nodes, connections or empty canvas
for contextual actions. Right-click elsewhere to relocate the menu; left-click outside
or Escape dismisses it without clicking through. Menus use a plain blue panel, with no
close button. A connection can be selected along its line and dragged to another target.
Layout changes retain the catalog and inspector while updating only
the canvas. Layout is saved separately from gameplay tables and supports Undo.
New prerequisites reject cycles; existing cycles and missing targets remain visible.

Stage diagrams include compact mission-name/ID cards above the entry stage for opening
prerequisites, and below terminal stages for each unlocked mission. Clicking a compact
card opens that mission. In Mission chains, drag **Unlock mission** to another mission
or click the port then select the target. This writes the source mission ID into the
target mission's first native task prerequisite slots; several successors are supported.
All incoming prerequisites are required, and other opening conditions still apply.
Removing or retargeting an opening dependency also clears its legacy repeats on later
tasks. IDs and task-array order stay unchanged. These edits use normal validation,
Save and Undo/Redo; a failed connection leaves the previous data intact.

NPC cards and NPC creation offer HNPC appearance; Create HNPC starts a new NPC draft.
Create placeholder is a separate NPC action, also used by + on mission NPC references.
It copies the complete Location A256 (NPC 1401) settings, allocates a new NPC ID and
an owned name. Ordinary creation and Duplicate retain their own template behavior.
Creating a placeholder defines an NPC type; it does not place an instance in the world.
The appearance view uses the production shared rig, published part/texture variants,
gender-compatible choices, all skin/hair palette colors and native height/build
selectors. Rotate/zoom the model while configuring face, hair, clothing, accessories
and weapon. The part picker searches model and texture names, retaining Unicode
selection/copy/paste editing; pagination adapts to window height. Unset palette
selectors preserve the runtime's white fallback until explicitly changed.
Back discards the appearance draft; appearance Undo/Redo is independent
of XDT Undo. Save as own appearance appends an index without reindexing existing
looks; Save shared updates the current look and displays its owner count. Both
validate the full native catalog before atomically writing `data/hnpc/catalog.json`,
then apply `m_iHNpc`/`m_iHNpcNum` to the NPC's XDT draft. Save XDT separately to
publish that NPC link. External changes to other appearances are retained; an
externally changed shared look requires reopening before overwriting it.
Declared references use a searchable picker with readable target names instead of
requiring a manually entered index. **All fields** exposes every original parameter,
technical keys, whole-row JSON, CSV and reload controls. Descriptions distinguish
verified native behavior from undocumented source parameters.
Strings accept ordinary Unicode text; numbers, booleans, arrays and objects
use JSON syntax and retain their field types. **Edit JSON** edits nested data and
optional fields as JSON. **New record / Duplicate** opens a separate creation form
using the selected record's settings, or the first populated record when nothing is
selected. Verified NPC, Nano, task, reward, tuning and item identity fields receive
a free ID. Required native parameters are marked ★; All fields allows additional
requirements. Creation validates ranges, names and declared targets and inserts only
on **Create record**. Switching fields does not insert a record; unfinished creation
blocks saving or closing. A new NPC/Nano/task/item accepts its new name directly
in the form and creates the corresponding text row together with its owner as one
undo step. Optional prose in that new text row starts empty. **Choose existing text**
reuses a shared name; Duplicate initially keeps existing text links and also offers
**Create a new name**. Model/icon rows stay shared; the form names the settings template.
Cancel discards both the owner and pending text without changing the document.
Unique populated ID columns are checked before accepting changes. Delete requires a second click.
Ctrl+A/C/X/V, Shift+arrows, mouse selection and Ctrl+Z/Y work in fields.
Tab / Shift+Tab commits a valid cell and moves to the next / previous field;
Ctrl+Enter commits without moving. Outside a field Ctrl+Z/Y undoes/redoes table
operations, including CSV import.

The card's **Connections** tab shows declared outgoing references (→), incoming uses (←)
and, in All fields, separately marked ID candidates (?). NPC/Nano icon, name and mesh routes
use source array indexes; mission NPC/task/reward/Nano routes use declared IDs.
Click a reference to open its row, and Back to return. Missing declared targets
are shown explicitly. Candidates match ID domains/values only when at least one table has a unique
identity column; repeated counters do not generate links. They are not a complete
foreign-key schema. Saving rejects newly broken declared references and silent retargeting of indexed
rows after deletion or reordering; existing
unresolved source routes do not block unrelated edits.

Export/import uses native Windows dialogs and the **entire selected table**,
including filtered-out rows. CSV is UTF-8 with BOM. Every present cell contains
a JSON value (strings include JSON quotes); an empty CSV cell means an absent
field. This preserves nulls, empty strings, arrays, objects and multiline text.
Import validates all rows before applying one undo step and rejects changes
made to the table while its dialog was open. It does not save automatically.

Save / Ctrl+S atomically replaces the native document after a three-way merge
with external edits. Conflicts leave both disk and draft intact. Reload refreshes
from disk; discarding changes requires a second click. Closing saves valid
pending edits and stays open on an error. Drafts live in memory. Restart the
game after saving to reload gameplay catalogs. Model viewers refresh from the saved
tables when next opened, preserving the current outfit and selected semantic entry.


## Icon generator

Open **Icon generator · 128 × 128** in the NPC, Nano or Equipment inspector.
The square preview shows the actual composed export at 1×, 2× or 3×. Left drag
orbits; right/middle drag moves the camera target; the wheel changes distance.
Shift + left drag changes roll continuously; Shift + wheel changes FOV.
Front/back/side/top presets, roll, FOV and Fit control the framing.

Nano layers include the affinity-colored contour, optional background, locked
silhouette, A/B/C affinity symbol and acquisition mark. **Source badge** extracts
the original ready-icon mark and number; some Nanos have none. **By level**,
**World Nano** and **From item** reuse the published atom/globe/item artwork.
Level numbers are assembled from the original numeral sprites (0–9), with an
editable two-digit value; they do not use a substitute system font. Click the
number to type, Ctrl+A selects it, Enter accepts it.

Equipment uses the production shared rig and resolved texture variants, keeps
only the selected part, freezes it in a geometric T-pose, and hides clothing's
exposed-skin submeshes. Male/female variants can be framed separately; **Both
genders** renders and exports each in sequence. Gender restrictions and missing
native models remain errors; no substitute model is used.

Exports go to `target/editor/icons`: RGBA PNG **128 × 128**, plus a JSON sidecar
with semantic identity, gender, camera and layer settings. Existing names receive
a suffix. **Last framing** restores the newest exported settings for this model
and gender. Rendering uses a 512×512 transparent target and premultiplied
downsampling; saving waits for visible rendered pixels. Published game icon
routes are unchanged.

For local capture checks use `--icon-generator`, `--icon-export`, `--icon-locked`
and `--nano ID` together with the existing `--capture`, `--equipment` or `--npc`
options.
