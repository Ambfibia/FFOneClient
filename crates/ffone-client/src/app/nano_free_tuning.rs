//! Nano free-tuning runtime, camera/preview state and opening.

use super::guide::GuideProductionRuntime;
use super::local_inventory::LocalInventoryRuntime;
use super::runtime_status::RuntimeStatus;
use super::tutorial_choreography::*;
use super::tutorial_session::TutorialProjectileRandomStream;
use super::{LocalPlayer, WorldSliceEntity};
use bevy::{
    ecs::system::SystemParam,
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use ffone_client::{
    assets::AssetLocator,
    bank_runtime::BankProductionRuntime0104,
    bank_ui::{BankLifecyclePhase, BankUiOutbox0104, BankUiState},
    character_scene::{NativeSceneRole, native_scene_container_transform},
    entity_lifecycle::NetworkNpcAppearance0104,
    gameplay_nano_portraits::GameplayNanoPortraitCatalog,
    gameplay_ui::{GameplayUiModel, GameplayUiOutbox},
    guide_ui::{GuideUiModel, GuideUiOutbox},
    legacy_npc_nano_animation::LegacyNanoStandRandomStream,
    mission_ui::MissionUiModel,
    movement::{LegacyOrbitCamera, LegacyPlayerController},
    nano_free_tuning_runtime::{NanoFreeTuningBank0104, project_nano_free_tuning_content},
    nano_free_tuning_ui::{
        NanoFreeTuningModel, NanoFreeTuningOpenContext, NanoFreeTuningPhase,
        NanoFreeTuningTransform, NanoFreeTuningUiCommandOutbox, NanoFreeTuningWorldSnapshot,
    },
    nanocom_message_ui::NanocomMessageUiModel,
    network_world_runtime::{
        NetworkNpcAnimationSoundEvent0104, parse_network_npc_animation_sound_events,
    },
    option_ui::OptionUiModel,
    quit_menu_runtime::QuitMenuRuntime,
    quit_menu_ui::QuitMenuUiModel,
    resurrect_ui::ResurrectUiModel,
    rule_runtime::RuleRuntime,
    rule_ui::{RuleUiModel, RuleUiOutbox},
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    system_message_ui::SystemMessageUiModel,
    tutorial_effects_runtime::{TutorialEffectRuntime, TutorialEffectRuntimeCommand},
    tutorial_mission_content::TutorialMissionContent,
    upsell_ui::UpsellUiModel,
    user_equip_ui::UserEquipUiState,
    vendor_runtime::VendorProductionRuntime0104,
    vendor_ui::{VendorLifecyclePhase, VendorUiOutbox0104, VendorUiState},
    world_map::{WorldMapPhase, WorldMapPresentation},
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct NanoFreeTuningOpenTrigger {
    pub(super) nano_id: i16,
    pub(super) killed_fusion: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct NanoFreeTuningCursorSnapshot {
    pub(super) grab_mode: CursorGrabMode,
    pub(super) visible: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct NanoFreeTuningCameraSnapshot {
    pub(super) target: Entity,
    pub(super) sub_target_forward: Option<Vec3>,
    pub(super) height: f32,
    pub(super) minimum_distance: f32,
    pub(super) maximum_distance: f32,
    pub(super) distance: f32,
    pub(super) default_distance: f32,
    pub(super) pitch_degrees: f32,
    pub(super) default_pitch_degrees: f32,
    pub(super) minimum_pitch_degrees: f32,
    pub(super) maximum_pitch_degrees: f32,
    pub(super) yaw_degrees: f32,
}

impl NanoFreeTuningCameraSnapshot {
    pub(super) fn capture(camera: &LegacyOrbitCamera) -> Self {
        Self {
            target: camera.target,
            sub_target_forward: camera.sub_target_forward,
            height: camera.height,
            minimum_distance: camera.minimum_distance,
            maximum_distance: camera.maximum_distance,
            distance: camera.distance,
            default_distance: camera.default_distance,
            pitch_degrees: camera.pitch_degrees,
            default_pitch_degrees: camera.default_pitch_degrees,
            minimum_pitch_degrees: camera.minimum_pitch_degrees,
            maximum_pitch_degrees: camera.maximum_pitch_degrees,
            yaw_degrees: camera.yaw_degrees,
        }
    }

    pub(super) fn restore(self, camera: &mut LegacyOrbitCamera) {
        camera.target = self.target;
        camera.sub_target_forward = self.sub_target_forward;
        camera.height = self.height;
        camera.minimum_distance = self.minimum_distance;
        camera.maximum_distance = self.maximum_distance;
        camera.distance = self.distance;
        camera.default_distance = self.default_distance;
        camera.pitch_degrees = self.pitch_degrees;
        camera.default_pitch_degrees = self.default_pitch_degrees;
        camera.minimum_pitch_degrees = self.minimum_pitch_degrees;
        camera.maximum_pitch_degrees = self.maximum_pitch_degrees;
        camera.yaw_degrees = self.yaw_degrees;
    }
}

#[derive(Debug, Resource)]
pub(super) struct NanoFreeTuningProductionRuntime {
    pub(super) pending_open: Option<NanoFreeTuningOpenTrigger>,
    pub(super) continuation_open_after_close: Option<NanoFreeTuningOpenTrigger>,
    pub(super) preview_entity: Option<Entity>,
    pub(super) cursor_snapshot: Option<NanoFreeTuningCursorSnapshot>,
    pub(super) camera_snapshot: Option<NanoFreeTuningCameraSnapshot>,
    pub(super) pending_animation: Option<&'static str>,
    pub(super) queued_animations: VecDeque<&'static str>,
    pub(super) asset_wait_elapsed: f32,
    pub(super) animation_graph: Option<Handle<AnimationGraph>>,
    pub(super) animation_nodes: BTreeMap<String, AnimationNodeIndex>,
    pub(super) idle_random_state: u32,
    pub(super) upsell_update_enabled: bool,
    pub(super) visible_ecom_icons: BTreeSet<i32>,
    pub(super) first_use_checks: Vec<i32>,
}

impl Default for NanoFreeTuningProductionRuntime {
    fn default() -> Self {
        Self {
            pending_open: None,
            continuation_open_after_close: None,
            preview_entity: None,
            cursor_snapshot: None,
            camera_snapshot: None,
            pending_animation: None,
            queued_animations: VecDeque::new(),
            asset_wait_elapsed: 0.0,
            animation_graph: None,
            animation_nodes: BTreeMap::new(),
            idle_random_state: 0x6e61_6e6f,
            upsell_update_enabled: true,
            visible_ecom_icons: BTreeSet::from([5]),
            first_use_checks: Vec::new(),
        }
    }
}

impl NanoFreeTuningProductionRuntime {
    #[must_use]
    pub(super) fn modal_active(&self, model: &NanoFreeTuningModel) -> bool {
        model.phase() != NanoFreeTuningPhase::Closed || self.pending_open.is_some()
    }

    pub(super) fn reset_transient(&mut self) {
        self.pending_open = None;
        self.continuation_open_after_close = None;
        self.preview_entity = None;
        self.cursor_snapshot = None;
        self.camera_snapshot = None;
        self.pending_animation = None;
        self.queued_animations.clear();
        self.asset_wait_elapsed = 0.0;
        self.animation_graph = None;
        self.animation_nodes.clear();
        self.upsell_update_enabled = true;
        self.visible_ecom_icons = BTreeSet::from([5]);
        self.first_use_checks.clear();
    }

    pub(super) fn queue_animation(&mut self, clip_name: &'static str) {
        if self.pending_animation.is_none() {
            self.pending_animation = Some(clip_name);
        } else if self.pending_animation != Some(clip_name)
            && self.queued_animations.back().copied() != Some(clip_name)
        {
            self.queued_animations.push_back(clip_name);
        }
    }

    pub(super) fn complete_pending_animation(&mut self) {
        self.pending_animation = self.queued_animations.pop_front();
    }
}

#[derive(Component)]
pub(super) struct NanoFreeTuningPreview {
    pub(super) nano_id: i16,
    pub(super) gltf: Handle<Gltf>,
    pub(super) sound_events: Arc<[NetworkNpcAnimationSoundEvent0104]>,
}

#[derive(Component)]
pub(super) struct NanoFreeTuningPreviewAnimation {
    pub(super) clip_name: &'static str,
    pub(super) node: AnimationNodeIndex,
}

#[derive(Component)]
pub(super) struct NanoFreeTuningPreviewSoundCursor {
    pub(super) clip_name: &'static str,
    pub(super) node: AnimationNodeIndex,
    pub(super) seek_time: f32,
    pub(super) completions: u32,
}

pub(super) fn reset_nano_free_tuning_shell(
    commands: &mut Commands,
    model: &mut NanoFreeTuningModel,
    outbox: &mut NanoFreeTuningUiCommandOutbox,
    bank: &mut NanoFreeTuningBank0104,
    production: &mut NanoFreeTuningProductionRuntime,
    clear_bank: bool,
) {
    if let Some(entity) = production.preview_entity.take() {
        commands.entity(entity).despawn();
    }
    *model = NanoFreeTuningModel::default();
    outbox.clear();
    production.reset_transient();
    if clear_bank {
        bank.clear();
    }
}

pub(super) fn reset_nano_free_tuning_session(
    mut commands: Commands,
    mut model: ResMut<NanoFreeTuningModel>,
    mut outbox: ResMut<NanoFreeTuningUiCommandOutbox>,
    mut bank: ResMut<NanoFreeTuningBank0104>,
    mut production: ResMut<NanoFreeTuningProductionRuntime>,
    mut cameras: Query<&mut LegacyOrbitCamera>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Some(snapshot) = production.camera_snapshot.take()
        && let Ok(mut camera) = cameras.single_mut()
    {
        snapshot.restore(&mut camera);
    }
    if let Some(snapshot) = production.cursor_snapshot.take()
        && let Ok(mut cursor) = cursors.single_mut()
    {
        cursor.grab_mode = snapshot.grab_mode;
        cursor.visible = snapshot.visible;
    }
    reset_nano_free_tuning_shell(
        &mut commands,
        &mut model,
        &mut outbox,
        &mut bank,
        &mut production,
        true,
    );
}

#[derive(SystemParam)]
pub(super) struct NanoFreeTuningOpenOwners<'w> {
    pub(super) gameplay_ui: ResMut<'w, GameplayUiModel>,
    pub(super) mission_ui: ResMut<'w, MissionUiModel>,
    pub(super) nanocom_messages: ResMut<'w, NanocomMessageUiModel>,
    pub(super) system_messages: Res<'w, SystemMessageUiModel>,
    pub(super) quit_menu: Res<'w, QuitMenuUiModel>,
    pub(super) quit_runtime: Res<'w, QuitMenuRuntime>,
    pub(super) option_ui: Res<'w, OptionUiModel>,
    pub(super) resurrect_ui: Res<'w, ResurrectUiModel>,
    pub(super) upsell_ui: Res<'w, UpsellUiModel>,
    pub(super) guide_ui: Res<'w, GuideUiModel>,
    pub(super) guide_outbox: Res<'w, GuideUiOutbox>,
    pub(super) guide_production: Res<'w, GuideProductionRuntime>,
    pub(super) bank_state: Res<'w, BankUiState>,
    pub(super) bank_outbox: Res<'w, BankUiOutbox0104>,
    pub(super) bank_production: Res<'w, BankProductionRuntime0104>,
    pub(super) vendor_ui: Res<'w, VendorUiState>,
    pub(super) vendor_outbox: Res<'w, VendorUiOutbox0104>,
    pub(super) vendor_production: Res<'w, VendorProductionRuntime0104>,
    pub(super) rule_ui: Res<'w, RuleUiModel>,
    pub(super) rule_outbox: Res<'w, RuleUiOutbox>,
    pub(super) rule_runtime: Res<'w, RuleRuntime>,
    pub(super) user_equip_ui: Res<'w, UserEquipUiState>,
    pub(super) world_map: Res<'w, WorldMapPresentation>,
}

impl NanoFreeTuningOpenOwners<'_> {
    pub(super) fn blocked(&self) -> bool {
        self.mission_ui.gameplay_input_blocked()
            || self.system_messages.is_popup()
            || self.mission_ui.system_popup_active()
            || self.quit_menu.visible
            || self.quit_runtime.is_waiting_for_server()
            || self.option_ui.visible
            || self.resurrect_ui.visible
            || self.upsell_ui.visible()
            || self.guide_production.modal_active(&self.guide_ui)
            || !self.guide_outbox.is_empty()
            || self.bank_state.phase != BankLifecyclePhase::Hidden
            || self.bank_production.modal_active()
            || !self.bank_outbox.is_empty()
            || self.vendor_ui.phase != VendorLifecyclePhase::Hidden
            || self.vendor_production.modal_active()
            || !self.vendor_outbox.is_empty()
            || self.rule_runtime.modal_active(&self.rule_ui)
            || !self.rule_outbox.is_empty()
            || self.user_equip_ui.is_active()
            || self.world_map.model.phase() != WorldMapPhase::Closed
    }

    pub(super) fn close_main_game_transients(&mut self) {
        self.gameplay_ui.chat.input_enabled = false;
        self.gameplay_ui.chat.active = false;
        self.gameplay_ui.chat.input.clear();
        let mut ignored = GameplayUiOutbox::default();
        self.mission_ui.close_nanocom_menu(&mut ignored);
        self.nanocom_messages.set_expanded(false);
    }
}

pub(super) fn open_pending_nano_free_tuning(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    asset_locator: Res<AssetLocator>,
    nano_catalog: Res<GameplayNanoPortraitCatalog>,
    content: Res<TutorialMissionContent>,
    mut owners: NanoFreeTuningOpenOwners,
    mut runtime: ResMut<RuntimeStatus>,
    mut model: ResMut<NanoFreeTuningModel>,
    mut production: ResMut<NanoFreeTuningProductionRuntime>,
    mut stand_random: ResMut<LegacyNanoStandRandomStream>,
    players: Query<(Entity, &Transform), With<LocalPlayer>>,
    npcs: Query<(&NetworkNpcAppearance0104, &Transform)>,
    cameras: Query<&LegacyOrbitCamera>,
) {
    if model.phase() != NanoFreeTuningPhase::Closed {
        return;
    }
    let Some(trigger) = production.pending_open else {
        return;
    };
    if owners.blocked() || runtime.hp.is_some_and(|hp| hp <= 0) {
        return;
    }
    let Some(player_id) = runtime.player_id else {
        return;
    };
    let Ok((_player_entity, player_transform)) = players.single() else {
        return;
    };
    let Ok(camera) = cameras.single() else {
        runtime.message =
            "NanoFreeTuning blocked: the clean mode requires one gameplay camera".to_owned();
        production.pending_open = None;
        return;
    };
    let projected = match project_nano_free_tuning_content(&content, trigger.nano_id) {
        Ok(projected) => projected,
        Err(error) => {
            runtime.message = format!("NanoFreeTuning blocked: {error}");
            production.pending_open = None;
            return;
        }
    };
    let Some(model_path) = nano_catalog.model_path(trigger.nano_id) else {
        runtime.message = format!(
            "NanoFreeTuning blocked: Nano {} has no validated native GLB",
            trigger.nano_id
        );
        production.pending_open = None;
        return;
    };
    let sound_events = match asset_locator
        .read(model_path)
        .and_then(|bytes| parse_network_npc_animation_sound_events(&bytes))
    {
        Ok(events) => Arc::from(events),
        Err(error) => {
            runtime.message = format!(
                "NanoFreeTuning blocked: Nano {} AnimationEvent audio is invalid: {error}",
                trigger.nano_id
            );
            production.pending_open = None;
            return;
        }
    };

    let first_defeated_fusion = npcs
        .iter()
        .find(|(appearance, _)| appearance.0.hp <= 0)
        .map(|(_, transform)| NanoFreeTuningTransform {
            position: transform.translation,
            rotation: transform.rotation,
        });
    let world = NanoFreeTuningWorldSnapshot {
        player: NanoFreeTuningTransform {
            position: player_transform.translation,
            rotation: player_transform.rotation,
        },
        first_defeated_fusion,
    };
    let gltf: Handle<Gltf> = asset_server.load(model_path.to_owned());
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_path.to_owned()));
    let preview = commands
        .spawn((
            Name::new(format!(
                "Retrobution NanoFreeTuning Nano {}",
                trigger.nano_id
            )),
            WorldSliceEntity,
            NanoFreeTuningPreview {
                nano_id: trigger.nano_id,
                gltf,
                sound_events,
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, -3_000.0)),
            Visibility::Hidden,
        ))
        .with_child((
            Name::new(format!(
                "Retrobution NanoFreeTuning Nano {} Scene0",
                trigger.nano_id
            )),
            WorldAssetRoot(scene),
            native_scene_container_transform(NativeSceneRole::CharacterGameplay),
            Visibility::Inherited,
        ))
        .id();

    let context = NanoFreeTuningOpenContext {
        player_id,
        killed_fusion: trigger.killed_fusion,
        content: projected,
        world,
    };
    if let Err(error) = model.open(context) {
        commands.entity(preview).despawn();
        runtime.message = format!("NanoFreeTuning open rejected: {error}");
        production.pending_open = None;
        return;
    }
    owners.close_main_game_transients();
    // The production consumer waits for the native preload's terminal state,
    // just as the original mode waits for its asynchronous effect request.
    production.asset_wait_elapsed = 0.0;
    production.preview_entity = Some(preview);
    production.pending_animation = None;
    production.queued_animations.clear();
    production.animation_graph = None;
    production.animation_nodes.clear();
    // Clean `NanoMoveControllerClone.SetupModel` starts a stand clip while the
    // wrapper is still parked offscreen. The later `Call()` therefore blends
    // from a valid pose instead of exposing the imported bind pose.
    production.queue_animation(stand_random.next_stand_clip());
    production.camera_snapshot = Some(NanoFreeTuningCameraSnapshot::capture(camera));
    production.pending_open = None;
    runtime.message = format!(
        "NanoFreeTuning mode 22 opened for Nano {}{}",
        trigger.nano_id,
        if trigger.killed_fusion {
            " after Nano creation"
        } else {
            ""
        }
    );
}

