# Character creation regression checks

## Shared-shard tutorial entry (RustyFusion)

FFOne enters the shard with `tutorial_flag=0` after the second Dexter cutscene.
Character selection must acknowledge that entry without marking the tutorial
complete. RustyFusion previously rejected it without a reply, leaving the client
on the tutorial loading screen. The original migration smoke missed this because
it completed the tutorial before selecting the character.

On an isolated RustyFusion database and separate local ports, run
`server_migration_smoke` with `FFONE_ISOLATED_TEST_DATABASE=1`,
`FFONE_SMOKE_TUTORIAL_ENTRY=1`, `FFONE_LOGIN_ADDRESS`, `FFONE_USERNAME` and
`FFONE_PASSWORD`. Use a fresh test account. The probe creates the character,
enters with the tutorial's scripted position before loading completion, checks
world bootstrap/Nano pages and exits. Restart the isolated server and repeat the
same probe to verify the persisted tutorial flag is still zero. Then restart it
and run the same account without `FFONE_SMOKE_TUTORIAL_ENTRY` to check completion
and ordinary world entry. Never point this probe at the live DB.

For independent render/loading checks, `FFONE_CUTSCENE_PROBE_OUTPUT` captures both
production Dexter cutscenes. `FFONE_PERF_OUTPUT` plus `FFONE_PERF_TUTORIAL=1`
enters the production tutorial scene offline at its starting position and angle;
the normal loading barrier must open before its frame/report are produced.
These GPU fixtures do not log in or write account/settings data. Keep outputs
under ignored `target/performance`.

September 18 verification reproduced the server rejection before the fix and
passed both network routes after it. Both cutscenes and the tutorial scene loaded
with the production renderer. Reports are under `target/performance/tutorial-*`.

The full-client follow-up also exposed a second handoff failure: RustyFusion
closed the authenticated login socket immediately after SHARD_SELECT_SUCC.
FFOne's retained login reader then emitted Disconnected after WorldReady, before
the scene finished loading. The server now retains that connection for
keepalives, tutorial completion and subsequent selection. Client disconnect
cleanup releases the loading overlay even when WorldReady has already cleared
the pending selection UID.

`FFONE_TUTORIAL_NETWORK_PROBE_OUTPUT` runs the real network worker and renderer
through login, character selection, the Dexter cutscene and tutorial admission.
It requires `FFONE_ISOLATED_TEST_DATABASE=1`, `FFONE_LOGIN_ADDRESS`,
`FFONE_USERNAME`, `FFONE_PASSWORD` and an existing unfinished test character.
It checks twelve seconds of gameplay after admission to cover login/shard
keepalives, then writes a screenshot and `passed.txt`. Against a deliberately
disconnecting test server, add `FFONE_TUTORIAL_NETWORK_EXPECT_DISCONNECT=1` to
require return to Login with no stale tutorial overlay. Neither fixture changes
the user settings. Reports live under `target/performance/tutorial-full-*` and
`target/performance/tutorial-disconnect-fixed`.

The final release pair was also run from `FFOne/Client` and `FFOne/Server`, using
their packaged assets and an isolated database on ports 24300/24301. It passed
tutorial admission, twelve seconds of gameplay, normal shutdown and a subsequent
login/shard-entry smoke without a server restart. See
`target/performance/tutorial-package-final`, `tutorial-package-reconnect.log`
and `tutorial-package-verification.json`; the last file also verifies installed
binary hashes and that the user's database was unchanged.

The idle login connection must also answer server keepalives while no UI commands
are submitted. OpenFusion disconnects after 32 seconds without a heartbeat. The
network worker services the menu connection every 100 ms on the same socket owner;
packet replies and creation commands retain one encoder sequence. The worker test
withholds appearance submission until three fragmented keepalives have been answered,
then verifies creation and retry on that same connection. This covers time spent in
selection, introductions, name entry and appearance/resource loading without extending
the server timeout.

Run `powershell -File ../FusionForge/tools/native/test-character-creation.ps1` from FFOneClient for the
client state/loading, native appearance data, UI/localization, network worker and
encrypted login-protocol checks. No live account is required.

Add `-Gpu` to run the production application with real asset loading and rendering.
The opt-in `FFONE_CREATION_PROBE_OUTPUT` fixture traverses character selection,
Dexter's introduction, the name screen, reserved-name admission, animated randomized
appearance, the extended female palette, return to selection and resumption of the
unfinished character. It uses the production state schedules and readiness barriers.
It simulates the reserved-name response; real packet sequencing is covered separately
by the loopback worker/protocol tests. It does not log in or persist user settings.
Each run writes screenshots, the last readiness status and a completion marker under
ignored `target/performance/character-creation`. A missing marker or 180-second
timeout is a failure, even if the application exits normally.

Add `-Server` when the sibling OpenFusion development checkout and its C++ toolchain
are available. `-Make`, `-Cxx` and `-Cc` can specify tool paths. This runs the actual
server validator against every palette combination declared in the native client
customization JSON (38,880 combinations at present), invalid field boundaries and
invalid starter equipment. This cross-repository check is development-only; neither
runtime requires the other repository's source or data to start.

The September 2026 regression was a mismatch between expanded native palettes and
the login server's old color limits. Eye color 6 already failed with ordinary skin,
while misplaced parentheses let other invalid values bypass the check when skin or
height was outside the old range. The server now admits the native creation ranges
(skin 1–36, hair 1–54, eyes 1–10) and validates every base field independently of the
others. Existing gender-specific face/hair-style and level-one item checks remain.
Deploy/rebuild OpenFusion as well as the client when changing this shared contract.
