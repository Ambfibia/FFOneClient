use super::*;

/// GPU-side material evidence for the currently ready preview generation.
///
/// This is intentionally produced by the same binder used by selection,
/// creation, inventory, and gameplay. Batch acceptance tools can therefore
/// prove that a table texture reached a visible mesh instead of merely
/// checking that a PNG exists on disk.
#[derive(Clone, Debug, Default, Resource)]
pub struct NativePlayerPreviewAudit {
    pub identity: Option<String>,
    pub gender: Option<PlayerRigGender>,
    pub surfaces: Vec<NativePlayerPreviewSurfaceAudit>,
}

#[derive(Clone, Debug)]
pub struct NativePlayerPreviewSurfaceAudit {
    pub kind: NativePlayerPartKind,
    pub exact_route: String,
    pub material_true_name: String,
    pub role: ActorSkinTextureRole,
    pub bound_texture_path: Option<String>,
    pub base_texture_assigned: bool,
    pub source_main_texture: Option<String>,
    pub hidden: bool,
}

pub(super) fn validate_asset_path(path: &str, extension: &str) -> Result<(), String> {
    if path.is_empty()
        || path.contains('\\')
        || path.contains(':')
        || path.starts_with('/')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || !path.to_ascii_lowercase().ends_with(extension)
    {
        return Err(format!("unsafe native asset path {path:?}"));
    }
    Ok(())
}

pub(super) fn validate_exact_route(route: &str) -> Result<(), String> {
    if route.is_empty()
        || route.contains('\\')
        || route.contains(':')
        || route.starts_with('/')
        || route
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("unsafe exact player route {route:?}"));
    }
    Ok(())
}

pub(super) fn validate_palette_color(label: &str, color: LinearRgba) -> Result<(), String> {
    let channels = [color.red, color.green, color.blue, color.alpha];
    if channels
        .into_iter()
        .any(|channel| !channel.is_finite() || !(0.0..=1.0).contains(&channel))
    {
        return Err(format!(
            "native player {label} color contains a non-finite or out-of-range channel"
        ));
    }
    Ok(())
}
