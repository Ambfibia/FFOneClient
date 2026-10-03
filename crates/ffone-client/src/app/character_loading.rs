//! Character data, creation prewarm, UI activation and selection/creation loading gates.

use super::WorldSliceEntity;
use super::asset_residency::{
    AssetResidency, AssetResidencyGroupId, TutorialEffectLibraryLoadStatus, resident_group_probe,
};
use super::character_flow::*;
use super::loading_screen::{GameplayLoadingPhase, GameplayLoadingState, ResourceLoadingScope};
use super::login::LoadedCharacterCreationData;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_session::TutorialSession;
use bevy::prelude::*;
use ffone_client::{
    character_creation_ui::{
        CharacterAppearance, CharacterCreationAssetStatus, CharacterCreationCapability,
        CharacterCreationScreen, CharacterCreationUiModel, CharacterGender, CharacterNameLists,
    },
    character_selection_portraits::{
        CharacterSelectionPortraitStatus, CharacterSelectionPortraitsModel,
    },
    character_selection_ui::{
        CharacterSelectionAssetStatus, CharacterSelectionUiModel, CharacterSlotUi,
    },
    player_preview::{
        NativePlayerPreviewModel, NativePlayerPreviewStatus, prewarm_native_player_look,
    },
    player_shared_rig::{NativePlayerRigAssetCache, NativePlayerRigCatalog},
    ui_startup::NativeUiStartupPhase,
};
use std::collections::VecDeque;

pub(super) fn initialize_character_creation_data(
    data: Res<LoadedCharacterCreationData>,
    mut names: ResMut<CharacterNameLists>,
    mut creator: ResMut<CharacterCreationUiModel>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    *names = data.0.name_lists();
    if !names.valid() {
        runtime.message = "Native character name table is incomplete".to_owned();
        return;
    }
    creator.name_table = CharacterCreationCapability::Enabled;
    match data.0.option_counts() {
        Ok(counts) => {
            creator.option_counts = counts;
            creator.creation_items = CharacterCreationCapability::Enabled;
            creator.save_appearance = CharacterCreationCapability::Enabled;
        }
        Err(error) => {
            runtime.message = format!("Native character choices are invalid: {error}");
        }
    }
}

pub(super) fn begin_character_creation_asset_prewarm(mut lease: ResMut<CharacterCreationAssetLease>) {
    if lease.total > 0 {
        return;
    }
    // The post-auth package only needs the two immediately reachable creator
    // previews. Prewarming every cosmetic permutation duplicated hundreds of
    // resolver calls, retained a large asset closure, and let an unrelated
    // optional item prevent existing characters from reaching selection.
    // Other variants remain demand-loaded when the player actually chooses
    // them; the complete option catalog itself is already resident.
    let pending = [CharacterGender::Boy, CharacterGender::Girl]
        .into_iter()
        .map(|gender| CharacterAppearance {
            gender,
            ..CharacterAppearance::default()
        })
        .collect::<VecDeque<_>>();
    lease.total = pending.len();
    lease.pending = pending;
}

pub(super) fn drive_character_creation_asset_prewarm(
    state: Res<State<ClientState>>,
    data: Res<LoadedCharacterCreationData>,
    asset_server: Res<AssetServer>,
    catalog: Res<NativePlayerRigCatalog>,
    mut rig_assets: ResMut<NativePlayerRigAssetCache>,
    mut lease: ResMut<CharacterCreationAssetLease>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if !matches!(
        *state.get(),
        ClientState::CharacterSelect
            | ClientState::CharacterCreateIntro
            | ClientState::CharacterCreate
    ) || lease.blocker.is_some()
    {
        return;
    }
    for _ in 0..CHARACTER_CREATION_PREWARM_BATCH_SIZE {
        let Some(appearance) = lease.pending.pop_front() else {
            break;
        };
        let result = data
            .0
            .resolve_creator(0, 0, "Prewarm", "Creator", &appearance)
            .map_err(|error| error.to_string())
            .and_then(|resolved| {
                prewarm_native_player_look(&asset_server, &mut rig_assets, &catalog, &resolved.look)
            });
        if let Err(error) = result {
            let error = format!(
                "Native character-creation catalog prewarm failed for {:?}: {error}",
                appearance.gender
            );
            warn!("{error}");
            runtime.message.clone_from(&error);
            lease.blocker = Some(error);
            lease.pending.clear();
            return;
        }
        lease.completed += 1;
    }
}

pub(super) fn release_character_creation_asset_lease(
    mut rig_assets: ResMut<NativePlayerRigAssetCache>,
    mut lease: ResMut<CharacterCreationAssetLease>,
) {
    rig_assets.release_cached_handles();
    lease.clear();
}

pub(super) fn activate_gameplay_ui(mut phase: ResMut<NextState<NativeUiStartupPhase>>) {
    phase.set(NativeUiStartupPhase::Gameplay);
}

