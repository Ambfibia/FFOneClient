//! Native XDT NPC/Nano browser for FFOneClient authoring and runtime review.
//!
//! The editor consumes only the published native table-set, semantic character
//! registry and GLB assets. It deliberately has no legacy-project or extractor
//! dependency.

#[path = "../editor/equipment.rs"]
mod equipment;
use equipment::*;
#[path = "../editor/strings.rs"]
mod strings;
#[path = "../editor/xdt.rs"]
mod xdt;
#[path = "../editor/backups.rs"]
mod backups;
#[path = "../editor/hnpc.rs"]
mod hnpc;
#[path = "../editor/icon_generator.rs"]
mod icon_generator;
#[path = "../editor/world_editor.rs"]
mod world_editor;

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::PathBuf,
};

#[cfg(test)]
use bevy::sprite::SliceScaleMode;
use bevy::{
    animation::RepeatAnimation,
    asset::AssetPlugin,
    camera::Viewport,
    gltf::{Gltf, GltfAssetLabel},
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    },
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    transform::TransformSystems,
    ui::RelativeCursorPosition,
    window::{PresentMode, WindowResolution},
};
#[cfg(test)]
use ffone_client::option_ui::OPTION_BIG_LABEL_BORDER;
use ffone_client::{
    assets::AssetLocator,
    character_scene::{
        LegacyCharacterSceneDeferredReveal, LegacyCharacterSceneStatus,
        spawn_legacy_character_scene,
    },
    coordinates::LegacyCharacterRootPolicy,
    gameplay_nano_portraits::GameplayNanoPortraitCatalog,
    hnpc_runtime::HnpcRuntimeCatalog,
    legacy_model_material::{LegacyMaterialMetadataError, LegacyModelMaterialPlugin},
    localization::{Language, Localization, LocalizationPlugin, LocalizationSet, LocalizedText},
    network_world_runtime::{
        NativeHnpcAppearancePlugin, NetworkHnpcRigAppearanceStatus0104,
        NetworkHnpcVisualDefinition0104, NetworkNpcVisualCatalog0104,
        NetworkNpcVisualDefinition0104, bind_network_npc_texture_variants_0104,
        finalize_network_npc_material_visibility_0104, spawn_network_hnpc_visual_0104,
        spawn_network_npc_visual_0104,
    },
    option_ui::OPTION_TEXT_FIELD_BORDER,
    player_shared_rig::{
        NativePlayerRigAssetCache, NativePlayerRigCatalog, NativePlayerSharedRigPlugin,
    },
    tutorial_mission_content::TutorialMissionContent,
};
use serde::Deserialize;
use serde_json::Value;

