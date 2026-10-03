use super::*;

pub(super) fn validate_relative_png_asset(value: &str, flag: &str) -> Result<(), String> {
    let path = Path::new(value);
    if value.is_empty() || path.is_absolute() {
        return Err(format!("{flag} must be a non-empty relative PNG path"));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!("{flag} may not escape --asset-root"));
    }
    let is_png = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"));
    if !is_png {
        return Err(format!("{flag} must point to a native .png file"));
    }
    Ok(())
}

pub(super) fn validate_png_path(path: &Path) -> Result<(), String> {
    let is_png = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"));
    if !is_png {
        return Err("--screenshot must end in .png".to_owned());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PreviewNpcTextureSlot {
    Main,
    Sub,
}

#[derive(Component)]
pub(super) struct PreviewNpcTextureOverrideBound {
    pub(super) slot: PreviewNpcTextureSlot,
    pub(super) texture: Handle<Image>,
}

pub(super) fn preview_npc_texture_role(
    true_name: &str,
    shader: ffone_client::legacy_model_material::LegacyShaderKind,
    has_main: bool,
    has_sub: bool,
    exact_main_material: Option<&str>,
    exact_sub_material: Option<&str>,
) -> Option<LegacyNpcTextureRole> {
    if has_main && exact_main_material == Some(true_name) {
        return Some(LegacyNpcTextureRole::Main);
    }
    if has_sub && exact_sub_material == Some(true_name) {
        return Some(LegacyNpcTextureRole::Sub);
    }
    match legacy_npc_texture_role(true_name, shader, has_main, has_sub) {
        Some(LegacyNpcTextureRole::Main) if exact_main_material.is_none() => {
            Some(LegacyNpcTextureRole::Main)
        }
        Some(LegacyNpcTextureRole::Sub) if exact_sub_material.is_none() => {
            Some(LegacyNpcTextureRole::Sub)
        }
        _ => None,
    }
}

pub(super) fn bind_preview_npc_texture_overrides(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    surfaces: Query<
        (
            Entity,
            &MeshMaterial3d<LegacyModelMaterial>,
            &PendingLegacyModelMaterial,
        ),
        (
            With<LegacyMaterialApplied>,
            Without<PreviewNpcTextureOverrideBound>,
        ),
    >,
    mut state: ResMut<RuntimeState>,
    shared: Res<SharedReport>,
    mut app_exit: MessageWriter<AppExit>,
    primitive_extras: Query<&bevy::gltf::GltfExtras>,
) {
    if state.terminal || (config.main_texture.is_none() && config.sub_texture.is_none()) {
        return;
    }
    for (entity, material_handle, pending) in &surfaces {
        if !ffone_client::legacy_model_material::native_npc_table_texture_writable(
            primitive_extras.get(entity).ok(),
        ) {
            continue;
        }
        let selected = match preview_npc_texture_role(
            &pending.true_name,
            pending.params.shader,
            config.main_texture.is_some(),
            config.sub_texture.is_some(),
            config.main_material.as_deref(),
            config.sub_material.as_deref(),
        ) {
            Some(LegacyNpcTextureRole::Sub) => config.sub_texture.as_deref().map(|path| {
                (
                    PreviewNpcTextureSlot::Sub,
                    path,
                    config.sub_sampler.as_ref(),
                )
            }),
            Some(LegacyNpcTextureRole::Main) => config.main_texture.as_deref().map(|path| {
                (
                    PreviewNpcTextureSlot::Main,
                    path,
                    config.main_sampler.as_ref(),
                )
            }),
            None => None,
        };
        let Some((slot, path, sampler)) = selected else {
            continue;
        };
        let replacement_result = if let Some(sampler) = sampler {
            load_legacy_main_texture_replacement_with_sampler(
                &asset_server,
                pending,
                path.to_owned(),
                sampler,
            )
        } else {
            load_legacy_main_texture_replacement(&asset_server, pending, path.to_owned())
        };
        let replacement = match replacement_result {
            Ok(replacement) => replacement,
            Err(error) => {
                fail_runtime(
                    &mut state,
                    &shared,
                    &mut app_exit,
                    format!(
                        "cannot apply XDT {} texture {path:?} to material {:?}: {error}",
                        match slot {
                            PreviewNpcTextureSlot::Main => "main",
                            PreviewNpcTextureSlot::Sub => "sub",
                        },
                        pending.true_name
                    ),
                );
                return;
            }
        };
        let Some(mut material) = materials.get(&material_handle.0).cloned() else {
            continue;
        };
        material.base_texture = Some(replacement.clone());
        commands.entity(entity).insert((
            MeshMaterial3d(materials.add(material)),
            PreviewNpcTextureOverrideBound {
                slot,
                texture: replacement,
            },
        ));
    }
}

#[derive(Debug)]
pub(super) struct CapturedPng {
    pub(super) bytes: Vec<u8>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) foreground_pixels: u64,
}

pub(super) fn expected_exact_mip_marker(
    bindings: &[LegacyGltfTextureBinding],
) -> Result<ExactMipChainApplied, String> {
    let mut expected = ExactMipChainApplied::default();
    for binding in bindings
        .iter()
        .filter(|binding| binding.texture_index.is_some())
    {
        expected.assigned_texture_bindings = expected
            .assigned_texture_bindings
            .checked_add(1)
            .ok_or_else(|| "assigned texture binding count exceeds u32".to_owned())?;
        let provenance = binding.mip_provenance.as_ref().ok_or_else(|| {
            format!(
                "assigned texture slot {} has no exact mip provenance",
                binding.slot
            )
        })?;
        let levels = binding.mip_levels.as_ref().ok_or_else(|| {
            format!(
                "assigned texture slot {} has no exact published mip levels",
                binding.slot
            )
        })?;
        match provenance.published_policy {
            PublishedMipPolicy::BaseLevelOnly => {
                if levels.len() != 1 {
                    return Err(format!(
                        "baseLevelOnly texture slot {} declares {} levels",
                        binding.slot,
                        levels.len()
                    ));
                }
            }
            PublishedMipPolicy::ExactSourceLevels => {
                expected.mip_chains_applied = expected
                    .mip_chains_applied
                    .checked_add(1)
                    .ok_or_else(|| "exact mip chain count exceeds u32".to_owned())?;
                expected.mip_levels = expected
                    .mip_levels
                    .checked_add(u32::try_from(levels.len()).map_err(|_| {
                        format!("texture slot {} mip level count exceeds u32", binding.slot)
                    })?)
                    .ok_or_else(|| "exact mip level count exceeds u32".to_owned())?;
            }
        }
    }
    Ok(expected)
}

pub(super) fn linear_rgba_to_srgb8(color: LinearRgba) -> [u8; 3] {
    let color = Srgba::from(color);
    [color.red, color.green, color.blue]
        .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
}

pub(super) fn texture_override_status(requested: Option<&str>, matching_materials: u64) -> &'static str {
    match (requested.is_some(), matching_materials > 0) {
        (false, _) => "not_requested",
        (true, false) => "no_matching_material_unity_noop",
        (true, true) => "bound",
    }
}
