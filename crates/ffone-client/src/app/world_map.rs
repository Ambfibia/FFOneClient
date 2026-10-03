//! World map production runtime, open blockers, input and projection.

use super::LocalPlayer;
use super::local_inventory::LocalInventoryRuntime;
use super::mission_indicators::WorldMissionWaypointRuntime;
use super::modal_gates::GameplayModalModels;
use super::option_runtime::{
    OptionProductionRuntime, clear_option_action_press, option_action_just_pressed,
};
use super::race::warp_away_xcom_index;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use bevy::{
    ecs::system::SystemParam,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    window::PrimaryWindow,
};
use ffone_client::{
    coordinates::{ProtocolPosition, native_to_unity_vector},
    entity_lifecycle::NetworkNpc0104,
    game_guide_ui::GameGuideUiModel,
    gameplay_audio::GameplayAudioRuntime,
    gameplay_ui::GameplayUiModel,
    guide_runtime::GuideRuntime,
    guide_ui::GuideUiOutbox,
    legacy_world_location::legacy_world_location_name_or_last,
    mission_ui::MissionUiModel,
    movement::{LegacyPlayerController, LegacyWorldColliderPending},
    nano_free_tuning_runtime::NanoFreeTuningBank0104,
    network::{NetworkBridge, NetworkCommand},
    option_ui::LegacyOptionAction,
    quit_menu_runtime::QuitMenuRuntime,
    race_ui::rank::RaceRankCatalog,
    skill_buff_ui::SkillBuffUiModel,
    transportation_ui::TransportationCatalog,
    tutorial_mission_content::TutorialMissionContent,
    upsell_ui::UpsellUiOutbox,
    vendor_ui::VendorUiOutbox0104,
    world_map::{
        WorldMapCatalog, WorldMapCatalogLookup, WorldMapCloseInput, WorldMapInputGates,
        WorldMapMissionAvailability, WorldMapNpcCatalogEntry, WorldMapNpcSource,
        WorldMapOpenContext, WorldMapOutboxEvent, WorldMapPanDirection, WorldMapPhase,
        WorldMapPlayer, WorldMapPoint, WorldMapPresentation, WorldMapPresentationAssetStatus,
        WorldMapPresentationControl, WorldMapUiPoint, WorldMapZoomDirection, world_map_marker_at,
        world_map_presentation_layout_with_scale,
    },
    world_mission_indicators::{ClientNpcWaypointCatalog, npc_class_supports_world_quest_symbols},
    world_mission_runtime::WorldMissionRuntime,
};
use ffone_protocol::{PresentNpcTypesReply0104, PresentNpcTypesRequest0104};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Resource)]
pub(super) struct WorldMapProductionRuntime {
    pub(super) pressed_control: Option<WorldMapPresentationControl>,
    pub(super) pan_audio_active: bool,
    pub(super) last_projection_error: Option<String>,
    /// Clean `WorldMapMode` iterates the global, ordered
    /// `clientnpc.asset` array rather than the currently streamed NPC set.
    /// Cache its projection once per gameplay session instead of rebuilding
    /// all 2,903 entries on every open-map frame.
    pub(super) npc_sources: Vec<WorldMapNpcSource>,
    pub(super) server_npcs: ffone_protocol::npc_map::NpcMapSnapshot,
    pub(super) snapshot_retry_seconds: f32,
    pub(super) observed_npcs: BTreeMap<i32, WorldMapNpcSource>,
    pub(super) combined_npc_sources: Vec<WorldMapNpcSource>,
    pub(super) npc_sources_dirty: bool,
}

/// The exact last server time independently observed on the shard.
///
/// Retrobution overwrites `GameFrame.uiServerTime` on `PC_ENTER_SUCC` and
/// movement broadcasts. It does not extrapolate that value from wall time, and
/// the present-NPC-types reply itself carries no timestamp.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub(super) struct WorldMapServerClock {
    pub(super) last_observed_server_time: Option<u64>,
}

impl WorldMapServerClock {
    pub(super) fn observe(&mut self, server_time: u64) {
        self.last_observed_server_time = Some(server_time);
    }

    pub(super) fn reset(&mut self) {
        self.last_observed_server_time = None;
    }
}

pub(super) fn apply_world_map_present_reply(
    presentation: &mut WorldMapPresentation,
    clock: WorldMapServerClock,
    reply: &PresentNpcTypesReply0104,
) -> Result<(), String> {
    let server_time = clock.last_observed_server_time.ok_or_else(|| {
        "present-NPC-types reply arrived before an independently observed server time".to_owned()
    })?;
    presentation
        .model
        .apply_present_npc_types(reply.clears_existing(), server_time, &reply.npc_types)
        .map_err(|error| format!("{error:?}"))
}

pub(super) fn world_map_player_from_native(
    transform: &Transform,
    controller: &LegacyPlayerController,
) -> WorldMapPlayer {
    let unity = native_to_unity_vector(transform.translation);
    WorldMapPlayer::new(
        WorldMapPoint::new(unity.x, unity.y, unity.z),
        controller.yaw_degrees,
    )
}

pub(super) fn world_map_npc_sources(catalog: &ClientNpcWaypointCatalog) -> Vec<WorldMapNpcSource> {
    catalog
        .rows()
        .iter()
        .map(|row| {
            let [x, y, z] = row.client_position;
            WorldMapNpcSource {
                npc_type: row.npc_type,
                position: WorldMapPoint::new(x, y, z),
            }
        })
        .collect()
}

