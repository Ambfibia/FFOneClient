//! Tutorial effect library loading and asset residency groups.

use super::dexter_ship_scene::{
    DEXTER_SHIP_BACKGROUND_PATH, DEXTER_SHIP_COMPUTER_EFFECT_PATH, DEXTER_SHIP_HOLOGRAM_DECAL_PATH,
    DEXTER_SHIP_NAME_AUDIO, DEXTER_SHIP_TUTORIAL_AUDIO, DexterShipActorRole,
    dexter_ship_audio_path,
};
use super::loading_screen::{
    LEGACY_LOADING_BACKGROUND_PATH, LEGACY_LOADING_BAR_PATH, LEGACY_LOADING_WINDOW_PATH,
};
use super::login::ClientConfig;
use super::state::ClientState;
use super::tutorial_ambience::{TUTORIAL_LAIR_AMBIENT_PATH, TUTORIAL_MAIN_AMBIENT_PATH};
use super::tutorial_combat::tutorial_nano_audio_path;
use super::tutorial_scene_audio::{
    tutorial_presenter_voice_path, tutorial_sound_asset, tutorial_sound_path_for_locale,
    tutorial_voice_path,
};
use bevy::{
    asset::{Asset, LoadState, RecursiveDependencyLoadState},
    audio::AudioSource,
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
};
use ffone_client::{
    localization::VoiceLanguage,
    network_world_runtime::{
        NetworkNpcVisualCatalog0104, NetworkNpcVisualCatalogRequest0104,
        NetworkNpcVisualCatalogState0104,
    },
    option_ui::{OPTION_CHALET_FONT_PATH, OPTION_JEFFE_FONT_PATH},
    semantic_audio::NativeAudioCatalog,
    tutorial::TutorialStage,
    tutorial_actors::tutorial_actor_npc_types,
    tutorial_auxiliary_choreography::TUTORIAL_INITIALIZATION,
    tutorial_choreography_runtime::TUTORIAL_PAN_PATHS,
    tutorial_effects_runtime::{TutorialEffectLibrary, TutorialEffectRuntime},
    tutorial_nano_gameplay::{
        TUTORIAL_BUTTERCUP_SKILL_SFX_TRUE_NAME, TUTORIAL_BUTTERCUP_SKILL_VOICE_TRUE_NAMES,
        TUTORIAL_BUTTERCUP_SUMMON_SFX_TRUE_NAME, TUTORIAL_BUTTERCUP_SUMMON_VOICE_TRUE_NAMES,
        TutorialNanoGameplayAudioCategory,
    },
    tutorial_presenter::{TUTORIAL_SCENE_PRESENTATIONS, TutorialAudioCueKind},
    tutorial_voice_subtitles::{
        TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH, TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{self, TryRecvError},
    },
    thread,
};

#[derive(Resource)]
pub(super) struct TutorialEffectLibraryLoader {
    pub(super) receiver: Mutex<mpsc::Receiver<Result<TutorialEffectLibrary, String>>>,
}