pub(super) fn nano_free_tuning_audio_path(
    catalog: &NativeAudioCatalog,
    true_name: &str,
) -> Result<String, String> {
    let candidates = catalog
        .by_true_name(true_name)
        .into_iter()
        .filter(|asset| asset.category == NativeAudioCategory::Sfx)
        .collect::<Vec<_>>();
    let [asset] = candidates.as_slice() else {
        return Err(format!(
            "NanoFreeTuning sound {true_name:?} resolved to {} native SFX assets",
            candidates.len()
        ));
    };
    Ok(asset.path.clone())
}

pub(super) fn nano_free_tuning_creation_projectile_command(
    bullet_types: [i32; 2],
    source: Vec3,
    target: Vec3,
    projectile_random: &mut TutorialProjectileRandomStream,
) -> TutorialEffectRuntimeCommand {
    // `NanoFreeTuningMode.CreationAnimation` uses the player-to-position Oni
    // overload. Like the tutorial's same overload, BulletContainer treats it
    // as reverse motion and consumes four Unity random draws before spawn.
    sampled_oni_projectile_pair_command(bullet_types, source, target, true, 161, projectile_random)
}

#[derive(SystemParam)]
pub(super) struct NanoFreeTuningProductionInputs<'w, 's> {
    pub(super) bank: ResMut<'w, NanoFreeTuningBank0104>,
    pub(super) production: ResMut<'w, NanoFreeTuningProductionRuntime>,
    pub(super) inventory: ResMut<'w, LocalInventoryRuntime>,
    pub(super) runtime: ResMut<'w, RuntimeStatus>,
    pub(super) effects: ResMut<'w, TutorialEffectRuntime>,
    pub(super) projectile_random: ResMut<'w, TutorialProjectileRandomStream>,
    pub(super) stand_random: ResMut<'w, LegacyNanoStandRandomStream>,
    pub(super) audio_catalog: Res<'w, NativeAudioCatalog>,
    pub(super) resurrect_ui: Res<'w, ResurrectUiModel>,
    pub(super) players: Query<
        'w,
        's,
        (
            Entity,
            &'static mut Transform,
            &'static LegacyPlayerController,
        ),
        With<LocalPlayer>,
    >,
    pub(super) previews: Query<
        'w,
        's,
        (
            Entity,
            &'static mut Transform,
            &'static mut Visibility,
            &'static NanoFreeTuningPreview,
        ),
        (Without<LocalPlayer>, Without<LegacyOrbitCamera>),
    >,
    pub(super) cameras: Query<
        'w,
        's,
        (&'static mut LegacyOrbitCamera, &'static mut Transform),
        Without<LocalPlayer>,
    >,
    pub(super) cursors: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
}
