use super::*;

pub const NATIVE_PLAYER_PREVIEW_RENDER_LAYER: usize = 31;

#[derive(Component, Clone, Debug)]
pub(super) struct NativePlayerPreviewMaterialBound {
    pub(super) kind: NativePlayerPartKind,
    pub(super) exact_route: String,
    pub(super) material_true_name: String,
    pub(super) binding: NativePlayerMaterialBinding,
    pub(super) base_texture_assigned: bool,
    pub(super) source_main_texture: Option<String>,
}

#[derive(Component, Clone)]
pub(super) struct NativePlayerPreviewMaterialBaseline {
    pub(super) base_color: LinearRgba,
    pub(super) emission: LinearRgba,
}

impl NativePlayerPreviewMaterialBaseline {
    pub(super) fn capture(material: &LegacyModelMaterial) -> Self {
        Self {
            base_color: material.uniform.base_color,
            emission: material.uniform.emission,
        }
    }

    pub(super) fn restore(&self, material: &mut LegacyModelMaterial) {
        // Appearance owns only these tints. Renderer/pass ordering can be
        // assigned after the first binding; restoring a whole material here
        // would erase that order and let the face overlay fight its skin.
        material.uniform.base_color = self.base_color;
        material.uniform.emission = self.emission;
    }
}

pub(super) fn bind_native_player_preview_render_layers(
    mut commands: Commands,
    runtime: Res<NativePlayerPreviewRuntime>,
    parents: Query<&ChildOf>,
    descendants: Query<Entity, (Added<ChildOf>, Without<RenderLayers>)>,
) {
    let Some(root) = runtime.root else {
        return;
    };
    for entity in &descendants {
        if is_descendant_of(entity, root, &parents) {
            commands
                .entity(entity)
                .insert(RenderLayers::layer(NATIVE_PLAYER_PREVIEW_RENDER_LAYER));
        }
    }
}