impl TutorialEffectLibraryLoader {
    pub(super) fn start(asset_root: PathBuf) -> Self {
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("ffone-effect-catalog-loader".to_owned())
            .spawn(move || {
                let result = TutorialEffectLibrary::load(asset_root).map_err(|error| error.0);
                let _ = sender.send(result);
            })
            .expect("tutorial effect catalog loader thread must start");
        Self {
            receiver: Mutex::new(receiver),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub(super) enum TutorialEffectLibraryLoadStatus {
    #[default]
    Dormant,
    Loading,
    Ready,
    Failed(String),
}

#[derive(Clone, Resource)]
pub(super) struct SharedTutorialEffectLibrary(pub(super) Arc<TutorialEffectLibrary>);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum AssetResidencyGroupId {
    DexterShip,
    Tutorial,
}

#[derive(Default)]
pub(super) struct ResidentAssetGroup {
    pub(super) voice_language: String,
    pub(super) gltfs: Vec<(String, Handle<Gltf>)>,
    pub(super) scenes: Vec<(String, Handle<WorldAsset>)>,
    pub(super) images: Vec<(String, Handle<Image>)>,
    pub(super) fonts: Vec<(String, Handle<Font>)>,
    pub(super) audio: Vec<(String, Handle<AudioSource>)>,
    pub(super) blockers: Vec<String>,
}

/// Strong handles owned only while their state-scoped load group is useful.
///
/// Paths continue to belong to typed code and the three domain catalogs. This
/// resource is deliberately a residency/lifetime layer, not another manifest.
#[derive(Default, Resource)]
pub(super) struct AssetResidency {
    pub(super) groups: BTreeMap<AssetResidencyGroupId, ResidentAssetGroup>,
}

impl AssetResidency {
    pub(super) fn group(&self, id: AssetResidencyGroupId) -> Option<&ResidentAssetGroup> {
        self.groups.get(&id)
    }

    pub(super) fn release_except(&mut self, keep: &[AssetResidencyGroupId]) {
        self.groups.retain(|id, _| keep.contains(id));
    }
}

pub(super) fn load_dexter_ship_residency_group(
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    npc_catalog: &NetworkNpcVisualCatalog0104,
    voice_language: &str,
) -> ResidentAssetGroup {
    let mut assets = ResidentAssetGroup {
        voice_language: voice_language.to_owned(),
        ..default()
    };

    let mut cutscene_models = vec![DEXTER_SHIP_COMPUTER_EFFECT_PATH];
    for role in DexterShipActorRole::ALL {
        match role.definition(npc_catalog) {
            Ok(definition) => {
                cutscene_models.push(definition.glb.as_str());
                // The shared binder loads table overrides with their exact
                // sampler/color settings. A plain Image preload here wins the
                // AssetServer cache race against glTF's sampler descriptor.
            }
            Err(error) => assets.blockers.push(error),
        }
    }
    for path in cutscene_models {
        assets
            .gltfs
            .push((path.to_owned(), asset_server.load(path.to_owned())));
        assets.scenes.push((
            format!("{path}#Scene0"),
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(path.to_owned())),
        ));
    }
    for path in [
        DEXTER_SHIP_BACKGROUND_PATH,
        DEXTER_SHIP_HOLOGRAM_DECAL_PATH,
        LEGACY_LOADING_BACKGROUND_PATH,
        LEGACY_LOADING_WINDOW_PATH,
        LEGACY_LOADING_BAR_PATH,
    ] {
        assets
            .images
            .push((path.to_owned(), asset_server.load(path)));
    }
    for path in [
        TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH,
        TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH,
        OPTION_CHALET_FONT_PATH,
        OPTION_JEFFE_FONT_PATH,
    ] {
        assets
            .fonts
            .push((path.to_owned(), asset_server.load(path)));
    }

    let mut cutscene_audio_paths = BTreeSet::new();
    for true_name in std::iter::once("CharacterCreation_Loop").chain(
        DEXTER_SHIP_NAME_AUDIO
            .iter()
            .chain(DEXTER_SHIP_TUTORIAL_AUDIO.iter())
            .filter(|cue| !cue.true_name.is_empty())
            .map(|cue| cue.true_name),
    ) {
        let Some((path, _)) = dexter_ship_audio_path(catalog, voice_language, true_name) else {
            assets
                .blockers
                .push(format!("semantic audio {true_name:?} is missing"));
            continue;
        };
        cutscene_audio_paths.insert(path);
    }
    assets.audio = cutscene_audio_paths
        .into_iter()
        .map(|path| {
            let handle = asset_server.load(path.clone());
            (path, handle)
        })
        .collect();

    assets
}

pub(super) fn tutorial_initial_actor_model_path(
    catalog: &NetworkNpcVisualCatalog0104,
) -> Result<&str, String> {
    let initial_npc = TUTORIAL_INITIALIZATION.initial_npc;
    catalog
        .get(initial_npc.npc_type)
        .map(|definition| definition.glb.as_str())
        .ok_or_else(|| {
            format!(
                "initial tutorial actor type {} has no validated shared XDT-to-GLB route",
                initial_npc.npc_type
            )
        })
}

pub(super) fn tutorial_residency_plain_image_paths() -> BTreeSet<String> {
    TUTORIAL_PAN_PATHS.into_iter().map(str::to_owned).collect()
}

pub(super) fn load_tutorial_residency_group(
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    npc_visual_catalog: &NetworkNpcVisualCatalog0104,
    voice_language: &str,
) -> ResidentAssetGroup {
    let mut assets = ResidentAssetGroup {
        voice_language: voice_language.to_owned(),
        ..default()
    };

    let mut tutorial_model_paths = BTreeSet::new();
    if let Err(error) = tutorial_initial_actor_model_path(npc_visual_catalog) {
        assets.blockers.push(error);
    }
    for npc_type in tutorial_actor_npc_types() {
        let Some(definition) = npc_visual_catalog.get(npc_type) else {
            continue;
        };
        tutorial_model_paths.insert(definition.glb.clone());
    }
    for path in tutorial_model_paths {
        assets
            .gltfs
            .push((path.clone(), asset_server.load(path.clone())));
        assets.scenes.push((
            format!("{path}#Scene0"),
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(path)),
        ));
    }
    // Do not plain-preload table-selected NPC textures here. Bevy keys Image
    // assets by path, so `asset_server.load(path)` would permanently win over
    // the later XDT binder's settings-carrying load request and discard the
    // source Texture2D sampler/color-space contract. The Editor never performs
    // that conflicting preload, which is why the same eyes and Fusion surfaces
    // looked correct there. The shared material binder owns the first and only
    // load of every XDT main/sub replacement.
    for path in tutorial_residency_plain_image_paths() {
        assets.images.push((path.clone(), asset_server.load(path)));
    }

    let mut tutorial_audio_paths = BTreeSet::new();
    tutorial_audio_paths.insert(TUTORIAL_MAIN_AMBIENT_PATH.to_owned());
    tutorial_audio_paths.insert(TUTORIAL_LAIR_AMBIENT_PATH.to_owned());
    for presentation in TUTORIAL_SCENE_PRESENTATIONS {
        for cue in presentation.audio {
            match cue.kind {
                TutorialAudioCueKind::Voice => {
                    match tutorial_presenter_voice_path(catalog, voice_language, cue) {
                        Ok(Some(path)) => {
                            tutorial_audio_paths.insert(path);
                        }
                        Ok(None) => {}
                        Err(error) => assets.blockers.push(error),
                    }
                }
                TutorialAudioCueKind::OneShot | TutorialAudioCueKind::StartLoop => {
                    match tutorial_sound_asset(catalog, cue) {
                        Ok(asset) => {
                            if let Some(path) =
                                tutorial_sound_path_for_locale(catalog, asset, voice_language)
                            {
                                tutorial_audio_paths.insert(path);
                            }
                        }
                        Err(error) => assets.blockers.push(error),
                    }
                }
                TutorialAudioCueKind::StopLoops => {}
            }
        }
    }
    for chapter in 0..=5 {
        for step in 0..=210 {
            let Some(cue) = TutorialStage::from_legacy(chapter, step)
                .and_then(|stage| stage.metadata().entry_voice_cue)
            else {
                continue;
            };
            match tutorial_voice_path(catalog, voice_language, cue) {
                Ok(Some(path)) => {
                    tutorial_audio_paths.insert(path);
                }
                Ok(None) => {}
                Err(error) => assets.blockers.push(error),
            }
        }
    }
    for true_name in [
        TUTORIAL_BUTTERCUP_SUMMON_SFX_TRUE_NAME,
        TUTORIAL_BUTTERCUP_SKILL_SFX_TRUE_NAME,
    ] {
        match tutorial_nano_audio_path(
            catalog,
            voice_language,
            true_name,
            TutorialNanoGameplayAudioCategory::Sfx,
        ) {
            Ok(Some(path)) => {
                tutorial_audio_paths.insert(path);
            }
            Ok(None) => {}
            Err(error) => assets.blockers.push(error),
        }
    }
    for true_name in TUTORIAL_BUTTERCUP_SUMMON_VOICE_TRUE_NAMES
        .into_iter()
        .chain(TUTORIAL_BUTTERCUP_SKILL_VOICE_TRUE_NAMES)
    {
        match tutorial_nano_audio_path(
            catalog,
            voice_language,
            true_name,
            TutorialNanoGameplayAudioCategory::Voice,
        ) {
            Ok(Some(path)) => {
                tutorial_audio_paths.insert(path);
            }
            Ok(None) => {}
            Err(error) => assets.blockers.push(error),
        }
    }
    assets.audio = tutorial_audio_paths
        .into_iter()
        .map(|path| {
            let handle = asset_server.load(path.clone());
            (path, handle)
        })
        .collect();

    assets
}

pub(super) fn sync_asset_residency_groups(
    state: Res<State<ClientState>>,
    asset_server: Res<AssetServer>,
    catalog: Res<NativeAudioCatalog>,
    voice_language: Res<VoiceLanguage>,
    mut residency: ResMut<AssetResidency>,
    mut npc_visual_catalog_request: ResMut<NetworkNpcVisualCatalogRequest0104>,
    npc_visual_catalog_state: Res<NetworkNpcVisualCatalogState0104>,
) {
    let desired = asset_residency_groups_for_state(*state.get());
    residency.release_except(desired);
    npc_visual_catalog_request.requested = !desired.is_empty();

    for &id in desired {
        let must_refresh = residency.group(id).map_or(true, |group| {
            group.voice_language.as_str() != voice_language.effective.as_str()
        });
        if !must_refresh {
            continue;
        }
        let Some(npc_visual_catalog) = npc_visual_catalog_state.catalog.as_ref() else {
            if let Some(error) = npc_visual_catalog_state.error.as_ref() {
                residency.groups.insert(
                    id,
                    ResidentAssetGroup {
                        voice_language: voice_language.effective.clone(),
                        blockers: vec![format!("shared NPC visual catalog failed: {error}")],
                        ..default()
                    },
                );
            }
            continue;
        };
        let group = match id {
            AssetResidencyGroupId::DexterShip => load_dexter_ship_residency_group(
                &asset_server,
                &catalog,
                npc_visual_catalog,
                &voice_language.effective,
            ),
            AssetResidencyGroupId::Tutorial => load_tutorial_residency_group(
                &asset_server,
                &catalog,
                npc_visual_catalog,
                &voice_language.effective,
            ),
        };
        residency.groups.insert(id, group);
    }
}

pub(super) fn asset_residency_groups_for_state(state: ClientState) -> &'static [AssetResidencyGroupId] {
    match state {
        ClientState::CharacterSelect | ClientState::CharacterCreateIntro => {
            &[AssetResidencyGroupId::DexterShip]
        }
        ClientState::TutorialIntro => &[
            AssetResidencyGroupId::DexterShip,
            AssetResidencyGroupId::Tutorial,
        ],
        ClientState::Tutorial => &[AssetResidencyGroupId::Tutorial],
        ClientState::Bootstrap
        | ClientState::Login
        | ClientState::CharacterCreate
        | ClientState::World => &[],
    }
}

