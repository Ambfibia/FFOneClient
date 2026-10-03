
pub const SEMANTIC_ROUNDTRIP_PROOF_SCHEMA: &str = "ffone.semantic-roundtrip-proof.v1";

pub(super) const COVERED_SCOPES: [&str; 6] = [
    "hierarchy and coordinates: exact native coordinate contract, logical/node names and order, roots, parents, TRS, mesh/skin bindings",
    "geometry: exact mesh/primitive order, positions, normals, UV0, indices, JOINTS_0 and WEIGHTS_0",
    "skins: exact skin names/order, skeleton roots, joint palettes and inverse bind matrices",
    "standard animations: exact clip/channel order and names, clip settings, targets, interpolation, times, values, in/out tangents and legacy tangent modes",
    "animation metadata: exact empty, identical duplicate, and serialized-last-write TRS bindings, identical same-time keys, typed non-strict-time recoveries and sibling-proof provenance, float curves, object-reference curves, events, unsupported-binding list and metadata-only clips, including all source curve/key indices, encodings, paths, original/recovered times, sample rates, tangent and object-reference values",
    "materials: exact native material records, ordered ShaderLab texture defaults, and texture-binding transform presence, including absent versus explicit-zero pivot and rotation",
];

pub(super) const EXCLUDED_SCOPES: [&str; 1] =
    ["GPU rendering and runtime playback parity require the separate Bevy acceptance gate"];
