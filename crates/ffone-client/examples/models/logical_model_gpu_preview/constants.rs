use super::*;

pub(super) const HELP: &str = "\
Native logical-model GPU preview

Usage:
  cargo run -p ffone-client --example logical_model_gpu_preview -- \\
    --asset-root <DIR> --model <RELATIVE.glb> --screenshot <OUT.png> [OPTIONS]

Required:
  --asset-root <DIR>   Root of the native candidate asset tree
  --model <PATH>       GLB path relative to --asset-root (Scene0 is loaded)

Options:
  --animation <NAME|INDEX>  Play an exact glTF animation name or zero-based index
  --animation-name <NAME>   Exact animation m_Name; required by evidence for animated GLBs
  --sample-midpoint <true>  Freeze diagnostic animation at half its duration
  --character-kind <KIND>   Apply exact runtime root policy: npc, nano, or player
  --true-root <NAME>        Exact imported Unity m_Name; required with --character-kind
  --npc-scale <SCALE>       Positive NPC table m_fScale; required only for npc
  --main-texture <PNG>      XDT Texture1 path below --asset-root
  --sub-texture <PNG>       XDT Texture2 path below --asset-root
  --main-material <NAME>    Exact material true_name receiving Texture1 (default: NPC role)
  --sub-material <NAME>     Exact material true_name receiving Texture2 (default: NPC role)
  --main-sampler <JSON>     Exact source Texture1 NativeSampler metadata
  --sub-sampler <JSON>      Exact source Texture2 NativeSampler metadata
  --only-material <NAME>    Diagnostic: hide source surfaces with another exact material name
  --camera-view <VIEW>      Diagnostic camera side: primary, reverse, +/-x, +/-y, or +/-z [default: primary]
  --blank-camera-retry <MODE>  Clear-frame retry: all-axes, opposite, or same [default: all-axes]
  --report <JSON>           Write the complete diagnostic JSON report outside the asset tree
  --evidence-root <DIR>     Write immutable .gpu.json/.gpu.png evidence outside the candidate
  --runtime-smoke <JSON>    Write immutable LoadedWithDependencies/AnimationPlayer proof
  --screenshot <PNG>        Diagnostic screenshot; required without --evidence-root
  --outline <source|hidden> Keep the native outline or hide it for render diagnosis
  --frames <COUNT>          Hard frame limit [default: 900]
  --timeout <SECONDS>       Wall-clock timeout [default: 45]
  -h, --help                Print this help
";

pub(super) const DEFAULT_MAX_FRAMES: u64 = 900;

pub(super) const DEFAULT_TIMEOUT_SECS: f64 = 45.0;

// `LegacyModelMaterialApplied` proves that the CPU-side material and its exact
// textures are ready, but Bevy's render-world pipeline specialization still
// happens asynchronously. Twelve frames were not sufficient on a cold Vulkan
// cache: the additive `spwaneye` pipeline could miss the capture even though
// the corresponding Fusion face and toon pipelines had already rendered.
// Keep the acceptance screenshot behind a full 60-frame render-world warmup so
// a successful report cannot intermittently publish a Fusion without its eyes.
pub(super) const PIPELINE_WARMUP_FRAMES: u64 = 60;

pub(super) const SCREENSHOT_COLOR_TOLERANCE: i16 = 24;