#[derive(Debug, Default)]
pub(super) struct AssetGroupProbe {
    pub(super) completed_weight: u32,
    pub(super) total_weight: u32,
    pub(super) blocker: Option<String>,
}

impl AssetGroupProbe {
    pub(super) fn progress(&self) -> f32 {
        if self.total_weight == 0 {
            1.0
        } else {
            self.completed_weight as f32 / self.total_weight as f32
        }
    }

    pub(super) fn is_ready(&self) -> bool {
        self.completed_weight == self.total_weight && self.blocker.is_none()
    }
}

pub(super) fn observe_resident_assets<A: Asset>(
    asset_server: &AssetServer,
    handles: &[(String, Handle<A>)],
    weight: u32,
    probe: &mut AssetGroupProbe,
) {
    for (path, handle) in handles {
        probe.total_weight = probe.total_weight.saturating_add(weight);
        if asset_server.is_loaded_with_dependencies(handle.id()) {
            probe.completed_weight = probe.completed_weight.saturating_add(weight);
        } else if let LoadState::Failed(error) = asset_server.load_state(handle.id()) {
            probe
                .blocker
                .get_or_insert_with(|| format!("{path} load failed: {error}"));
        } else if let Some(RecursiveDependencyLoadState::Failed(error)) =
            asset_server.get_recursive_dependency_load_state(handle.id())
        {
            probe
                .blocker
                .get_or_insert_with(|| format!("{path} dependency load failed: {error}"));
        }
    }
}

