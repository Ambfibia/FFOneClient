use crate::tutorial_actors::*;
use crate::{
    character_scene::LegacyCharacterSceneDeferredReveal,
    network_world_runtime::{NetworkNpcVisualCatalog0104, NetworkNpcVisualCatalogState0104},
    tutorial::TutorialScene,
    tutorial_auxiliary_choreography::{
        TUTORIAL_AUXILIARY_DEFINITIONS, TUTORIAL_INITIALIZATION, TutorialAuxiliaryAction,
        TutorialAuxiliarySequence,
    },
    tutorial_choreography::{
        ChoreographyAction, NpcAction, NpcCommand, NpcSpawn, TUTORIAL_SCENE_CHOREOGRAPHIES,
    },
    tutorial_logic::LegacySpawnPosition,
};
use bevy::time::TimeUpdateStrategy;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

mod operations_installed_delta_ready_mob_high_layer_clips_s;
mod operations_tutorial_targeting_replaces_shared_world_tar;
mod assets;
mod animation;
mod materials;
mod entities;
mod collision;
mod models;
mod input;
mod validation;
mod commands;
mod codec;

use operations_installed_delta_ready_mob_high_layer_clips_s::{
    spawn, production_content, app, assert_vec3_close, playback_fixture, applied_fixture,
    gather_spawn_types
};
use assets::{production_visual_catalog, asset_root};
use animation::{
    gltf_with_named_animation, add_animation_requirement, choreography_animation_requirements,
    glb_animation_names
};
use input::resolve_requirement_type;
use validation::audit_npc_action;
