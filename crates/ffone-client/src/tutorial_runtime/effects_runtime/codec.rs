
// A dense streamed tile can publish hundreds of EPElementController entries.
// Compiling all of their ES closures in the frame where behaviour JSON becomes
// ready moves the streaming hitch from JSON parsing to the effect runtime.
// Keep gameplay commands immediate and admit only a small ambient slice after
// them on each Update.
pub(super) const STREAMED_WORLD_EFFECT_COMMANDS_PER_FRAME: usize = 8;
