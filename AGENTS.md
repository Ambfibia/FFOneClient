# FFOneClient

Native Rust/Bevy FusionFall runtime, editors and editable `assets/game`. Server:
`../RustyFusion`, FFOne / Retrobution 0104; OpenFusion is reference only.
Preserve accepted IDs, variants, EN/RU and replacement fonts. No Unity/runtime
bundles, parallel `content/` tree or conversion tooling here.

Search affected code first; read matching ranges, not entire files or linked trees.
`docs/source-map.md` locates owners. `.ignore` excludes generated code, fixtures,
probes, recipes and history; use `rg --no-ignore` with an exact path when relevant.
Do not reload unchanged references, generate unrequested reports or run global suites
for local changes. Widen checks for shared APIs/build changes; verify UI interactions
in the real client. Keep cohesive files normally below 1,000 lines, not arbitrary cuts.

| Change | Contract |
| --- | --- |
| Gameplay/network | `docs/server-gameplay.md` |
| UI | `docs/native-ui.md` (choose one screen) |
| Text/voice | `docs/localization-and-voice.md` |
| Assets | `docs/native-asset-workflow.md` |
| Rendering/performance | `docs/performance.md` |

Only explicit Unity research/import or a bug requiring legacy evidence uses
`../FusionForge/AGENTS.md` and its reverse-engineering skill. Native features and
ordinary bugs do not. Preserve unique regression tests; unrelated cleanup, conversion
and parity audits are not prerequisites.

Active user bug tasks live in `docs/bugs/README.md`. For "исправляй Bug N", check
that N is active, then read `docs/bugs/AGENTS.md` and `docs/bugs/bug-NNN.md`.
Keep Bug IDs stable. Already fixed/code-fixed items are excluded by user request;
the archived audit is not a work queue and does not require re-verification.