pub(super) fn resident_group_probe(
    asset_server: &AssetServer,
    residency: &AssetResidency,
    id: AssetResidencyGroupId,
) -> AssetGroupProbe {
    let Some(assets) = residency.group(id) else {
        return AssetGroupProbe {
            total_weight: 1,
            ..default()
        };
    };
    let mut probe = AssetGroupProbe {
        blocker: assets.blockers.first().cloned(),
        ..default()
    };
    // A loaded GLB/Scene closure is substantially more expensive than a font
    // or short audio cue. Phase progress therefore follows expected work
    // instead of treating every Handle as an equal unit.
    observe_resident_assets(asset_server, &assets.gltfs, 6, &mut probe);
    observe_resident_assets(asset_server, &assets.scenes, 4, &mut probe);
    observe_resident_assets(asset_server, &assets.images, 2, &mut probe);
    observe_resident_assets(asset_server, &assets.fonts, 1, &mut probe);
    observe_resident_assets(asset_server, &assets.audio, 1, &mut probe);
    probe
}

pub(super) fn begin_tutorial_effect_library_load(
    mut commands: Commands,
    config: Res<ClientConfig>,
    loader: Option<Res<TutorialEffectLibraryLoader>>,
    mut status: ResMut<TutorialEffectLibraryLoadStatus>,
) {
    if !matches!(*status, TutorialEffectLibraryLoadStatus::Dormant) || loader.is_some() {
        return;
    }
    // This is the largest validated JSON closure in the client. Starting it
    // during login competes with the login UI and authentication. The closed
    // post-auth character package is its first consumer and waits fail-closed
    // before it reveals character selection.
    commands.insert_resource(TutorialEffectLibraryLoader::start(
        config.asset_root.clone(),
    ));
    *status = TutorialEffectLibraryLoadStatus::Loading;
}