pub(super) fn defer_native_ui(mut phase: ResMut<NextState<NativeUiStartupPhase>>) {
    phase.set(NativeUiStartupPhase::Deferred);
}

pub(super) fn activate_character_selection_ui(mut phase: ResMut<NextState<NativeUiStartupPhase>>) {
    phase.set(NativeUiStartupPhase::CharacterSelection);
}

pub(super) fn activate_character_creation_ui(mut phase: ResMut<NextState<NativeUiStartupPhase>>) {
    phase.set(NativeUiStartupPhase::CharacterCreation);
}

pub(super) fn begin_character_selection_loading(
    mut commands: Commands,
    runtime: Res<RuntimeStatus>,
    mut loading: ResMut<GameplayLoadingState>,
    mut buffered_entry: ResMut<BufferedCharacterEntry>,
    mut tutorial: ResMut<TutorialSession>,
    world_slice: Query<Entity, With<WorldSliceEntity>>,
) {
    // CharacterSelection never owns gameplay/tutorial streamed entities. This
    // also covers a synchronous tutorial-completion transport failure, where
    // no later WorldReady event exists to replace the current shard slice.
    for entity in &world_slice {
        commands.entity(entity).despawn();
    }
    // A manual tutorial exit uses the existing completion lock only as a
    // one-frame teardown barrier. It never sends SAVE_CHAR_TUTOR, and the
    // barrier must be released once CharacterSelection owns the frame.
    if tutorial.completion_requested && tutorial.character().is_none() {
        tutorial.reset_runtime_observation();
    }
    buffered_entry.clear();
    // Roster handling can already have sent SelectCharacter (automatic entry)
    // before this OnEnter runs. Preserve its handshake barrier; replacing it
    // with selection loading would reopen the menu during an active request.
    if runtime.roster.pending_character_entry_uid.is_none() {
        loading.begin(ResourceLoadingScope::CharacterSelection);
    }
}

pub(super) fn reset_character_selection_asset_lease(
    mut rig_assets: ResMut<NativePlayerRigAssetCache>,
    mut lease: ResMut<CharacterCreationAssetLease>,
) {
    // A world return starts a fresh bounded selection visit. Active rig and
    // material components keep their own handles; the cache need not retain
    // every appearance seen across previous roster visits.
    rig_assets.release_cached_handles();
    lease.clear();
}

pub(super) fn begin_character_creation_loading(mut loading: ResMut<GameplayLoadingState>) {
    loading.begin(ResourceLoadingScope::CharacterCreation);
}