pub(super) fn open_world_map_for_ready_player(
    presentation: &mut WorldMapPresentation,
    asset_status: &WorldMapPresentationAssetStatus,
    player: Option<WorldMapPlayer>,
    map_number: Option<i32>,
    race_catalog: &RaceRankCatalog,
) -> Result<(), String> {
    let episode_id = world_map_episode_id(map_number, race_catalog)
        .ok_or_else(|| "WorldMap blocked in this authoritative instance".to_owned())?;
    match asset_status {
        WorldMapPresentationAssetStatus::Ready => {}
        WorldMapPresentationAssetStatus::Loading => {
            return Err(
                "WorldMap blocked while its exact native assets are still loading".to_owned(),
            );
        }
        WorldMapPresentationAssetStatus::Failed { asset_path } => {
            return Err(format!(
                "WorldMap blocked by missing exact asset {asset_path}"
            ));
        }
    }
    let player = player.ok_or_else(|| {
        "WorldMap blocked before the authoritative local player is ready".to_owned()
    })?;
    let mut context = WorldMapOpenContext::gameplay(player);
    context.instance_map = map_number != Some(0);
    context.episode_id = episode_id;
    presentation
        .model
        .try_open(context)
        .map(|_| ())
        .map_err(|error| format!("WorldMap open rejected: {error:?}"))
}

fn world_map_episode_id(map_number: Option<i32>, race_catalog: &RaceRankCatalog) -> Option<i32> {
    // The shard owns map_number; the bundled InstanceTable-derived catalog
    // identifies EP maps. Unknown maps and non-EP instances have no WorldMap.
    match map_number? {
        0 => Some(0),
        map => race_catalog
            .locations()
            .iter()
            .find(|location| location.instance_name_id == map)
            .map(|location| location.ep_id),
    }
}

fn world_map_npc_visible(npc_class: Option<i32>, sound: Option<i32>, radar: bool) -> bool {
    radar || npc_class != Some(0) || sound == Some(2)
}

#[cfg(test)]
mod map_access_tests {
    use super::*;

    #[test]
    fn ui_sfx_map_zoom_boundaries_and_modal_controls_follow_accepted_input() {
        use ffone_client::gameplay_ui::{GameplayUiAudioCue, GameplayUiAudioOutbox};
        let mut map = WorldMapPresentation::default();
        open_world_map_for_ready_player(
            &mut map,
            &WorldMapPresentationAssetStatus::Ready,
            Some(WorldMapPlayer::new(WorldMapPoint::new(4096.0, 0.0, 4096.0), 0.0)),
            Some(0),
            &RaceRankCatalog::default(),
        ).unwrap();
        let world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let mut runtime = RuntimeStatus::default();
        let mut audio = GameplayUiAudioOutbox::default();
        let gates = WorldMapInputGates::default();
        let blocked = WorldMapInputGates { system_popup_open: true, ..gates };
        for control in [WorldMapPresentationControl::ZoomIn, WorldMapPresentationControl::Filter(0), WorldMapPresentationControl::Help, WorldMapPresentationControl::Close] {
            apply_world_map_control(&mut commands, control, &mut map, blocked, &mut runtime, &mut audio);
        }
        assert_eq!(audio.drain().count(), 0);
        assert_eq!(map.model.phase(), WorldMapPhase::Open);
        apply_world_map_control(&mut commands, WorldMapPresentationControl::WorldView, &mut map, gates, &mut runtime, &mut audio);
        let _ = audio.drain().count();
        apply_world_map_control(&mut commands, WorldMapPresentationControl::ZoomIn, &mut map, gates, &mut runtime, &mut audio);
        assert_eq!(audio.drain().collect::<Vec<_>>(), [GameplayUiAudioCue::ButtonSound, GameplayUiAudioCue::HeightUp]);
        for _ in 0..8 {
            apply_world_map_control(&mut commands, WorldMapPresentationControl::ZoomIn, &mut map, gates, &mut runtime, &mut audio);
        }
        assert_eq!(audio.drain().last(), Some(GameplayUiAudioCue::NoButton));
        apply_world_map_control(&mut commands, WorldMapPresentationControl::ZoomOut, &mut map, gates, &mut runtime, &mut audio);
        assert_eq!(audio.drain().collect::<Vec<_>>(), [GameplayUiAudioCue::ButtonSound, GameplayUiAudioCue::HeightDown]);
        apply_world_map_control(&mut commands, WorldMapPresentationControl::Close, &mut map, gates, &mut runtime, &mut audio);
        assert_eq!(audio.drain().collect::<Vec<_>>(), [GameplayUiAudioCue::ButtonSound]);
        assert_eq!(map.model.phase(), WorldMapPhase::Closed);
        apply_world_map_control(&mut commands, WorldMapPresentationControl::Close, &mut map, gates, &mut runtime, &mut audio);
        assert_eq!(audio.drain().count(), 0);
    }

