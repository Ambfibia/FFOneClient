//! Native Bevy runtime slices for FFOneClient.

#![forbid(unsafe_code)]

pub use characters::attachment;
pub use gameplay::audio_channel;
pub use characters::avatar_action;
pub use gameplay::bank as bank_runtime;
pub use ui::bank as bank_ui;
pub use ui::buddy as buddy_ui;
pub use ui::cashmall as cashmall_ui;
pub use characters::character_creation_data;
pub use ui::character_creation as character_creation_ui;
pub use characters::character_scene;
pub use characters::character_selection_portraits;
pub use ui::character_selection as character_selection_ui;
pub use gameplay::combi as combi_runtime;
pub use ui::combi as combi_ui;
pub mod damage_bar;
pub use gameplay::email as email_runtime;
pub use ui::email as email_ui;
pub use gameplay::enchant as enchant_runtime;
pub use ui::enchant as enchant_ui;
pub use world_systems::entity_lifecycle;
pub use ui::game_guide as game_guide_ui;
pub use gameplay::gameplay_audio;
pub use characters::gameplay_nano_portraits;
pub use ui::gameplay as gameplay_ui;
pub use gameplay::group as group_runtime;
pub use ui::group as group_ui;
pub use rendering::gui_skin;
pub use gameplay::guide as guide_runtime;
pub use ui::guide as guide_ui;
pub use characters::hnpc_runtime;
pub use ui::input_focus;
pub use gameplay::inventory as inventory_runtime;
pub use ui::launcher as launcher_ui;
pub use rendering::legacy_environment;
pub use rendering::legacy_glow;
pub use rendering::legacy_material_animation;
pub use rendering::legacy_model_material;
pub use rendering::legacy_npc_nano_animation;
pub use world_systems::legacy_world_location;
pub mod localization;
pub use ui::login as login_ui;
pub mod map_preferences;
#[cfg(test)]
mod mission_performance_tests;
pub use ui::mission as mission_ui;
pub use gameplay::movement;
pub use gameplay::nano_free_tuning as nano_free_tuning_runtime;
pub use ui::nano_free_tuning as nano_free_tuning_ui;
pub use ui::nanocom_message as nanocom_message_ui;
pub mod native_gltf;
pub use rendering::native_terrain;
pub use world_systems::network_world_runtime;
pub use ui::option as option_ui;
pub use ui::overheat as overheat_ui;
pub use ui::pc2pc as pc2pc_ui;
pub use characters::player_appearance_material;
pub use characters::player_emote;
pub use characters::player_preview;
pub use characters::player_shared_rig;
pub use ui::quick_slot as quick_slot_ui;
pub use gameplay::quit_menu as quit_menu_runtime;
pub use ui::quit_menu as quit_menu_ui;
pub use ui::race as race_ui;
pub use world_systems::remote;
pub use ui::resurrect as resurrect_ui;
pub use gameplay::rule as rule_runtime;
pub use ui::rule as rule_ui;
pub use ui::server_selection as server_selection_ui;
pub use ui::skill_buff as skill_buff_ui;
pub use ui::system_message as system_message_ui;
pub use world_systems::terrain_ambience;
pub use ui::transportation as transportation_ui;
pub use tutorial_runtime::actors as tutorial_actors;
pub use tutorial_runtime::choreography as tutorial_choreography;
pub use tutorial_runtime::choreography_formula as tutorial_choreography_formula;
pub use tutorial_runtime::choreography_runtime as tutorial_choreography_runtime;
pub use tutorial_runtime::cinematic_title as tutorial_cinematic_title;
pub use tutorial_runtime::effects_runtime as tutorial_effects_runtime;
pub use tutorial_runtime::ep_barrier as tutorial_ep_barrier;
pub use tutorial_runtime::logic as tutorial_logic;
pub use tutorial_runtime::mission_content as tutorial_mission_content;
pub use tutorial_runtime::nano_gameplay as tutorial_nano_gameplay;
pub use tutorial_runtime::nano_presentation as tutorial_nano_presentation;
pub use tutorial_runtime::nanocom_message as tutorial_nanocom_message;
pub use tutorial_runtime::native_mechanics as tutorial_native_mechanics;
/// Tutorial-only subtitles, illustrations and animated cursor presentation,
/// kept outside the persistent gameplay HUD.
pub use tutorial_runtime::overlay_ui as tutorial_overlay_ui;
pub use tutorial_runtime::player_presentation as tutorial_player_presentation;
pub use tutorial_runtime::player_rig_runtime as tutorial_player_rig_runtime;
pub use tutorial_runtime::presenter as tutorial_presenter;
pub use tutorial_runtime::voice_subtitles as tutorial_voice_subtitles;
pub use ui::icon_variants as ui_icon_variants;
pub use ui::startup as ui_startup;
pub use ui::upsell as upsell_ui;
pub use gameplay::user_equip as user_equip_runtime;
pub use ui::user_equip as user_equip_ui;
pub use gameplay::user_settings;
pub use gameplay::user_store as user_store_runtime;
pub use ui::user_store as user_store_ui;
pub use gameplay::vendor as vendor_runtime;
pub use ui::vendor as vendor_ui;
pub mod world;
pub use world_systems::audio as world_audio;
pub use world_systems::behaviour as world_behaviour;
pub use world_systems::combat as world_combat;
pub use world_systems::map as world_map;
pub use world_systems::mission_indicators as world_mission_indicators;
pub use world_systems::mission_runtime as world_mission_runtime;
pub use world_systems::nano_authority as world_nano_authority;
pub use world_systems::nano_cooldown as world_nano_cooldown;
pub use world_systems::nano_runtime as world_nano_runtime;
pub use world_systems::npc_skill_authority as world_npc_skill_authority;
pub use world_systems::targeting as world_targeting;

// Preserve the established `ffone_client::<module>` API while compiling these
// stable boundaries as independent crates.
pub use ffone_client_foundation::{assets, coordinates, semantic_audio, xdt};
pub use ffone_client_network::network;
pub use ffone_tutorial_core::{tutorial, tutorial_auxiliary_choreography};

pub use characters::service_portrait;

pub use ui::shared_input as shared_input_ui;

pub use ui::item_card;

pub(crate) use ui::scroll as service_scroll;

pub use characters::character_customization;

pub use ui::text_edit;
pub use characters::barber;


#[doc(hidden)]
pub use ui::shared as ui_support;


#[doc(hidden)]
pub use characters::scene_hierarchy;

pub mod characters;

pub mod gameplay;

pub mod ui;

pub mod world_systems;

pub mod rendering;

pub mod tutorial_runtime;
