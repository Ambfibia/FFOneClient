use super::*;

pub const GAMEPLAY_PLAYER_PORTRAIT_TEXTURE_WIDTH: u32 = 100;

pub const GAMEPLAY_PLAYER_PORTRAIT_TEXTURE_HEIGHT: u32 = 120;

pub(super) fn remember_texture(
    textures: &mut Vec<(String, Handle<Image>)>,
    path: String,
    handle: Handle<Image>,
    apply: impl FnOnce(),
) {
    apply();
    if !textures.iter().any(|(_, existing)| existing == &handle) {
        textures.push((path, handle));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ActorSkinTextureRole {
    SecondarySkin,
    PreserveOverride,
    Primary,
}

pub(super) fn actor_skin_texture_role(material_name: &str) -> ActorSkinTextureRole {
    if material_name.contains("sub") {
        ActorSkinTextureRole::SecondarySkin
    } else if material_name.contains("override") {
        ActorSkinTextureRole::PreserveOverride
    } else {
        ActorSkinTextureRole::Primary
    }
}