    #[test]
    fn world_map_uses_authoritative_map_number_for_every_open_route() {
        let player = Some(WorldMapPlayer::new(
            WorldMapPoint::new(6320.0, 0.0, 1871.0),
            0.0,
        ));
        let mut map = WorldMapPresentation::default();
        let assets = WorldMapPresentationAssetStatus::Ready;
        let catalog = RaceRankCatalog::default();
        assert!(
            open_world_map_for_ready_player(&mut map, &assets, player, None, &catalog).is_err()
        );
        assert!(
            open_world_map_for_ready_player(&mut map, &assets, player, Some(1), &catalog).is_err()
        );
        assert_eq!(map.model.phase(), WorldMapPhase::Closed);
        assert!(
            open_world_map_for_ready_player(&mut map, &assets, player, Some(0), &catalog).is_ok()
        );
        assert_eq!(map.model.phase(), WorldMapPhase::Open);
        map.reset_session();
        assert!(
            open_world_map_for_ready_player(&mut map, &assets, player, Some(15), &catalog).is_ok()
        );
        assert_eq!(map.model.phase(), WorldMapPhase::Open);
    }

    #[test]
    fn world_map_radar_only_reveals_ordinary_mobs() {
        assert!(!world_map_npc_visible(Some(0), Some(0), false));
        assert!(world_map_npc_visible(Some(0), Some(0), true));
        assert!(world_map_npc_visible(Some(0), Some(2), false));
        assert!(world_map_npc_visible(Some(1), Some(0), false));
    }
}

pub(super) fn reset_world_map_shell(
    presentation: &mut WorldMapPresentation,
    production: &mut WorldMapProductionRuntime,
    clock: &mut WorldMapServerClock,
) {
    presentation.reset_session();
    *production = WorldMapProductionRuntime::default();
    clock.reset();
}

pub(super) fn reset_world_map_session(
    mut presentation: ResMut<WorldMapPresentation>,
    mut production: ResMut<WorldMapProductionRuntime>,
    mut clock: ResMut<WorldMapServerClock>,
) {
    reset_world_map_shell(&mut presentation, &mut production, &mut clock);
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(super) struct WorldMapOpenBlockers {
    pub(super) chat: bool,
    pub(super) buddy: bool,
    pub(super) mission: bool,
    pub(super) system_popup: bool,
    pub(super) nanocom: bool,
    pub(super) quit: bool,
    pub(super) option: bool,
    pub(super) resurrect: bool,
    pub(super) upsell: bool,
    pub(super) guide: bool,
    pub(super) bank: bool,
    pub(super) vendor: bool,
    pub(super) rule: bool,
    pub(super) nano_free_tuning: bool,
    pub(super) user_equip: bool,
    pub(super) transportation: bool,
    pub(super) race: bool,
    pub(super) email: bool,
    pub(super) combi: bool,
    pub(super) enchant: bool,
    pub(super) cashmall: bool,
    pub(super) user_store: bool,
}

#[derive(SystemParam)]
pub(super) struct WorldMapInputResources<'w> {
    pub(super) mission_ui: Res<'w, MissionUiModel>,
    pub(super) guide_outbox: Res<'w, GuideUiOutbox>,
    pub(super) vendor_outbox: Res<'w, VendorUiOutbox0104>,
    pub(super) upsell_outbox: Res<'w, UpsellUiOutbox>,
    pub(super) quit_runtime: Res<'w, QuitMenuRuntime>,
    pub(super) asset_status: Res<'w, WorldMapPresentationAssetStatus>,
    pub(super) race_catalog: Res<'w, RaceRankCatalog>,
}

#[derive(SystemParam)]
pub(super) struct WorldMapProjectionAuthority<'w> {
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) race_catalog: Res<'w, RaceRankCatalog>,
    pub(super) skill_buffs: Res<'w, SkillBuffUiModel>,
}

impl WorldMapOpenBlockers {
    #[must_use]
    pub(super) const fn blocked(self) -> bool {
        self.chat
            || self.buddy
            || self.mission
            || self.system_popup
            || self.nanocom
            || self.quit
            || self.option
            || self.resurrect
            || self.upsell
            || self.guide
            || self.bank
            || self.vendor
            || self.rule
            || self.nano_free_tuning
            || self.user_equip
            || self.transportation
            || self.race
            || self.email
            || self.combi
            || self.enchant
            || self.cashmall
            || self.user_store
    }
}

pub(super) fn world_map_window_pointer(window: &Window, ui_scale: f32) -> Option<WorldMapUiPoint> {
    let cursor = window.cursor_position()?;
    let layout =
        world_map_presentation_layout_with_scale(window.width(), window.height(), ui_scale)?;
    Some(
        layout
            .window
            .screen_to_local(WorldMapUiPoint::new(cursor.x, cursor.y)),
    )
}