pub(super) fn poll_tutorial_effect_library(
    mut commands: Commands,
    loader: Option<Res<TutorialEffectLibraryLoader>>,
    mut status: ResMut<TutorialEffectLibraryLoadStatus>,
    mut runtime: ResMut<TutorialEffectRuntime>,
) {
    if !matches!(*status, TutorialEffectLibraryLoadStatus::Loading) {
        return;
    }
    let Some(loader) = loader else {
        *status = TutorialEffectLibraryLoadStatus::Failed(
            "tutorial effect catalog loader disappeared before completion".to_owned(),
        );
        return;
    };
    let result = match loader.receiver.lock() {
        Ok(receiver) => receiver.try_recv(),
        Err(_) => {
            *status = TutorialEffectLibraryLoadStatus::Failed(
                "tutorial effect catalog loader lock was poisoned".to_owned(),
            );
            return;
        }
    };
    match result {
        Ok(Ok(library)) => {
            let library = Arc::new(library);
            runtime.install_shared_library(Arc::clone(&library));
            commands.insert_resource(SharedTutorialEffectLibrary(library));
            commands.remove_resource::<TutorialEffectLibraryLoader>();
            *status = TutorialEffectLibraryLoadStatus::Ready;
        }
        Ok(Err(error)) => {
            commands.remove_resource::<TutorialEffectLibraryLoader>();
            *status = TutorialEffectLibraryLoadStatus::Failed(error);
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            commands.remove_resource::<TutorialEffectLibraryLoader>();
            *status = TutorialEffectLibraryLoadStatus::Failed(
                "tutorial effect catalog loader exited without a result".to_owned(),
            );
        }
    }
}
