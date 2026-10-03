//! Preserve the native node namespace across Bevy glTF loader upgrades.

use bevy::{
    asset::LoadContext,
    gltf::{
        extensions::{ErasedGltfExtensionHandler, GltfExtensionHandler, GltfExtensionHandlers},
        gltf,
    },
    prelude::*,
};

pub struct NativeGltfPlugin;

impl Plugin for NativeGltfPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GltfExtensionHandlers>();
        app.world()
            .resource::<GltfExtensionHandlers>()
            .0
            .write_blocking()
            .push(Box::new(NativeSceneRoot));
    }
}

pub(crate) fn install(app: &mut App) {
    if !app.is_plugin_added::<NativeGltfPlugin>() {
        app.add_plugins(NativeGltfPlugin);
    }
}

#[derive(Clone)]
struct NativeSceneRoot;

impl GltfExtensionHandler for NativeSceneRoot {
    fn dyn_clone(&self) -> Box<dyn ErasedGltfExtensionHandler> {
        Box::new(self.clone())
    }

    fn on_scene_completed(
        &mut self,
        _load_context: &mut LoadContext<'_>,
        _scene: &gltf::Scene,
        world_root_id: Entity,
        scene_world: &mut World,
    ) {
        // Bevy 0.19 names its synthetic wrapper after the scene. Native bone
        // and behavior paths refer only to authored nodes; a scene named "m"
        // must not hide the actual "m/Bip01" hierarchy. Keep the wrapper,
        // transform, GltfSceneName, extras and every authored node unchanged.
        scene_world.entity_mut(world_root_id).remove::<Name>();
    }
}

#[cfg(test)]
mod tests;