fn main() {
    let arguments = EditorArguments::parse();
    let asset_root = fs::canonicalize(&arguments.asset_root).unwrap_or_else(|error| {
        panic!(
            "cannot open FFOne asset root {}: {error}",
            arguments.asset_root.display()
        )
    });
    let locator = AssetLocator::open(&asset_root).expect("open native FFOne assets");
    let mut catalog = EditorCatalog::open(&locator).expect("build native NPC/Nano editor catalog");
    let (localization, language) =
        Localization::open(&asset_root, &arguments.language).expect("open editor localization");
    let equipment =
        EquipmentLibrary::open(&asset_root, &mut catalog).expect("open native equipment");
    let mut initial = EditorState::new(&catalog);
    strings::editing_tools::restore_tab(&asset_root, &mut initial, &catalog);
    if let Some(id) = arguments.npc {
        initial.xdt_open = false;
        initial.strings_open = false;
        initial.kind = CatalogKind::Npc;
        initial.selected = catalog
            .entries
            .iter()
            .position(|entry| entry.kind == CatalogKind::Npc && entry.network_id == Some(id))
            .expect("requested NPC exists");
    }
    if let Some(item) = arguments.equipment.as_ref() {
        initial.xdt_open = false;
        initial.strings_open = false;
        initial.kind = CatalogKind::Equipment;
        initial.selected = catalog
            .entries
            .iter()
            .position(|entry| entry.semantic_id == format!("equipment/{item}"))
            .expect("requested native equipment exists (category/id)");
    }
    initial.equipment_female = arguments.female;
    if let Some(id) = arguments.nano {
        initial.xdt_open=false; initial.strings_open=false; initial.kind=CatalogKind::Nano;
        initial.selected=catalog.entries.iter().position(|entry|entry.kind==CatalogKind::Nano && entry.network_id==Some(id)).expect("requested Nano exists");
    }
    initial.search = arguments.search.clone();
    if env::args().any(|arg| arg == "--strings") {
        initial.xdt_open = false;
        initial.strings_open = true;
    }
    if env::args().any(|arg| arg == "--xdt") {
        initial.strings_open = false;
        initial.xdt_open = true;
    }
    let rig_catalog = catalog
        .hnpc
        .as_ref()
        .expect("production HNPC catalog")
        .rig_catalog()
        .clone();

    let mut xdt_editor=xdt::XdtEditor::open(asset_root.clone()).with_selection(
        arguments.xdt_table.as_deref(),arguments.xdt_row,&arguments.search,
    ).with_graph(env::args().any(|arg|arg=="--mission-graph"));
    if env::args().any(|arg| arg == "--missions") || initial.missions_open {
        initial.xdt_open = true;
        initial.strings_open = false;
        initial.missions_open = true;
        initial.world_open = None;
        xdt_editor.open_missions();
    }
    for (flag, three_d) in [("--world-2d", false), ("--world-3d", true)] {
        if env::args().any(|arg| arg == flag) {
            initial.world_open = Some(three_d);
            initial.xdt_open = false;
            initial.strings_open = false;
        }
    }
    if env::args().any(|arg| matches!(arg.as_str(), "--npc" | "--nano" | "--equipment" | "--strings" | "--xdt" | "--hnpc-appearance" | "--icon-generator")) {
        initial.world_open = None;
        initial.missions_open = false;
    }
    let mut hnpc_editor=hnpc::HnpcEditor::default();
    if env::args().any(|arg|arg=="--hnpc-appearance") {
        hnpc_editor.open(&asset_root.join(ffone_client::assets::TABLE_SET_PATH),xdt_editor.hnpc_value().expect("select an NPC table row"),&catalog).expect("open native HNPC appearance");
        initial.xdt_open=false;initial.strings_open=false;
    }
    if env::args().any(|arg|arg=="--icon-generator") {
        initial.xdt_open=false; initial.strings_open=false;
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.008, 0.019, 0.028)))
        .insert_resource(EditorCapture {
            output: arguments.capture.clone(),
            frames: 0,
            ready_frames: 0,
            issued: false,
        })
        .insert_resource(rig_catalog)
        .insert_resource(catalog)
        .insert_resource(equipment)
        .insert_resource(initial)
        .insert_resource(xdt_editor)
        .insert_resource(hnpc_editor)
        .insert_resource(
            strings::StringEditor::open(asset_root.clone()).with_search(&arguments.search),
        )
        .insert_resource(localization)
        .insert_resource(language)
        .init_resource::<ModelPreview>()
        .init_resource::<PreparedAnimations>()
        .init_resource::<OrbitCamera>()
        .init_resource::<EditorRuntimeStatus>()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    close_when_requested: false,
                    primary_window: Some(Window {
                        visible: arguments.capture.is_none(),
                        title: "FFOneClient Editor — Missions & World".to_owned(),
                        resolution: if arguments.compact {
                            WindowResolution::new(1180, 720)
                        } else {
                            WindowResolution::new(1600, 940)
                        },
                        resize_constraints: WindowResizeConstraints {
                            min_width: 1180.0,
                            min_height: 720.0,
                            ..default()
                        },
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            LocalizationPlugin,
            NativePlayerSharedRigPlugin,
            NativeHnpcAppearancePlugin,
            ffone_client::player_preview::NativePlayerPreviewPlugin,
        ))
        .add_systems(
            Update,
            (handle_equipment_buttons, sync_equipment_look)
                .chain()
                .after(handle_editor_shortcuts)
                .before(spawn_selected_model)
                .before(ffone_client::player_preview::NativePlayerPreviewSet::Rebuild),
        )
        .add_systems(
            Update,
            sync_equipment_camera
                .after(ffone_client::player_preview::NativePlayerPreviewSet::Rebuild)
                .after(update_orbit_camera),
        )
        .add_systems(
            Update,
            (bind_section_visibility, bind_outfit_summary)
                .after(bind_editor_ui)
                .before(LocalizationSet::Apply),
        )
        .add_plugins(strings::StringsPlugin)
        .add_plugins(xdt::XdtPlugin)
        .add_plugins(hnpc::HnpcEditorPlugin)
        .add_plugins(icon_generator::IconGeneratorPlugin::new(asset_root.clone()))
        .add_plugins(world_editor::WorldEditorPlugin::new(asset_root.clone()))
        .add_systems(Startup, (setup_scene, setup_editor_ui))
        .add_systems(
            Update,
            (
                handle_editor_buttons,
                handle_search_keyboard,
                handle_editor_shortcuts,
                scroll_catalog_list,
                reveal_keyboard_selection,
                scroll_inspector,
                bind_catalog_scrollbar,
                spawn_selected_model,
                bind_network_npc_texture_variants_0104,
                finalize_network_npc_material_visibility_0104,
                prepare_animation_graph,
                sync_animation_players,
                update_runtime_status,
                capture_editor_preview,
                sync_preview_camera_viewport,
                update_orbit_camera,
                animate_turntable,
                bind_editor_ui,
                style_editor_buttons,
            )
                .chain()
                .before(LocalizationSet::Apply),
        )
        .add_systems(
            PostUpdate,
            apply_editor_t_pose.after(bevy::app::AnimationSystems).before(TransformSystems::Propagate),
        )
        .add_systems(
            PostUpdate,
            fit_loaded_model.after(TransformSystems::Propagate),
        )
        .run();
}

