use ffone_skinned_model::NativeCoordinateContract;
use serde::{Deserialize, Serialize};

use crate::CharacterAppearanceCategory;

pub const PLAYER_SHARED_RIG_SCHEMA: &str = "ffone.player-shared-rig.v3";
pub const PLAYER_SHARED_RIG_CONTRACT_PATH: &str =
    "characters/player/shared/player_rig_contract.json";
pub const MALE_SHARED_SKELETON_GLB_PATH: &str = "characters/player/male/base/male_skeleton.glb";
pub const FEMALE_SHARED_SKELETON_GLB_PATH: &str =
    "characters/player/female/base/female_skeleton.glb";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerRigGender {
    Male,
    Female,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerRigSourceIdentity {
    pub object_dump: String,
    pub object_dump_bytes: u64,
    pub object_dump_blake3: String,
    pub appearance: String,
    pub appearance_blake3: String,
    pub avatar_items: String,
    pub avatar_items_blake3: String,
    pub source_build: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerRigNode {
    pub actor_bone_index: u32,
    pub source_transform_path_id: i64,
    pub source_game_object_path_id: i64,
    pub true_name: String,
    pub full_path: String,
    pub parent_actor_bone_index: Option<u32>,
    pub translation: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerRigClipContract {
    pub name: String,
    pub source_path_id: i64,
    pub gltf_animation_index: u32,
    pub channel_count: u32,
    pub source_key_count: u64,
    pub duration_seconds_bits: u64,
    pub playback: String,
    pub runtime_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerRigSkinRemap {
    pub renderer_true_name: String,
    pub renderer_path_id: i64,
    pub actor_wear_index_table_path_id: i64,
    pub actor_bone_indices: Vec<u32>,
    pub actor_bone_paths: Vec<String>,
    pub gltf_joint_paths: Vec<String>,
    pub exact_transform_index_parity: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerRigPartContract {
    pub exact_route: String,
    pub true_name: String,
    pub glb: String,
    pub actor_skin_combiner_clothes_index: u8,
    pub skins: Vec<PlayerRigSkinRemap>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerRigCreatorChoiceContract {
    pub appearance_category: CharacterAppearanceCategory,
    pub creation_index: u16,
    pub item_number: u32,
    pub exact_route: String,
    pub glb: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerGenderRigContract {
    pub gender: PlayerRigGender,
    pub exact_actor_route: String,
    pub actor_root_path_id: i64,
    pub actor_skin_combiner_path_id: i64,
    pub animation_component_path_id: i64,
    pub skeleton_glb: String,
    pub nodes: Vec<PlayerRigNode>,
    pub clips: Vec<PlayerRigClipContract>,
    pub creator_parts: Vec<PlayerRigPartContract>,
    pub default_creator_part_routes: Vec<String>,
    pub creator_choices: Vec<PlayerRigCreatorChoiceContract>,
    pub stand1_runtime_ready: bool,
    pub height_shape_assets_published: bool,
    pub height_shape_runtime_status: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerSharedRigContract {
    pub schema: String,
    pub status: String,
    pub source: PlayerRigSourceIdentity,
    pub native_coordinate_contract: NativeCoordinateContract,
    pub creator_preview_ready: bool,
    pub fake_animation_used: bool,
    pub unity_runtime_required: bool,
    pub genders: Vec<PlayerGenderRigContract>,
}
