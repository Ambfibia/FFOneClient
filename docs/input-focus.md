# Desktop focus recovery

The September 5 repair cancels transient input when the primary game window
loses focus. Bevy 0.17.3 `keyboard_input_system` handles `KeyboardFocusLost`,
but `mouse_button_input_system` only applies received press/release messages.
Releasing a mouse button outside the game therefore can leave its held bit set.
The next press then fails to produce `just_pressed`. A retained UI `Pressed`
interaction can also continue to consume the gameplay pointer.

`input_focus::GameInputFocusPlugin` runs after `InputSystems` and before
`UiSystems::Focus`. It clears held/edge keyboard and mouse state, pending text,
button and wheel messages, accumulated mouse motion/scroll, and UI interactions
while unfocused or after a primary-window focus-loss event. It does not fabricate
a release that could commit a drag/drop action. A loss/regain batch in one frame
is cancelled once. Events for other windows do not cancel the primary input.

The existing gameplay cursor owner additionally checks window focus and this
cancellation boundary. It releases the cursor while unfocused and recomputes
the desired lock from current gameplay/menu state on return. It does not restore
a stale menu snapshot.

Verification:

- `cargo test -p ffone-client --lib input_focus::tests -- --nocapture`: two passed.
  These use the production `InputPlugin` and avatar action-input reader, covering
  lost releases, cancellation without a release edge, stale UI capture, camera
  motion/scroll, the first return click, batched focus changes and another window.
- `cargo build -p ffone-client --bin ffone-client`: passed, with the existing
  unused `CameraProjection` warning.
- The full-client fixture with `FFONE_PERF_INPUT_FOCUS=1` and
  `FFONE_PERF_OUTPUT=target/performance/input-focus-20260905` completed 600 frames.
  Its `input-focus.json` confirms cancellation without a release, visible/unlocked
  cursor while unfocused, locked/hidden cursor after return, and a fresh first
  click. The fixture feeds the same window-state changes and `WindowFocused`
  messages produced by Winit, through the ordinary application schedules.
  This is a simulated focus transition, not an automated physical OS Alt-Tab.
- Logs: `target/parity-input-focus-tests.log`, `target/parity-input-focus-build.log`
  and `target/performance/input-focus-20260905/`. Runtime stderr contains only the
  fixture's settings-persistence warning.

This is a native desktop input-adapter repair. No Unity behavior, extracted
payload, UI geometry, text or voice content was reconstructed in this change.

## Gameplay pointer ownership

The September 6 repair makes UI consumption respect the hidden, locked gameplay
cursor. Bevy UI hit testing uses the last absolute cursor position even during
relative camera input. Treating every hovered or pressed HUD node as a pointer
owner could therefore suppress mouse attack/use while the Z binding still worked.

`GameplayPointerCapturePlugin`, installed by the avatar action plugin, snapshots
ownership in `PreUpdate` after `UiSystems::Focus`. Hidden and locked gameplay
cursors ignore HUD hover; visible menu pointers retain the existing interaction
and nonzero-node-size check. Both default and configured action readers consume
this same snapshot. Closing a dialog and locking the cursor during `Update`
cannot turn that dialog click into a world action in the same frame.

Regression coverage lives in `input_focus::tests`, the avatar dialogue-click
test, and `app::hotkeys::tests`: physical input messages, held/repeated clicks,
focus recovery, configured bindings, and dialog-close ordering.

September 6 verification: all 8 `app::hotkeys::tests`, 3 `input_focus::tests`,
and 23 `avatar_action::tests` passed. These are schedule/input regression tests;
they do not constitute a manual in-world mouse test.
