//! Exact `ActorSkinCombiner` material binding shared by preview and gameplay.
//!
//! The binder owns only per-look texture/tint decisions. Scene ownership,
//! readiness, and fallback visibility remain caller responsibilities.

use bevy::{color::LinearRgba, prelude::*};

use crate::{
    legacy_model_material::{
        LegacyModelMaterial, PendingLegacyModelMaterial,
        load_legacy_main_texture_replacement_with_contract,
    },
    player_preview::{NativePlayerLook, NativePlayerPartKind, NativePlayerPartLook},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActorSkinTextureRole {
    SecondarySkin,
    PreserveOverride,
    Primary,
}

#[derive(Clone, Debug)]
pub struct NativePlayerBoundTexture {
    pub path: String,
    pub handle: Handle<Image>,
}

/// Proof of the exact decision applied to one instance-local material.
#[derive(Clone, Debug)]
pub struct NativePlayerMaterialBinding {
    pub role: ActorSkinTextureRole,
    pub texture: Option<NativePlayerBoundTexture>,
    pub hide_surface: bool,
    pub skin_tint_applied: bool,
    pub hair_tint_applied: bool,
}

/// Apply the original `ActorSkinCombiner` branch order to one material.
///
/// A returned texture handle is still asynchronous. Callers must keep their
/// visual hidden until `AssetServer::load_state` reports `Loaded`.
pub fn bind_native_player_look_material(
    asset_server: &AssetServer,
    pending: &PendingLegacyModelMaterial,
    material: &mut LegacyModelMaterial,
    look: &NativePlayerLook,
    part: &NativePlayerPartLook,
    is_pass_companion: bool,
) -> Result<NativePlayerMaterialBinding, String> {
    let ownership_count = look
        .parts
        .iter()
        .filter(|candidate| *candidate == part)
        .count();
    if ownership_count != 1 {
        return Err(format!(
            "native player look owns exact part {:?} {ownership_count} times",
            part.exact_route
        ));
    }

    let material_name = pending.true_name.as_str();
    let role = if !uses_global_skin_secondary(part.kind)
        && !material_name.contains("main")
        && !material_name.contains("sub")
    {
        // `ActorSkinCombiner.AttachGO` only replaces rigid attachment
        // materials explicitly named `main` or `sub`. Glass lenses and other
        // auxiliary materials keep their authored Texture2D.
        ActorSkinTextureRole::PreserveOverride
    } else {
        actor_skin_texture_role(material_name)
    };
    // A few clothing skin-sub companion passes retain the authored bind-pose
    // silhouette. The preview already proved that rendering those duplicates
    // crosses the correctly animated exposed skin, so preserve the exact
    // established hide rule for every consumer of this binder.
    if is_pass_companion
        && role == ActorSkinTextureRole::SecondarySkin
        && matches!(
            part.kind,
            NativePlayerPartKind::Shirt | NativePlayerPartKind::Pants | NativePlayerPartKind::Shoes
        )
    {
        return Ok(NativePlayerMaterialBinding {
            role,
            texture: None,
            hide_surface: true,
            skin_tint_applied: false,
            hair_tint_applied: false,
        });
    }

    let mut texture = None;
    let mut skin_tint_applied = false;
    match role {
        ActorSkinTextureRole::SecondarySkin => {
            let selected_texture = if uses_global_skin_secondary(part.kind) {
                skin_tint_applied = true;
                Some(look.skin_texture.as_ref().ok_or_else(|| {
                    format!(
                        "exact body part {:?} requires the selected global skin texture",
                        part.exact_route
                    )
                })?)
            } else {
                // Rigid attachments receive the item's own secondary texture;
                // they never substitute global player skin.
                part.secondary_texture.as_ref()
            };
            if let Some(selected_texture) = selected_texture {
                texture = Some(bind_texture(
                    asset_server,
                    pending,
                    material,
                    selected_texture,
                )?);
            }
            if skin_tint_applied {
                material.uniform.base_color = half_tint(look.skin_color);
                material.uniform.emission =
                    multiply_rgb(material.uniform.emission, look.skin_color);
            }
        }
        ActorSkinTextureRole::PreserveOverride => {}
        ActorSkinTextureRole::Primary => {
            if let Some(selected_texture) = &part.primary_texture {
                texture = Some(bind_texture(
                    asset_server,
                    pending,
                    material,
                    selected_texture,
                )?);
            }
        }
    }

    // This is an independent post-branch `if` in the original combiner.
    let hair_tint_applied = material_name.contains("hair");
    if hair_tint_applied {
        material.uniform.base_color = half_tint(look.hair_color);
        material.uniform.emission = multiply_rgb(material.uniform.emission, look.hair_color);
    }
    Ok(NativePlayerMaterialBinding {
        role,
        texture,
        hide_surface: false,
        skin_tint_applied,
        hair_tint_applied,
    })
}

fn bind_texture(
    asset_server: &AssetServer,
    pending: &PendingLegacyModelMaterial,
    material: &mut LegacyModelMaterial,
    texture: &crate::player_preview::NativePlayerTexture,
) -> Result<NativePlayerBoundTexture, String> {
    let handle = load_legacy_main_texture_replacement_with_contract(
        asset_server,
        pending,
        texture.path.clone(),
        &texture.contract,
    )?;
    material.base_texture = Some(handle.clone());
    Ok(NativePlayerBoundTexture {
        path: texture.path.clone(),
        handle,
    })
}

pub(crate) fn actor_skin_texture_role(material_name: &str) -> ActorSkinTextureRole {
    if material_name.contains("sub") {
        ActorSkinTextureRole::SecondarySkin
    } else if material_name.contains("override") {
        ActorSkinTextureRole::PreserveOverride
    } else {
        ActorSkinTextureRole::Primary
    }
}

pub(crate) const fn uses_global_skin_secondary(kind: NativePlayerPartKind) -> bool {
    matches!(
        kind,
        NativePlayerPartKind::Face
            | NativePlayerPartKind::Hair
            | NativePlayerPartKind::Shirt
            | NativePlayerPartKind::Pants
            | NativePlayerPartKind::Shoes
    )
}

pub(crate) fn half_tint(color: LinearRgba) -> LinearRgba {
    LinearRgba::new(
        color.red * 0.5,
        color.green * 0.5,
        color.blue * 0.5,
        color.alpha * 0.5,
    )
}

pub(crate) fn multiply_rgb(left: LinearRgba, right: LinearRgba) -> LinearRgba {
    LinearRgba::new(
        left.red * right.red,
        left.green * right.green,
        left.blue * right.blue,
        left.alpha * right.alpha,
    )
}
