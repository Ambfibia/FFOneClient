# Shared text editing

Use the shared edit owner with scalar caret/selection and bounded UTF-16 serialization.
Physical Ctrl+A must work on RU keyboards. Support selection replacement, Backspace/
Delete, caret arrows/Home/End and Shift extension. Pointer hit testing uses shaped
advances including spaces; password input uses masked layout. Long content clips and
scrolls at its original font size, never label auto-fit. Chat retains history Up/Down
and distinct Enter/SEND actions. Respect owner focus, modal and cancellation boundaries.

Capture cursor geometry after Bevy has shaped the current text. Label auto-fit waits
for a nonempty computed node; hidden/provisional minimum width must not permanently
shrink login copy. Measure physical shaped width and summed line heights divided by
layout scale, not GPU allocation rounding or selection-decoration bounds. Editable
text is excluded. Localization/font behavior is canonical in
[localization and voice](../localization-and-voice.md), not a second edit implementation.

The login GPU preview must use production localization/edit components and compare
pointer positions with every shaped boundary, including Cyrillic. This tests presentation
and editing, not live authentication. Shared service drafts register after DefaultPlugins
and the explicit Deferred startup state; gameplay assets are not login prerequisites.
