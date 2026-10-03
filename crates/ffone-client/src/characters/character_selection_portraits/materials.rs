use super::*;

pub const CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS: [usize; 4] = [27, 28, 29, 30];

#[derive(Component)]
pub(super) struct CharacterSelectionPortraitMaterialBound;

pub(super) fn sync_character_selection_portrait_render_layers(
    mut commands: Commands,
    runtime: Res<CharacterSelectionPortraitsRuntime>,
    children: Query<&Children>,
    render_layers: Query<&RenderLayers>,
    mut hierarchy_scratch: Local<Vec<Entity>>,
) {
    for (slot, runtime_slot) in runtime.slots.iter().enumerate() {
        let Some(root) = runtime_slot.root else {
            continue;
        };
        let expected = RenderLayers::layer(CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS[slot]);
        visit_portrait_hierarchy(root, &children, hierarchy_scratch.as_mut(), |entity| {
            if !matches!(render_layers.get(entity), Ok(layers) if layers == &expected) {
                commands.entity(entity).insert(expected.clone());
            }
        });
    }
}