pub(super) fn apply_world_map_control(
    commands: &mut Commands,
    control: WorldMapPresentationControl,
    presentation: &mut WorldMapPresentation,
    gates: WorldMapInputGates,
    runtime: &mut RuntimeStatus,
    audio: &mut ffone_client::gameplay_ui::GameplayUiAudioOutbox,
) {
    use ffone_client::gameplay_ui::GameplayUiAudioCue;
    use ffone_client::world_map::WorldMapInputResult;
    let result = match control {
        WorldMapPresentationControl::Close => {
            if presentation
                .model
                .try_close(WorldMapCloseInput::CloseButton, gates)
            {
                audio.push(GameplayUiAudioCue::ButtonSound);
            }
            None
        }
        WorldMapPresentationControl::Help => {
            if !gates.system_popup_open {
                audio.push(GameplayUiAudioCue::ButtonSound);
                commands.queue(|world: &mut World| {
                    if world.resource_mut::<GameGuideUiModel>().open_first_use(15)
                        && let Some(mut audio) = world.get_resource_mut::<GameplayAudioRuntime>()
                    {
                        audio.queue_gameplay_ui_sound("Open_Screen");
                    }
                });
            }
            None
        }
        WorldMapPresentationControl::Up
        | WorldMapPresentationControl::Down
        | WorldMapPresentationControl::Left
        | WorldMapPresentationControl::Right => None,
        WorldMapPresentationControl::ZoomIn => Some(
            presentation
                .model
                .zoom_button(WorldMapZoomDirection::In, gates),
        ),
        WorldMapPresentationControl::ZoomOut => Some(
            presentation
                .model
                .zoom_button(WorldMapZoomDirection::Out, gates),
        ),
        WorldMapPresentationControl::ZoomTick(index) => Some(
            presentation
                .model
                .select_zoom_tick(usize::from(index), gates),
        ),
        WorldMapPresentationControl::LocalView => Some(presentation.model.select_local_view(gates)),
        WorldMapPresentationControl::WorldView => Some(presentation.model.select_world_view(gates)),
        WorldMapPresentationControl::Filter(index) => {
            if presentation.model.toggle_filter(usize::from(index), gates) == WorldMapInputResult::Changed {
                audio.push(GameplayUiAudioCue::ButtonSound);
            }
            None
        }
        WorldMapPresentationControl::ShowFilters => {
            if presentation.model.toggle_filters(gates) == WorldMapInputResult::Changed {
                audio.push(GameplayUiAudioCue::ButtonSound);
            }
            None
        }
    };
    let direction = match control {
        WorldMapPresentationControl::ZoomIn => Some(WorldMapZoomDirection::In),
        WorldMapPresentationControl::ZoomOut => Some(WorldMapZoomDirection::Out),
        _ => None,
    };
    if let Some(Ok(result)) = result.as_ref() {
        if let Some(direction) = direction {
            if *result != WorldMapInputResult::Ignored {
                // GUI.Button sounds before FuncZoomIn/Out adds its cue.
                audio.push(GameplayUiAudioCue::ButtonSound);
                queue_world_map_zoom_audio(audio, direction, *result);
            }
        } else if *result == WorldMapInputResult::Changed {
            audio.push(GameplayUiAudioCue::ButtonSound);
        }
    }
    if let Some(Err(error)) = result {
        runtime.message = format!("WorldMap control {control:?} rejected: {error:?}");
    }
}

