# Audio channel and spatial ownership

Bevy 0.19.1 starts queued sources with `settings.volume * GlobalVolume.volume`.
Subsequent changes to either resource do not change an existing sink. The native
`GameplayAudioChannel` component retains the semantic category and original
per-sound gain, avoiding any need to divide by the current volume when unmuting.
Its system updates queued settings before Bevy's post-transform audio startup,
then updates both ordinary and spatial sinks with channel gain times master.
It does not replace players, seek, resume paused sources or change audio assets.
Unchanged volume values are not written back.

Admission covers ordinary gameplay/NPC/character sounds, the gameplay UI click
queue, the inventory loop, and streamed spatial environment emitters. NPC voices
retain catalog resolution, `LocalizedVoice` and independent EN/RU voice language.
Sources with separate fade/envelope owners (music and tutorial cutscene audio)
are outside this scoped admission; their gain behavior requires separate checks.
That gain repair did not establish or modify left/right stereo channel ordering.

The workspace stereo patch corrects the reversed distance-difference term
in rodio's spatial pan law (retained for 0.22.2) through the workspace's local dependency patch.
The listener remains on the camera with its normal left/right ear coordinates.
Per-ear distance attenuation and ordinary stereo playback remain unchanged.
See `vendor/rodio/FFONE-PATCH.md` for the patch scope and removal condition.
Device-free spatial tests cover screen side at four headings, near/far distances, centered gain and movement without restarting playback.

```powershell
cargo test --manifest-path vendor/rodio/Cargo.toml --lib source::spatial::tests --target-dir target/spatial-regression
```

Verification commands:

```powershell
cargo test -p ffone-client --lib audio_channel::tests -- --include-ignored --nocapture
cargo test -p ffone-client --lib gameplay_audio::tests -- --nocapture
cargo build -p ffone-client --bin ffone-client
```

The explicitly ignored device test is run manually in the first command. It
creates paused ordinary and spatial Bevy audio sinks and checks voice-only mute,
independent effects/ambient mute, restoration, and master gain without restarting
the sources. The device-free test covers pending sources and proves that master
is excluded from `PlaybackSettings`, preventing double multiplication at startup.