use ffone_client::ui_support::sliced_image;

#[cfg(test)]
mod tests;

mod animation_sync_animation_players;
mod assets_editor_catalog;
mod commands;
mod constants;
mod containers;
mod entities_spawn_catalog_panel;
mod entities_spawn_inspector_panel;
mod models;
mod operations_bind_editor_ui;
mod operations_handle_editor_buttons;
mod state;
mod systems;
mod types;

use animation_sync_animation_players::{
    ANIMATION_SLOTS, AnimationSlot, EditorAnimationApplied, EditorPoseMode, PreparedAnimationSet,
    apply_editor_t_pose, prepare_animation_graph, select_relative_clip, sync_animation_players,
};
#[cfg(test)]
use animation_sync_animation_players::{horizontal_t_pose_rotation, t_pose_child_segment};
use assets_editor_catalog::{
    CATALOG_ROW_GAP, CATALOG_ROW_HEIGHT, CatalogKind, CatalogScroll, CatalogScrollThumb,
    CatalogScrollbar, CatalogSlot, EDITOR_CATALOG_WIDTH, EditorCatalog, EditorCatalogEntry,
    bind_catalog_scrollbar, scroll_catalog_list,
};
#[cfg(test)]
use assets_editor_catalog::{CharacterRegistryDocument, catalog_scroll_thumb};
use commands::{EditorAction, NpcInspectorTab};
use constants::{
    CONSOLIDATED_TABLE, EDITOR_BODY_GAP, EDITOR_BODY_PADDING_X, EDITOR_BODY_PADDING_Y,
    EDITOR_HEADER_HEIGHT, EDITOR_INSPECTOR_WIDTH, EDITOR_PLAYBACK_HEIGHT, EDITOR_UI_CAMERA_ORDER,
    EDITOR_VIEWPORT_CONTENT_GAP, EDITOR_VIEWPORT_PADDING_X, EDITOR_VIEWPORT_PADDING_Y,
    EDITOR_VIEWPORT_TITLE_HEIGHT,
};
use containers::EditorTextBundle;
use entities_spawn_catalog_panel::{
    spawn_catalog_panel, spawn_header, spawn_selected_model, spawn_viewport_overlay,
};
use entities_spawn_inspector_panel::{
    spawn_action_button, spawn_action_button_with_role, spawn_inspector_panel,
};
use models::{CharacterRegistryModel, ModelPreview};
use operations_bind_editor_ui::{bind_editor_ui, style_editor_buttons};
use operations_handle_editor_buttons::{
    animate_turntable, array, capture_editor_preview, editor_entry_name, editor_text,
    handle_editor_buttons, handle_editor_shortcuts, handle_search_keyboard, humanize_name, integer,
    native_string, number, page_count, panel_node, preview_viewport_logical_rect, setup_editor_ui,
    setup_scene, single_line_text,
};
use state::{EditorRuntimeStatus, EditorState, reveal_keyboard_selection, update_runtime_status};
use systems::{sync_preview_camera_viewport, update_orbit_camera};
use types::{
    DynamicTextRole, EditorArguments, EditorButtonLabel, EditorButtonSkin, EditorCapture,
    EditorFonts, OrbitCamera, PreparedAnimations, PreviewCamera, PreviewRoot, TimelineFill,
    NpcInspectorTabs, NpcEditSection,
};