fn queue_world_map_zoom_audio(
    audio: &mut ffone_client::gameplay_ui::GameplayUiAudioOutbox,
    direction: WorldMapZoomDirection,
    result: ffone_client::world_map::WorldMapInputResult,
) {
    use ffone_client::{gameplay_ui::GameplayUiAudioCue, world_map::WorldMapInputResult};
    let cue = match (result, direction) {
        (WorldMapInputResult::Changed, WorldMapZoomDirection::In) => GameplayUiAudioCue::HeightUp,
        (WorldMapInputResult::Changed, WorldMapZoomDirection::Out) => GameplayUiAudioCue::HeightDown,
        (WorldMapInputResult::Boundary, _) => GameplayUiAudioCue::NoButton,
        (WorldMapInputResult::Ignored, _) => return,
    };
    audio.push(cue);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_world_map_input(
    mut commands: Commands,
    state: Res<State<ClientState>>,
    time: Res<Time>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut mouse_buttons: ResMut<ButtonInput<MouseButton>>,
    option_runtime: Res<OptionProductionRuntime>,
    mouse_motion: Option<Res<AccumulatedMouseMotion>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    gameplay_ui: Res<GameplayUiModel>,
    inputs: WorldMapInputResources,
    mut map_params: ParamSet<(GameplayModalModels, ResMut<WorldMapPresentation>)>,
    mut production: ResMut<WorldMapProductionRuntime>,
    players: Query<
        (&Transform, &LegacyPlayerController),
        (With<LocalPlayer>, Without<LegacyWorldColliderPending>),
    >,
    mut runtime: ResMut<RuntimeStatus>,
    mut audio: ResMut<ffone_client::gameplay_ui::GameplayUiAudioOutbox>,
) {
    production.pan_audio_active = false;
    let WorldMapInputResources {
        mission_ui,
        guide_outbox,
        vendor_outbox,
        upsell_outbox,
        quit_runtime,
        asset_status,
        race_catalog,
    } = inputs;
    let (blockers, system_popup_open, help_open) = {
        let modals = map_params.p0();
        (
            WorldMapOpenBlockers {
                chat: modals.chat_active(*state.get()),
                buddy: modals.buddy_modal(),
                mission: mission_ui.gameplay_input_blocked(),
                system_popup: modals.system_popup()
                    || modals.pc2pc_modal()
                    || mission_ui.system_popup_active(),
                nanocom: modals.nanocom_popup(),
                quit: modals.quit_modal() || quit_runtime.is_waiting_for_server(),
                option: modals.option_modal(),
                resurrect: modals.resurrect_modal(),
                upsell: modals.upsell_modal() || !upsell_outbox.is_empty(),
                guide: modals.guide_modal() || !guide_outbox.is_empty(),
                bank: modals.bank_modal(),
                vendor: modals.vendor_modal() || !vendor_outbox.is_empty(),
                rule: modals.rule_modal(),
                nano_free_tuning: modals.nano_free_tuning_modal(),
                user_equip: modals.user_equip_modal(),
                transportation: modals.transportation_world_owned(),
                race: modals.race_modal(),
                email: modals.email_modal(),
                combi: (modals.combi_modal() || modals.barber_modal()),
                enchant: modals.enchant_modal(),
                cashmall: modals.cashmall_modal(),
                user_store: modals.user_store_modal(),
            },
            modals.system_popup() || mission_ui.system_popup_active(),
            modals.game_guide_modal(),
        )
    };
    if help_open {
        return;
    }
    let player = players
        .single()
        .ok()
        .map(|(transform, controller)| world_map_player_from_native(transform, controller));
    let pointer = windows
        .single()
        .ok()
        .and_then(|window| world_map_window_pointer(window, gameplay_ui.ui_scale));
    let mut presentation = map_params.p1();
    presentation.set_ui_scale(gameplay_ui.ui_scale);
    if let Some(pointer) = pointer {
        presentation.hover.pointer = pointer;
        presentation.hover.control = presentation.model.control_at(pointer);
        presentation.hover.marker_index = world_map_marker_at(&presentation.markers, pointer);
    } else {
        presentation.hover.control = None;
        presentation.hover.marker_index = None;
    }

    if *state.get() != ClientState::World {
        production.pressed_control = None;
        return;
    }

    if presentation.model.phase() == WorldMapPhase::Closed {
        if option_action_just_pressed(
            &option_runtime.input,
            LegacyOptionAction::WorldMap,
            &keyboard,
            &mouse_buttons,
        ) && !blockers.blocked()
        {
            match open_world_map_for_ready_player(
                &mut presentation,
                &asset_status,
                player,
                runtime.map_number,
                &race_catalog,
            ) {
                Ok(()) => {
                    clear_option_action_press(
                        &option_runtime.input,
                        LegacyOptionAction::WorldMap,
                        &mut keyboard,
                        &mut mouse_buttons,
                    );
                }
                Err(message) => {
                    runtime.message = message;
                }
            }
        }
        return;
    }

    let gates = WorldMapInputGates {
        system_popup_open,
        // While WorldMap is the active native mode, its local Escape owner is
        // the source-equivalent successful `(2, 24)` return path.
        escape_close_allowed: true,
    };
    if option_action_just_pressed(
        &option_runtime.input,
        LegacyOptionAction::WorldMap,
        &keyboard,
        &mouse_buttons,
    ) && presentation
        .model
        .try_close(WorldMapCloseInput::MapKey22, gates)
    {
        clear_option_action_press(
            &option_runtime.input,
            LegacyOptionAction::WorldMap,
            &mut keyboard,
            &mut mouse_buttons,
        );
        // The active WorldMap owns the whole input frame; a simultaneous
        // Escape must not fall through and open QuitMenu after action 22
        // commits the close.
        keyboard.clear_just_pressed(KeyCode::Escape);
        clear_option_action_press(
            &option_runtime.input,
            LegacyOptionAction::Escape,
            &mut keyboard,
            &mut mouse_buttons,
        );
        production.pressed_control = None;
        return;
    }
    if (keyboard.just_pressed(KeyCode::Escape)
        || option_action_just_pressed(
            &option_runtime.input,
            LegacyOptionAction::Escape,
            &keyboard,
            &mouse_buttons,
        ))
        && presentation
            .model
            .try_close(WorldMapCloseInput::Escape, gates)
    {
        keyboard.clear_just_pressed(KeyCode::Escape);
        clear_option_action_press(
            &option_runtime.input,
            LegacyOptionAction::Escape,
            &mut keyboard,
            &mut mouse_buttons,
        );
        production.pressed_control = None;
        return;
    }
    if presentation.model.phase() != WorldMapPhase::Open {
        return;
    }

    if let Some(pointer) = pointer
        && let Some(scroll) = mouse_scroll.as_ref()
    {
        let direction = if scroll.delta.y > 0.0 {
            Some(WorldMapZoomDirection::In)
        } else if scroll.delta.y < 0.0 {
            Some(WorldMapZoomDirection::Out)
        } else {
            None
        };
        if let Some(direction) = direction {
            match presentation.model.zoom_wheel_at(pointer, direction, gates) {
                Ok(result) => queue_world_map_zoom_audio(&mut audio, direction, result),
                Err(error) => runtime.message = format!("WorldMap wheel zoom rejected: {error:?}"),
            }
        }
    }

    if mouse_buttons.just_pressed(MouseButton::Right)
        && !gates.system_popup_open
        && presentation.hover.control.is_none()
    {
        let selected = presentation
            .hover
            .marker_index
            .and_then(|index| presentation.markers.get(index))
            .and_then(|marker| match marker.kind {
                ffone_client::world_map::WorldMapMarkerKind::CustomWaypoint { color, .. } => {
                    Some(color)
                }
                _ => None,
            });
        if let Some(color) = selected {
            if presentation.model.remove_custom_waypoint(color, gates) {
                audio.push(ffone_client::gameplay_ui::GameplayUiAudioCue::ScrollDown);
            }
        } else if let Some(pointer) = pointer {
            if presentation.model.map_draw_rect().contains(pointer) {
                let added = presentation.model.place_custom_waypoint(pointer, gates);
                audio.push(if added {
                    ffone_client::gameplay_ui::GameplayUiAudioCue::SelectColor
                } else {
                    ffone_client::gameplay_ui::GameplayUiAudioCue::ActionFailure
                });
            }
        }
    }
    if mouse_buttons.just_pressed(MouseButton::Left) {
        production.pressed_control = presentation.hover.control;
        if production.pressed_control.is_none() && let Some(pointer) = pointer {
            match presentation.model.begin_drag(pointer, gates) {
                Ok(ffone_client::world_map::WorldMapInputResult::Changed) =>
                    audio.push(ffone_client::gameplay_ui::GameplayUiAudioCue::ButtonSound),
                Err(error) => runtime.message = format!("WorldMap drag start rejected: {error:?}"),
                _ => {}
            }
        }
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        if presentation.model.is_dragging()
            && let Some(motion) = mouse_motion.as_ref()
            && let Err(error) = presentation.model.drag_by(
                motion.delta.x / gameplay_ui.ui_scale,
                motion.delta.y / gameplay_ui.ui_scale,
                time.delta_secs(),
                gates,
            )
        {
            runtime.message = format!("WorldMap drag rejected: {error:?}");
        }
        let repeat = production
            .pressed_control
            .filter(|control| Some(*control) == presentation.hover.control)
            .and_then(|control| match control {
                WorldMapPresentationControl::Up => Some(WorldMapPanDirection::Up),
                WorldMapPresentationControl::Down => Some(WorldMapPanDirection::Down),
                WorldMapPresentationControl::Left => Some(WorldMapPanDirection::Left),
                WorldMapPresentationControl::Right => Some(WorldMapPanDirection::Right),
                _ => None,
            });
        if let Some(direction) = repeat {
            match presentation.model.repeat_pan(direction, time.delta_secs(), gates) {
                Ok(result) => production.pan_audio_active = result != ffone_client::world_map::WorldMapInputResult::Ignored,
                Err(error) => runtime.message = format!("WorldMap repeated pan rejected: {error:?}"),
            }
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        let dragging = presentation.model.is_dragging();
        presentation.model.end_drag(gates);
        if dragging && !presentation.model.is_dragging() {
            audio.push(ffone_client::gameplay_ui::GameplayUiAudioCue::ButtonSound);
        }
        let pressed = production.pressed_control.take();
        if pressed.is_some() && pressed == presentation.hover.control {
            apply_world_map_control(
                &mut commands,
                pressed.expect("pressed control was checked above"),
                &mut presentation,
                gates,
                &mut runtime,
                &mut audio,
            );
        }
    }
}

pub(super) fn consume_world_map_outbox(
    bridge: Res<NetworkBridge>,
    mut presentation: ResMut<WorldMapPresentation>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    for event in presentation.model.drain_outbox().collect::<Vec<_>>() {
        match event {
            WorldMapOutboxEvent::RequestPresentNpcTypes { last_sync_time } => {
                if let Err(error) = bridge.send(NetworkCommand::RequestPresentNpcTypes(
                    PresentNpcTypesRequest0104 { last_sync_time },
                )) {
                    runtime.message = format!("WorldMap present-NPC-types request failed: {error}");
                }
            }
            // The pure model has already committed the close. No additional
            // gameplay packet exists for clean WorldMapMode's local ExitMode.
            WorldMapOutboxEvent::ExitMode => {}
        }
    }
}

pub(super) fn fail_world_map_projection(
    presentation: &mut WorldMapPresentation,
    production: &mut WorldMapProductionRuntime,
    runtime: &mut RuntimeStatus,
    detail: String,
) {
    presentation.markers.clear();
    presentation.hover.marker_index = None;
    if production.last_projection_error.as_deref() != Some(detail.as_str()) {
        runtime.message = format!("WorldMap projection rejected: {detail}");
    }
    production.last_projection_error = Some(detail);
}

pub(super) struct RuntimeWorldMapCatalog<'a> {
    pub(super) content: &'a TutorialMissionContent,
    pub(super) mission_availability: &'a BTreeMap<i32, (bool, bool)>,
}

impl WorldMapCatalog for RuntimeWorldMapCatalog<'_> {
    fn npc(&self, npc_type: i32) -> WorldMapCatalogLookup<WorldMapNpcCatalogEntry> {
        let WorldMapCatalogLookup::Unique(mut entry) = WorldMapCatalog::npc(self.content, npc_type)
        else {
            return WorldMapCatalog::npc(self.content, npc_type);
        };
        let npc_class = self
            .content
            .gameplay_npc(npc_type)
            .expect("WorldMapCatalog returned a row for this NPC type")
            .npc_class;
        let (new_available, advance_available) =
            if npc_class_supports_world_quest_symbols(npc_class) {
                self.mission_availability
                    .get(&npc_type)
                    .copied()
                    .unwrap_or_default()
            } else {
                // Exact clean cnMissionManager outer class rejection. These rows
                // may still draw their Type4 service icon, but never a quest icon.
                (false, false)
            };
        entry.mission = match (new_available, advance_available) {
            (false, false) => WorldMapMissionAvailability::None,
            (true, false) => WorldMapMissionAvailability::New,
            (false, true) => WorldMapMissionAvailability::Advance,
            (true, true) => WorldMapMissionAvailability::NewAndAdvance,
        };
        WorldMapCatalogLookup::Unique(entry)
    }
}