pub(super) fn gate_character_selection_loading(
    state: Res<State<ClientState>>,
    selection: Res<CharacterSelectionUiModel>,
    creator: Res<CharacterCreationUiModel>,
    portraits: Res<CharacterSelectionPortraitsModel>,
    preview: Res<NativePlayerPreviewModel>,
    prewarm: Res<CharacterCreationAssetLease>,
    effect_library: Res<TutorialEffectLibraryLoadStatus>,
    asset_server: Res<AssetServer>,
    residency: Res<AssetResidency>,
    mut loading: ResMut<GameplayLoadingState>,
) {
    if *state.get() != ClientState::CharacterSelect
        || loading.scope != Some(ResourceLoadingScope::CharacterSelection)
    {
        return;
    }
    if let CharacterSelectionAssetStatus::Failed { path } = &selection.asset_status {
        loading.block(format!("character-selection asset failed: {path}"));
        return;
    }
    if let CharacterCreationAssetStatus::Failed { path } = &creator.asset_status {
        loading.block(format!("character-creation asset failed: {path}"));
        return;
    }
    if let Some(error) = prewarm.blocker.as_ref() {
        loading.block(format!("character-creation preload failed: {error}"));
        return;
    }
    if let TutorialEffectLibraryLoadStatus::Failed(error) = effect_library.as_ref() {
        loading.block(format!("first-cutscene effect catalog failed: {error}"));
        return;
    }
    let cutscene_assets =
        resident_group_probe(&asset_server, &residency, AssetResidencyGroupId::DexterShip);
    if let Some(error) = cutscene_assets.blocker.as_ref() {
        loading.block(format!("first-cutscene asset failed: {error}"));
        return;
    }

    let occupied = selection
        .slots
        .iter()
        .enumerate()
        .filter(|(_, slot)| matches!(slot, CharacterSlotUi::Occupied(_)))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    for index in &occupied {
        if let CharacterSelectionPortraitStatus::Blocked(error) = &portraits.slots[*index].status {
            loading.block(format!("character portrait {} failed: {error}", index + 1));
            return;
        }
    }
    if let NativePlayerPreviewStatus::Blocked(error) = &preview.status {
        loading.block(format!("selected character preview failed: {error}"));
        return;
    }
    let selection_ready = matches!(selection.asset_status, CharacterSelectionAssetStatus::Ready);
    let creation_ready = matches!(creator.asset_status, CharacterCreationAssetStatus::Ready);
    let prewarm_ready = prewarm.is_ready();
    let effect_library_ready = matches!(
        effect_library.as_ref(),
        TutorialEffectLibraryLoadStatus::Ready
    );
    let cutscene_assets_ready = cutscene_assets.is_ready();
    let ready_portraits = occupied
        .iter()
        .filter(|index| {
            matches!(
                portraits.slots[**index].status,
                CharacterSelectionPortraitStatus::ReadyAnimated { .. }
            )
        })
        .count();
    let preview_required = selection.selected_character().is_some();
    let preview_ready = !preview_required
        || matches!(
            preview.status,
            NativePlayerPreviewStatus::ReadyAnimated { .. }
        );
    // This is the single post-auth package: selection, creation, the creator
    // rig closure, and the first Dexter-ship cutscene. Route-specific tutorial
    // or open-world resources are intentionally not part of this barrier.
    let total = 5.0 + occupied.len() as f32 + usize::from(preview_required) as f32;
    let completed = usize::from(selection_ready) as f32
        + usize::from(creation_ready) as f32
        + prewarm.progress()
        + usize::from(effect_library_ready) as f32
        + cutscene_assets.progress()
        + ready_portraits as f32
        + usize::from(preview_required && preview_ready) as f32;
    let progress = completed / total.max(1.0);
    let acquisition_ready = selection_ready
        && creation_ready
        && prewarm_ready
        && effect_library_ready
        && cutscene_assets_ready;
    let phase_progress = if !selection_ready || !creation_ready {
        0.0
    } else if !prewarm_ready {
        prewarm.progress()
    } else if !effect_library_ready {
        0.0
    } else if !cutscene_assets_ready {
        cutscene_assets.progress()
    } else if ready_portraits < occupied.len() {
        ready_portraits as f32 / occupied.len().max(1) as f32
    } else if preview_required {
        if preview_ready { 1.0 } else { 0.0 }
    } else {
        1.0
    };
    if !(acquisition_ready && ready_portraits == occupied.len() && preview_ready) {
        if !acquisition_ready {
            loading.loading(progress, phase_progress);
        } else {
            let phase = if ready_portraits < occupied.len() {
                GameplayLoadingPhase::SceneAssembly
            } else {
                GameplayLoadingPhase::PresentationBinding
            };
            loading.prepare(phase, progress, phase_progress);
        }
        return;
    }
    if loading.settle_render_presentation() {
        loading.finish();
    }
}

pub(super) fn gate_character_creation_loading(
    state: Res<State<ClientState>>,
    creator: Res<CharacterCreationUiModel>,
    preview: Res<NativePlayerPreviewModel>,
    prewarm: Res<CharacterCreationAssetLease>,
    mut loading: ResMut<GameplayLoadingState>,
) {
    if *state.get() != ClientState::CharacterCreate
        || loading.scope != Some(ResourceLoadingScope::CharacterCreation)
    {
        return;
    }
    if let CharacterCreationAssetStatus::Failed { path } = creator.asset_status {
        loading.block(format!("character-creation asset failed: {path}"));
        return;
    }
    if creator.screen == CharacterCreationScreen::Appearance
        && let NativePlayerPreviewStatus::Blocked(error) = &preview.status
    {
        loading.block(format!("character-creation preview failed: {error}"));
        return;
    }
    if creator.screen == CharacterCreationScreen::Appearance
        && let Some(error) = prewarm.blocker.as_ref()
    {
        loading.block(format!("character-creation preload failed: {error}"));
        return;
    }
    let static_ready = matches!(creator.asset_status, CharacterCreationAssetStatus::Ready);
    let preview_required = creator.screen == CharacterCreationScreen::Appearance;
    let prewarm_required = preview_required;
    let preview_ready = !preview_required
        || matches!(
            preview.status,
            NativePlayerPreviewStatus::ReadyAnimated { .. }
        );
    let prewarm_ready = !prewarm_required || prewarm.is_ready();
    let total = 1.0 + usize::from(preview_required) as f32 + usize::from(prewarm_required) as f32;
    let completed = usize::from(static_ready) as f32
        + if preview_required {
            usize::from(preview_ready) as f32
        } else {
            0.0
        }
        + if prewarm_required {
            prewarm.progress()
        } else {
            0.0
        };
    let progress = completed / total;
    let phase_progress = if !static_ready {
        0.0
    } else if prewarm_required && !prewarm_ready {
        prewarm.progress()
    } else if preview_required && !preview_ready {
        0.0
    } else {
        1.0
    };
    if !(static_ready && preview_ready && prewarm_ready) {
        if !static_ready || !prewarm_ready {
            loading.loading(progress, phase_progress);
        } else {
            loading.prepare(
                GameplayLoadingPhase::PresentationBinding,
                progress,
                phase_progress,
            );
        }
        return;
    }
    if loading.settle_render_presentation() {
        loading.finish();
    }
}
