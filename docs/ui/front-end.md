# Login, character selection and diagnostic UI

Login (`CnGuiLauncher`, `CnGuiLogin`, `CnGuiServerSelection`, `CnGuiEUALA`) and roster
are separate parity slices, not gameplay-HUD variants. Earlier login extraction notes
lacked complete prefab/GUISkin rectangle, state texture and focus/tab evidence; later
native editing/auto-fit fixes do not by themselves establish complete Unity parity.

The native manual-login panel has a text-language button above registration.
By user request, the three actions use a centered 280×35 column with 8 px gaps,
aligned to the input edges; this is a native visual adaptation of the legacy form.
Discord is omitted by user request.
Input backgrounds cover the full 280×25 control rectangle, including text padding.
It cycles the locales in the text catalog before authentication, updates the same
`Language` resource as Options, and persists through the existing user settings
snapshot. New settings request Russian text and English voice independently;
saved choices and explicit startup language overrides retain their precedence.

Character selection's four source entry Rects are (43,43,413,67), (43,141,413,67),
(43,237,413,67), (43,334,413,67). Offscreen assembled players and camera movement
are required for parity, not static portraits. Preserve admission and loading ownership
from [loading](loading.md); editing rules are [shared](text-editing.md).
Diagnostic layouts/previews are opt-in and must not become normal gameplay or login
UI. Follow the actual source module for a changed diagnostic, not the whole UI catalog.

`character_selection_text_gpu_preview en|ru [OUTPUT.png] [--interactive]` loads the
production selection UI at 1264×681 with a synthetic three-slot roster. It measures
client glyph ink and recovered Parley pen Y per line; avatars/portraits are outside
this capture. Live EN comparison shows mixed text residuals (EMPTY ink top +3 px,
normal slot names -3/-4 px). Quit/Create/Delete backgrounds use BorderBox so label
padding does not shrink their images; state rebinding retains that policy. Text parity
and a shared JEFFE baseline correction are unconfirmed; do not infer baseline from ink.

## Appearance rebinding

Face/eye changes reuse animated geometry for both genders.
`NativePlayerPreviewMaterialBaseline` restores only authored base/emission tints,
not a whole material: renderer/pass order may have changed after initial binding.
Prepare texture replacement on a copy; commit only after loading, leaving displayed
material intact on failure. Readiness includes rebinding after a rig is already visible.
Keep `appearance_rebind_preserves_late_face_render_order_for_both_genders` coverage.
The character-creation preview's fourth argument boy|girl selects gender after two
output paths/capture count. Review changed-face captures; the default cheek/neck
pixel boxes apply only to the original male fixture.

`GameplayUiModel::default()` is hidden and contains no debug/reference strings.
`retrobution_reference_frame()` is only for deterministic comparison. Enable production
diagnostics explicitly outside GameplayUiPlugin.
