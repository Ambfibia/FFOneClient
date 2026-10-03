
pub(super) const DEFAULT_CHARACTER_ROOT: &str = "npc_dexter";

pub(super) const HELP: &str = "\
FFOne Client - native Rust/Bevy FusionFall client

Usage:
  cargo dev [OPTIONS]
  cargo run -- [OPTIONS]

Options:
  --asset-root <DIR>       Native Bevy asset tree [default: assets/game]
  --validate-assets        Open runtime domain contracts, print routing JSON, and exit
  --network-smoke          Login, enter one character, print result, and exit
  --language <LOCALE>      UI text locale; voice remains an independent user setting
  --login-address <ADDR>   OpenFusion login address [default: 127.0.0.1:23000]
  --login-user <NAME>      Auto-login user (or set FFONE_USERNAME)
  --password-env <NAME>    Environment variable containing password [default: FFONE_PASSWORD]
  --character <UID>        Select this character automatically after login
  --character-asset-root <DIR>
                           Permanent native project-asset tree [default: assets/game]
  --character-model <GLB>  Logical character GLB relative to its asset root
                           [default: characters/npcs/npc_dexter/npc_dexter.glb]
  --character-root <NAME>  Exact original root m_Name [default: npc_dexter]
  --character-animation <NAME>
                           Exact glTF idle animation [default: stand1]
  -h, --help               Print help

The runtime opens only PNG, OGG, GLB, fonts, JSON and WGSL below native project
assets. Character selections are resolved through the generated
TableData model routes. The selected account appearance is assembled
from the shared player skeleton, body parts, clothing and equipment contracts.
It never opens Unity bundles or .ffclient projects.
";

// The native UI was reconstructed against the measured clean client surface,
// not against post-layout transforms on each individual panel. A single
// physical-to-logical window scale keeps every UI owner, clip rectangle,
// pointer and glyph rasterizer in the same coordinate system.
pub(super) const NATIVE_UI_REFERENCE_WIDTH: f32 = 1_264.0;

pub(super) const NATIVE_UI_REFERENCE_HEIGHT: f32 = 681.0;

pub(super) const LOCAL_INFECTION_EFFECT_NAME: &str = "cnOwnAvatarStatus infection ES376";

pub(super) const LOCAL_INFECTION_AURA_EFFECT_ID: i32 = 376;

pub(super) const LOCAL_INFECTION_DAMAGE_EFFECT_ID: i32 = 385;

pub(super) const LOCAL_INFECTION_PROTECTION_EFFECT_ID: i32 = 804;