pub(super) fn sync_world_map_projection(
    state: Res<State<ClientState>>,
    time: Res<Time>,
    bridge: Res<NetworkBridge>,
    authority: WorldMapProjectionAuthority,
    transportation: Res<TransportationCatalog>,
    client_npcs: Res<ClientNpcWaypointCatalog>,
    mission: Res<WorldMissionRuntime>,
    inventory: Res<LocalInventoryRuntime>,
    guide: Res<GuideRuntime>,
    nano_bank: Res<NanoFreeTuningBank0104>,
    waypoint: Res<WorldMissionWaypointRuntime>,
    mut runtime: ResMut<RuntimeStatus>,
    mut presentation: ResMut<WorldMapPresentation>,
    mut production: ResMut<WorldMapProductionRuntime>,
    players: Query<
        (&Transform, &LegacyPlayerController),
        (With<LocalPlayer>, Without<LegacyWorldColliderPending>),
    >,
    npcs: Query<(&NetworkNpc0104, &Transform), Without<LocalPlayer>>,
) {
    let content: &TutorialMissionContent = &authority.content;
    if *state.get() != ClientState::World {
        return;
    }
    if world_map_episode_id(runtime.map_number, &authority.race_catalog).is_none() {
        if presentation.model.phase() != WorldMapPhase::Closed {
            presentation.reset_session();
            presentation.markers.clear();
        }
        return;
    }
    if production.server_npcs.entries.is_none() {
        production.snapshot_retry_seconds -= time.delta_secs();
        if production.snapshot_retry_seconds <= 0.0 {
            production.snapshot_retry_seconds = 5.0;
            if let Err(error) = bridge.send(NetworkCommand::RequestPresentNpcTypes(
                PresentNpcTypesRequest0104 { last_sync_time: 0 },
            )) {
                runtime.message = format!("NPC map snapshot request failed: {error}");
            }
        }
    }

    // The server snapshot retains friendly placements outside streaming range.
    // Track streamed mobs separately; discard them after despawn or range exit.
    let mut live_ids = BTreeSet::new();
    for (npc, transform) in &npcs {
        live_ids.insert(npc.npc_id);
        if content.gameplay_npc_minimap(npc.npc_type).is_none() {
            continue;
        }
        if production.server_npcs.entries.is_some()
            && world_map_npc_visible(
                content.gameplay_npc(npc.npc_type).map(|row| row.npc_class),
                content.gameplay_npc_minimap(npc.npc_type).map(|row| row.sound),
                false,
            )
        {
            continue;
        }
        let position = native_to_unity_vector(transform.translation);
        let source = WorldMapNpcSource {
            npc_type: npc.npc_type,
            position: WorldMapPoint::new(position.x, position.y, position.z),
        };
        if production.observed_npcs.get(&npc.npc_id) != Some(&source) {
            production.observed_npcs.insert(npc.npc_id, source);
            production.npc_sources_dirty = true;
        }
    }
    if production.server_npcs.entries.is_some() {
        let previous = production.observed_npcs.len();
        production.observed_npcs.retain(|id, _| live_ids.contains(id));
        production.npc_sources_dirty |= production.observed_npcs.len() != previous;
    }
    if runtime.hp.is_some_and(|hp| hp <= 0) {
        if presentation.model.phase() != WorldMapPhase::Closed {
            presentation.reset_session();
            *production = WorldMapProductionRuntime::default();
        }
        return;
    }
    let Ok((transform, controller)) = players.single() else {
        if presentation.model.phase() != WorldMapPhase::Closed {
            presentation.reset_session();
            production.pressed_control = None;
            fail_world_map_projection(
                &mut presentation,
                &mut production,
                &mut runtime,
                "authoritative local player is missing or ambiguous".to_owned(),
            );
        }
        return;
    };
    let player = world_map_player_from_native(transform, controller);
    if let Err(error) = presentation.model.set_player(player) {
        fail_world_map_projection(
            &mut presentation,
            &mut production,
            &mut runtime,
            format!("invalid player state: {error:?}"),
        );
        return;
    }
    let waypoint = waypoint.native_target.map(|native| {
        let unity = native_to_unity_vector(native);
        WorldMapPoint::new(unity.x, unity.y, unity.z)
    });
    presentation
        .model
        .set_waypoint(waypoint)
        .expect("validated client-NPC waypoint coordinates are finite");
    presentation.current_location = legacy_world_location_name_or_last(
        &presentation.current_location,
        player.position.x,
        player.position.z,
    )
    .map(str::to_owned)
    .unwrap_or_else(|| runtime.map_name.clone());

    if presentation.model.phase() != WorldMapPhase::Open {
        presentation.markers.clear();
        presentation.hover.marker_index = None;
        production.last_projection_error = None;
        return;
    }
    if let Err(error) = presentation.model.advance_paint() {
        fail_world_map_projection(
            &mut presentation,
            &mut production,
            &mut runtime,
            format!("paint interpolation failed: {error:?}"),
        );
        return;
    }
    if production.npc_sources.is_empty() {
        production.npc_sources = world_map_npc_sources(&client_npcs);
        production.npc_sources_dirty = true;
    }
    let owned_nanos = nano_bank
        .entries()
        .iter()
        .filter_map(|nano| (nano.id > 0).then_some(i32::from(nano.id)))
        .collect::<BTreeSet<_>>();
    let guide = guide
        .authoritative()
        .map_or(0, |state| i32::from(state.raw_mentor()));
    let quest_inventory = inventory
        .quest_inventory
        .as_ref()
        .map(|inventory| inventory.as_slice())
        .unwrap_or(&[]);
    let mission_availability = mission.mission_availability_by_npc(
        i32::from(runtime.player_level),
        guide,
        &owned_nanos,
        quest_inventory,
        content,
    );
    let catalog = RuntimeWorldMapCatalog {
        content,
        mission_availability: &mission_availability,
    };
    if production.npc_sources_dirty {
        let combined = if let Some(entries) = &production.server_npcs.entries {
            let mut observed_positions: BTreeMap<i32, Vec<WorldMapPoint>> = BTreeMap::new();
            for observed in production.observed_npcs.values() {
                observed_positions.entry(observed.npc_type).or_default().push(observed.position);
            }
            let friendly = entries
                .values()
                .map(|entry| WorldMapNpcSource {
                    npc_type: entry.npc_type,
                    position: WorldMapPoint::new(
                        entry.position[0] as f32 / 100.,
                        entry.position[2] as f32 / 100.,
                        entry.position[1] as f32 / 100.,
                    ),
                });
            let authored_mobs = production.npc_sources.iter()
                .filter(|npc| runtime.map_number == Some(0)
                    && !observed_positions.get(&npc.npc_type).is_some_and(|positions|
                        positions.iter().any(|position|
                            (position.x - npc.position.x).powi(2)
                                + (position.z - npc.position.z).powi(2) < 1.0))
                    && !world_map_npc_visible(
                        content.gameplay_npc(npc.npc_type).map(|row| row.npc_class),
                        content.gameplay_npc_minimap(npc.npc_type).map(|row| row.sound),
                        false,
                    ))
                .cloned();
            let observed_mobs = production.observed_npcs.values()
                .filter(|npc| !world_map_npc_visible(
                    content.gameplay_npc(npc.npc_type).map(|row| row.npc_class),
                    content.gameplay_npc_minimap(npc.npc_type).map(|row| row.sound),
                    false,
                ))
                .cloned();
            friendly.chain(authored_mobs).chain(observed_mobs).collect()
        } else {
            let observed_types: BTreeSet<_> = production
                .observed_npcs
                .values()
                .map(|n| n.npc_type)
                .collect();
            production
                .npc_sources
                .iter()
                .filter(|n| !observed_types.contains(&n.npc_type))
                .cloned()
                .chain(production.observed_npcs.values().cloned())
                .collect()
        };
        production.combined_npc_sources = combined;
        production.npc_sources_dirty = false;
    }
    let visible_npcs: Vec<_> = production
        .combined_npc_sources
        .iter()
        .filter(|npc| {
            world_map_npc_visible(
                content.gameplay_npc(npc.npc_type).map(|definition| definition.npc_class),
                content.gameplay_npc_minimap(npc.npc_type).map(|definition| definition.sound),
                authority.skill_buffs.reveals_mobs(),
            )
        })
        .cloned()
        .collect();
    match presentation.model.project_markers(&catalog, &visible_npcs) {
        Ok(markers) => {
            presentation.transport.clear();
            for marker in &markers {
                let ffone_client::world_map::WorldMapMarkerKind::Npc {
                    npc_type, map_icon, ..
                } = marker.kind
                else {
                    continue;
                };
                if !matches!(map_icon, 21 | 22 | 34) {
                    continue;
                }
                let locations = if map_icon == 21 {
                    transportation.broomstick_locations()
                } else {
                    transportation.warp_locations()
                };
                let Some(start) = locations
                    .iter()
                    .find(|location| location.npc_id == npc_type && location.location_id != 0)
                else {
                    continue;
                };
                let registered = |id| {
                    if map_icon == 21 {
                        runtime.transportation_unlocks.wyvern_registered(id)
                    } else {
                        runtime.transportation_unlocks.warp_registered(id)
                    }
                };
                let mut seen = BTreeSet::new();
                let destinations = transportation
                    .routes()
                    .iter()
                    .filter(|route| {
                        route.start_location == start.location_id
                            && (map_icon != 21 || route.move_type == 2)
                    })
                    .filter_map(|route| {
                        let destination =
                            locations.get(usize::try_from(route.end_location).ok()?)?;
                        if route.end_location <= 0
                            || destination.location_id == 0
                            || !seen.insert(destination.row_index)
                        {
                            return None;
                        }
                        Some((
                            WorldMapPoint::new(destination.position.x, 0.0, destination.position.y),
                            registered(destination.location_id),
                        ))
                    })
                    .collect();
                presentation.transport.insert(
                    npc_type,
                    ffone_client::world_map::WorldMapTransportNode {
                        registered: registered(start.location_id),
                        destinations,
                    },
                );
            }
            presentation.respawn_position = warp_away_xcom_index(
                content,
                runtime.map_number,
                ProtocolPosition::from_native(transform.translation).raw(),
            )
            .and_then(|index| content.xcom_position(index))
            .and_then(|position| {
                let x = position[0] as f32 / 100.0;
                let z = position[1] as f32 / 100.0;
                let mut distance = i32::MAX;
                let mut result = None;
                for npc in &production.combined_npc_sources {
                    if !world_map_npc_visible(
                        content.gameplay_npc(npc.npc_type).map(|row| row.npc_class),
                        content.gameplay_npc_minimap(npc.npc_type).map(|row| row.sound),
                        false,
                    ) {
                        continue;
                    }
                    let candidate =
                        ((npc.position.x - x).powi(2) + (npc.position.z - z).powi(2)).sqrt();
                    if candidate < distance as f32 {
                        distance = candidate as i32;
                        result = Some(npc.position);
                    }
                }
                result
            });
            presentation.markers = markers;
            presentation.hover.marker_index =
                world_map_marker_at(&presentation.markers, presentation.hover.pointer);
            production.last_projection_error = None;
        }
        Err(error) => {
            fail_world_map_projection(
                &mut presentation,
                &mut production,
                &mut runtime,
                format!("{error:?}"),
            );
        }
    }
}
