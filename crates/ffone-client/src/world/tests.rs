// Resources became entities in Bevy 0.19; unload budgets count scene entities.


use crate::coordinates::ProtocolPosition;
use crate::world::*;
use crate::{
    entity_lifecycle::{
        NetworkNpcMotion0104, NetworkTransportation0104, NetworkTransportationMotion0104,
    },
    network_world_runtime::{
        advance_network_npc_motion_0104, advance_network_transportation_motion_0104,
    },
};
use bevy::{
    asset::RenderAssetUsages, ecs::world::CommandQueue, mesh::PrimitiveTopology,
    time::TimeUpdateStrategy,
};
use ffone_protocol::NpcAppearance0104;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

mod operations_city_hall_water_occlusion_uses_production_sl;
mod operations_darktree_bridge_primary_guard_is_visible_nea;
mod operations_repeated_npc_move_packets_preserve_ground_y;
mod operations_hero_square_memorial_contact_activates_the_o;
mod operations_authored_wall_probe_blocks_vertical_faces_bu;
mod terrain;
mod assets;
mod collision_runtime_vehicle_collider_uses_current_parent;
mod collision_tutorial_dome_collision_blocks_faces_and_cor;
mod models;
mod containers;
mod animation;
mod output;
mod state;
mod materials;
mod codec;
mod types;
mod entities;
mod input;
mod validation;

use operations_city_hall_water_occlusion_uses_production_sl::{
    game_entity_count, published_character_node_world_transform, first_scene,
    remove_key_recursively
};
use operations_darktree_bridge_primary_guard_is_visible_nea::count_residency_visibility_changes;
use terrain::sanitize_runtime_terrain;
use assets::{asset_root, rewrite_registry};
use collision_runtime_vehicle_collider_uses_current_parent::{
    closed_vehicle_test_collider, authored_collider_from_glb, authored_collider_from_glb_mesh,
    authored_collider_from_glb_meshes, published_gltf_collider_from_glb,
    native_water_collider_from_glb, cooked_collider_from_glb,
    published_map_collider_for_source, authored_collider_contains_point,
    step_sized_dimple_collider
};
use models::authored_gltf_mesh_data;
use output::{write_pretty_json, write_runtime_world_fixture};
use state::sanitize_runtime_environment;
use types::{DespawnBeforeResidency, ResidencyVisibilityChanges};
use entities::despawn_before_residency;

fn init_world_presentation_assets(app: &mut App) {
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<NativeWorldStreamingStatus>();
}
