//! Opt-in captures through the production app, player, HUD, sky and glow.
//! FFONE_PERF_OUTPUT=target/performance/name; FFONE_PERF_POSITION="x y z".
//! Default fixtures are offline. The explicitly isolated tutorial-network
//! fixture uses test credentials; all fixtures disable user-settings writes.
use crate::app::*;
use bevy::{
    render::{
        renderer::RenderAdapterInfo,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    time::TimeUpdateStrategy,
};
use ffone_client::transportation_ui::{
    TransportationPresentationAssetStatus, TransportationPresentationNpcCameraSlot,
    TransportationUnlocks,
};
use ffone_client::{
    legacy_model_material::static_asset_sharing_statistics,
    world::{NativeWorldPresentationStatus, NativeWorldSceneRoot, SpawnedNativeWorldVisual},
};
use std::time::{Duration, Instant};

mod ability_presentation;
mod traversal;
mod audio_regression;
mod ui_sfx_trace;
mod npc_speech;
mod player_face;
mod character_creation;
mod character_session_network;
mod coco;
mod dexter_cutscenes;
mod email_regression;
mod enchant_regression;
mod guide_nanocom;
mod inventory_availability;
mod service_dialogue;
mod pc2pc_interaction;
mod player_menu;
mod chat_commands;
mod npc_bubble_visibility;
mod journal_selection;
mod npc_interaction_range;
mod mob_animation;
mod npc_floor;
mod nano_regression;
mod nano_hud;
mod nano_attack_network;
mod npc_skill_network;
mod civilian_routes_network;
mod vendor_regression;
mod visual_effects;
mod resolution_regression;
mod skill_effects;
mod gameplay_regression;
mod tutorial_finale;
mod tutorial_combat_hud;
mod quick_chat;
mod tutorial_network;
mod quit_menu_network;
mod race_pods;
mod location_loading;

mod operations_measure;
mod operations_highlight_retrobution_transport_fixture;
mod types;
mod output;

pub(super) use operations_measure::requested;
use operations_measure::{
    mount_vehicle_fixture, show_reward_fixture, setup, open_transport_fixture, drive,
    discard_live_fixture_input, measure, open_vendor_portrait_fixture, open_retrobution_map_fixture
};
use operations_highlight_retrobution_transport_fixture::highlight_retrobution_transport_fixture;
use types::{Capture, VehicleFixture};
pub(super) use output::install;
use output::save;
