# FFOne spatial pan correction

Vendored from the rodio 0.22.2 crates.io package, retaining its MIT/Apache
licenses. Cargo uses this copy for Bevy audio through the workspace patch.

The upstream `src/source/spatial.rs` pan term increases a channel's gain when
that ear is farther from the emitter. At distances where inverse-square gain
is clamped to one, this sends a left-side emitter predominantly to the right.
FFOne's spatial scale of 0.1 makes this audible near characters.

Reverse only the distance difference in the two pan modifiers. Ear positions,
per-ear inverse-square attenuation, source decoding, timing and non-spatial
stereo remain unchanged. Swapping listener ears would also swap attenuation
and therefore is not the repair.

Device-free regression tests in `source::spatial::tests` inspect actual stereo
samples for left, right, center, rotated listeners, distance and live updates.
Run `cargo test --manifest-path vendor/rodio/Cargo.toml --lib source::spatial::tests --target-dir target/spatial-regression` from the workspace.

Remove this patch when the Bevy dependency uses a backend with the corrected
pan law and the regression cases pass there.
