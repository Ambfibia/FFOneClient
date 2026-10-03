//! Tutorial audio components, ambient cues/fades, dome and legacy world ambience.

use super::WorldSliceEntity;
use super::dexter_ship_drive::dexter_ship_descendant;
use super::option_runtime::OptionProductionRuntime;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_startup::{TutorialStartupProbe, native_asset_probe};
use bevy::{
    audio::{AudioSink, AudioSinkPlayback, AudioSource, Volume},
    gltf::Gltf,
    prelude::*,
};
use ffone_client::{
    character_selection_portraits::CharacterSelectionPortraitLight,
    legacy_glow::LegacyGlowSettings,
    legacy_model_material::LegacyModelMaterial,
    movement::LegacyOrbitCamera,
    native_terrain::{
        NATIVE_TERRAIN_ANISOTROPIC_TAPS, NATIVE_TERRAIN_ISOTROPIC_TAPS, NativeTerrainMaterial,
    },
    terrain_ambience::distance_haze_density,
    tutorial::{NanoPowerStage, TutorialStage},
    world::NativeWorldCatalog,
};

pub(super) const TUTORIAL_MAIN_AMBIENT_TRUE_NAME: &str = "TutorialMain_wAmbient_Loop";
pub(super) const TUTORIAL_MAIN_AMBIENT_PATH: &str = "audio/ambient/tutorialmain_wambient_loop.ogg";
pub(super) const TUTORIAL_LAIR_AMBIENT_TRUE_NAME: &str = "ButtercupLair_wAmbient_Loop";
pub(super) const TUTORIAL_LAIR_AMBIENT_PATH: &str = "audio/ambient/buttercuplair_wambient_loop.ogg";
pub(super) const TUTORIAL_LAIR_WARP_ID: i32 = 254;
pub(super) const TUTORIAL_AMBIENT_FADE_STEPS: u8 = 10;
pub(super) const TUTORIAL_AMBIENT_SECONDS_PER_STEP: f32 = 0.05;

#[derive(Component)]
pub(super) struct TutorialVoiceAudio;

#[derive(Component)]
pub(super) struct TutorialSceneAudio;

#[derive(Component)]
pub(super) struct TutorialCutsceneSfxAudio;

#[derive(Component)]
pub(super) struct TutorialMusicAudio;

#[derive(Component)]
pub(super) struct TutorialLoopAudio {
    pub(super) cue: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TutorialAmbientCue {
    TutorialMain,
    ButtercupLair,
}

impl TutorialAmbientCue {
    pub(super) const fn true_name(self) -> &'static str {
        match self {
            Self::TutorialMain => TUTORIAL_MAIN_AMBIENT_TRUE_NAME,
            Self::ButtercupLair => TUTORIAL_LAIR_AMBIENT_TRUE_NAME,
        }
    }

    pub(super) const fn semantic_path(self) -> &'static str {
        match self {
            Self::TutorialMain => TUTORIAL_MAIN_AMBIENT_PATH,
            Self::ButtercupLair => TUTORIAL_LAIR_AMBIENT_PATH,
        }
    }
}

pub(super) fn tutorial_ambient_cue_from_legacy(cue: &str) -> Result<Option<TutorialAmbientCue>, String> {
    if cue.to_ascii_lowercase().contains("none") {
        return Ok(None);
    }
    match cue {
        TUTORIAL_MAIN_AMBIENT_TRUE_NAME => Ok(Some(TutorialAmbientCue::TutorialMain)),
        TUTORIAL_LAIR_AMBIENT_TRUE_NAME => Ok(Some(TutorialAmbientCue::ButtercupLair)),
        _ => Err(format!("unknown exact tutorial ambient cue {cue:?}")),
    }
}

pub(super) fn tutorial_ambient_after_confirmed_warp(warp_id: i32) -> Option<TutorialAmbientCue> {
    (warp_id == TUTORIAL_LAIR_WARP_ID).then_some(TutorialAmbientCue::ButtercupLair)
}

pub(super) fn tutorial_ambient_after_chapter_init(stage: TutorialStage) -> Option<TutorialAmbientCue> {
    matches!(
        stage,
        TutorialStage::NanoPower(NanoPowerStage::AwaitCollapse)
    )
    .then_some(TutorialAmbientCue::TutorialMain)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum TutorialAmbientPhase {
    #[default]
    Stopped,
    Preloading,
    FadingOut,
    WaitingForSink,
    FadingIn,
    Stable,
    Blocked,
}

#[derive(Debug, Resource)]
pub(super) struct TutorialAmbientRuntime {
    pub(super) phase: TutorialAmbientPhase,
    pub(super) current_cue: Option<TutorialAmbientCue>,
    pub(super) current_entity: Option<Entity>,
    pub(super) target_cue: Option<TutorialAmbientCue>,
    pub(super) target_handle: Option<Handle<AudioSource>>,
    pub(super) requested: Option<Option<TutorialAmbientCue>>,
    pub(super) fade_step: u8,
    pub(super) step_elapsed_seconds: f32,
    pub(super) blocked_error: Option<String>,
}

impl Default for TutorialAmbientRuntime {
    fn default() -> Self {
        Self {
            phase: TutorialAmbientPhase::Stopped,
            current_cue: None,
            current_entity: None,
            target_cue: None,
            target_handle: None,
            requested: None,
            fade_step: 0,
            step_elapsed_seconds: 0.0,
            blocked_error: None,
        }
    }
}

impl TutorialAmbientRuntime {
    pub(super) fn request_legacy_cue(&mut self, cue: &str) -> Result<(), String> {
        self.request(tutorial_ambient_cue_from_legacy(cue)?);
        Ok(())
    }

    pub(super) fn request(&mut self, cue: Option<TutorialAmbientCue>) {
        let effective_target = self.requested.unwrap_or(self.target_cue);
        if effective_target == cue
            && !matches!(
                self.phase,
                TutorialAmbientPhase::Stopped | TutorialAmbientPhase::Blocked
            )
        {
            return;
        }
        if self.requested.is_none()
            && self.current_cue == cue
            && matches!(
                self.phase,
                TutorialAmbientPhase::Stable | TutorialAmbientPhase::Stopped
            )
        {
            return;
        }
        self.requested = Some(cue);
    }

    pub(super) fn is_stable(&self, cue: TutorialAmbientCue) -> bool {
        self.current_cue == Some(cue)
            && self.target_cue == Some(cue)
            && self.current_entity.is_some()
            && self.phase == TutorialAmbientPhase::Stable
    }

    pub(super) fn reset_transition_clock(&mut self) {
        self.fade_step = 0;
        self.step_elapsed_seconds = 0.0;
    }
}

#[derive(Component)]
pub(super) struct TutorialAmbientAudio {
    pub(super) cue: TutorialAmbientCue,
}

#[derive(Component)]
pub(super) struct TutorialDome {
    pub(super) gltf: Handle<Gltf>,
    pub(super) scene: Handle<WorldAsset>,
}

/// Reproduces `DongLoader.GetPositionColor` followed by
/// `cnPlayerCamera.DefaultAmbience`. The catalog contains the exact original
/// 16x16 source grid and interpolation. The legacy method assigns the sampled
/// fog to `RenderSettings.fogColor` before it brightens the by-reference value
/// used as the tutorial skybox tint, so world fog must retain the raw sample.
pub(super) fn legacy_world_ambience_active(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::Tutorial | ClientState::World)
}

pub(super) const fn client_state_uses_tutorial_ambience(state: ClientState) -> bool {
    matches!(state, ClientState::Tutorial)
}

#[derive(Debug, Default, Resource)]
pub(super) struct LegacyWorldAmbienceCache {
    pub(super) camera: Option<Entity>,
    pub(super) state: Option<ClientState>,
    pub(super) translation: Vec3,
    pub(super) directional_light_count: usize,
    pub(super) glow_count: usize,
    pub(super) terrain_material_count: usize,
    pub(super) camera_far: f32,
    pub(super) terrain_anisotropic_taps: f32,
}

pub(super) fn apply_legacy_world_ambience(
    mut commands: Commands,
    state: Res<State<ClientState>>,
    catalog: Res<NativeWorldCatalog>,
    cameras: Query<(Entity, &Transform, &Projection), (With<Camera3d>, With<LegacyOrbitCamera>)>,
    mut lights: Query<&mut DirectionalLight, Without<CharacterSelectionPortraitLight>>,
    mut glow_settings: Query<&mut LegacyGlowSettings>,
    mut terrain_materials: ResMut<Assets<NativeTerrainMaterial>>,
    option_runtime: Option<Res<OptionProductionRuntime>>,
    mut cache: ResMut<LegacyWorldAmbienceCache>,
) {
    let Ok((camera, camera_transform, projection)) = cameras.single() else {
        return;
    };
    let camera_far = projection.far();
    // A runtime without the option owner (GPU acceptance and editor tooling)
    // keeps the filtered default instead of inventing a persisted setting.
    let terrain_anisotropic_taps =
        option_runtime
            .as_deref()
            .map_or(NATIVE_TERRAIN_ANISOTROPIC_TAPS, |runtime| {
                if runtime.options.graphics.anisotropic_filtering {
                    NATIVE_TERRAIN_ANISOTROPIC_TAPS
                } else {
                    NATIVE_TERRAIN_ISOTROPIC_TAPS
                }
            });
    let directional_light_count = lights.iter().count();
    let glow_count = glow_settings.iter().count();
    let terrain_material_count = terrain_materials.len();
    if cache.camera == Some(camera)
        && cache.state == Some(*state.get())
        && cache.translation == camera_transform.translation
        && cache.directional_light_count == directional_light_count
        && cache.glow_count == glow_count
        && cache.terrain_material_count == terrain_material_count
        && cache.camera_far == camera_far
        && cache.terrain_anisotropic_taps == terrain_anisotropic_taps
    {
        return;
    }
    cache.camera = Some(camera);
    cache.state = Some(*state.get());
    cache.translation = camera_transform.translation;
    cache.directional_light_count = directional_light_count;
    cache.glow_count = glow_count;
    cache.terrain_material_count = terrain_material_count;
    cache.camera_far = camera_far;
    cache.terrain_anisotropic_taps = terrain_anisotropic_taps;
    // `cnPlayerCamera.Update` passes the camera transform, not the avatar,
    // to `DongLoader.GetPositionColor`. The two diverge throughout tutorial
    // cinematics, so sampling the player produces the wrong fog, light and
    // sky tint precisely while those scenes are playing.
    let tutorial = client_state_uses_tutorial_ambience(*state.get());
    let sample = catalog.sample_ambience(camera_transform.translation, tutorial);
    let applied = catalog.applied_ambience(camera_transform.translation, tutorial);
    let fog_color = Color::srgba(
        sample.fog_color[0],
        sample.fog_color[1],
        sample.fog_color[2],
        sample.fog_color[3],
    );
    // `DefaultAmbience` alone leaves a legible band of unfogged geometry at the
    // native far clip and at every object's own visibility range, which is what
    // exposed the short draw distance. Raise only the density to the value that
    // closes the horizon at this camera's far plane; the color stays the exact
    // sampled area fog, so the Future keeps its green haze and Downtown its
    // blue one.
    let fog_density = distance_haze_density(&applied, camera_far);
    commands.entity(camera).insert(DistanceFog {
        color: fog_color,
        directional_light_color: Color::NONE,
        falloff: FogFalloff::Exponential {
            density: fog_density,
        },
        ..default()
    });
    let light_color = Color::srgba(
        applied.light_color[0],
        applied.light_color[1],
        applied.light_color[2],
        applied.light_color[3],
    );
    for light in &mut lights {
        light
            .map_unchanged(|value| &mut value.color)
            .set_if_neq(light_color);
    }
    if let Ok(mut glow) = glow_settings.get_mut(camera) {
        glow.set_filter_color(applied.light_color);
    }
    let ambience_light = Vec4::from_array(applied.light_color);
    let ambience_fog = Vec4::new(
        sample.fog_color[0],
        sample.fog_color[1],
        sample.fog_color[2],
        fog_density,
    );
    let render_quality = Vec4::new(terrain_anisotropic_taps, 0.0, 0.0, 0.0);
    // Mutable asset iteration reports every resident terrain as modified, even
    // when camera movement sampled exactly the same ambience. Only changed
    // uniforms need GPU preparation; preserve the original sampling cadence.
    let changed: Vec<_> = terrain_materials
        .iter()
        .filter_map(|(id, material)| {
            (material.uniform.ambience_light != ambience_light
                || material.uniform.ambience_fog != ambience_fog
                || material.uniform.render_quality != render_quality)
                .then_some(id)
        })
        .collect();
    for id in changed {
        if let Some(mut material) = terrain_materials.get_mut(id) {
            material.uniform.ambience_light = ambience_light;
            material.uniform.ambience_fog = ambience_fog;
            material.uniform.render_quality = render_quality;
        }
    }
}

/// `ETC_domeglass_04` owns the autoplaying `nif-default` AnimationClip
/// (pathId 1021). Its only curve is `Line02` material
/// `_MainTex.offset.y = time * 0.15`, cycling after the texture reaches 1.
/// glTF cannot encode a Unity material-property animation, so apply that exact
/// curve to the converted native material.
pub(super) fn animate_tutorial_dome_material(
    time: Res<Time>,
    domes: Query<Entity, With<TutorialDome>>,
    parents: Query<&ChildOf>,
    mut surfaces: Query<(Entity, &mut MeshMaterial3d<LegacyModelMaterial>)>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
) {
    let Some(dome) = domes.iter().next() else {
        return;
    };
    let offset_y = (time.elapsed_secs() * 0.15).fract();
    for (entity, mut handle) in &mut surfaces {
        if !dexter_ship_descendant(entity, dome, &parents) {
            continue;
        }
        ffone_client::legacy_model_material::make_legacy_material_unique(
            &mut handle.0,
            &mut materials,
        );
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.uniform.uv_scale_offset.w = offset_y;
        }
    }
}

pub(super) fn cleanup_tutorial_ambient(
    mut commands: Commands,
    mut ambient: ResMut<TutorialAmbientRuntime>,
    ambient_audio: Query<Entity, With<TutorialAmbientAudio>>,
) {
    for entity in &ambient_audio {
        commands.entity(entity).despawn();
    }
    *ambient = TutorialAmbientRuntime::default();
}

pub(super) fn tutorial_ambient_fade_volume(fading_in: bool, completed_steps: u8) -> f32 {
    let completed = f32::from(completed_steps.min(TUTORIAL_AMBIENT_FADE_STEPS));
    let fraction = completed / f32::from(TUTORIAL_AMBIENT_FADE_STEPS);
    if fading_in { fraction } else { 1.0 - fraction }
}

pub(super) fn advance_tutorial_ambient_fade(ambient: &mut TutorialAmbientRuntime, delta_seconds: f32) -> bool {
    ambient.step_elapsed_seconds += delta_seconds.max(0.0);
    let previous_step = ambient.fade_step;
    while ambient.fade_step < TUTORIAL_AMBIENT_FADE_STEPS
        && ambient.step_elapsed_seconds + f32::EPSILON >= TUTORIAL_AMBIENT_SECONDS_PER_STEP
    {
        ambient.step_elapsed_seconds -= TUTORIAL_AMBIENT_SECONDS_PER_STEP;
        ambient.fade_step += 1;
    }
    ambient.fade_step != previous_step
}

pub(super) fn begin_tutorial_ambient_target(
    commands: &mut Commands,
    ambient: &mut TutorialAmbientRuntime,
) -> Result<(), String> {
    ambient.reset_transition_clock();
    let Some(cue) = ambient.target_cue else {
        ambient.current_cue = None;
        ambient.current_entity = None;
        ambient.target_handle = None;
        ambient.phase = TutorialAmbientPhase::Stopped;
        return Ok(());
    };
    let Some(handle) = ambient.target_handle.take() else {
        return Err(format!(
            "exact tutorial ambient {} was not preloaded before playback",
            cue.true_name()
        ));
    };
    let entity = commands
        .spawn((
            Name::new(format!("Tutorial ambient {}", cue.true_name())),
            WorldSliceEntity,
            TutorialAmbientAudio { cue },
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
        ))
        .id();
    ambient.current_cue = Some(cue);
    ambient.current_entity = Some(entity);
    ambient.phase = TutorialAmbientPhase::WaitingForSink;
    Ok(())
}

pub(super) fn block_tutorial_ambient(
    ambient: &mut TutorialAmbientRuntime,
    runtime: &mut RuntimeStatus,
    error: String,
) {
    runtime.message = format!("Tutorial ambient blocked: {error}");
    ambient.blocked_error = Some(error);
    ambient.phase = TutorialAmbientPhase::Blocked;
}

pub(super) fn drive_tutorial_ambient(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut ambient: ResMut<TutorialAmbientRuntime>,
    mut runtime: ResMut<RuntimeStatus>,
    mut audio: Query<(&TutorialAmbientAudio, Option<&mut AudioSink>)>,
) {
    if let Some(requested) = ambient.requested.take() {
        ambient.target_cue = requested;
        ambient.target_handle = requested.map(|cue| asset_server.load(cue.semantic_path()));
        ambient.blocked_error = None;
        ambient.reset_transition_clock();
        ambient.phase = TutorialAmbientPhase::Preloading;
    }

    match ambient.phase {
        TutorialAmbientPhase::Stopped
        | TutorialAmbientPhase::Stable
        | TutorialAmbientPhase::Blocked => {}
        TutorialAmbientPhase::Preloading => {
            if let (Some(cue), Some(handle)) = (ambient.target_cue, ambient.target_handle.as_ref())
            {
                match native_asset_probe(&asset_server, handle, cue.true_name()) {
                    TutorialStartupProbe::Ready => {}
                    TutorialStartupProbe::Loading(_) => return,
                    TutorialStartupProbe::Blocked(error) => {
                        block_tutorial_ambient(&mut ambient, &mut runtime, error);
                        return;
                    }
                }
            }

            if ambient.current_cue == ambient.target_cue {
                if ambient
                    .current_entity
                    .is_some_and(|entity| matches!(audio.get_mut(entity), Ok((_, Some(_)))))
                {
                    ambient.target_handle = None;
                    ambient.phase = TutorialAmbientPhase::Stable;
                    return;
                }
                ambient.current_cue = None;
                ambient.current_entity = None;
            }

            if ambient.current_entity.is_some() {
                ambient.reset_transition_clock();
                ambient.phase = TutorialAmbientPhase::FadingOut;
            } else if let Err(error) = begin_tutorial_ambient_target(&mut commands, &mut ambient) {
                block_tutorial_ambient(&mut ambient, &mut runtime, error);
            }
        }
        TutorialAmbientPhase::FadingOut => {
            let Some(entity) = ambient.current_entity else {
                if let Err(error) = begin_tutorial_ambient_target(&mut commands, &mut ambient) {
                    block_tutorial_ambient(&mut ambient, &mut runtime, error);
                }
                return;
            };
            let Ok((marker, sink)) = audio.get_mut(entity) else {
                ambient.current_cue = None;
                ambient.current_entity = None;
                if let Err(error) = begin_tutorial_ambient_target(&mut commands, &mut ambient) {
                    block_tutorial_ambient(&mut ambient, &mut runtime, error);
                }
                return;
            };
            if Some(marker.cue) != ambient.current_cue {
                block_tutorial_ambient(
                    &mut ambient,
                    &mut runtime,
                    "single-owner ambient entity contradicts runtime cue".to_owned(),
                );
                return;
            }
            let Some(mut sink) = sink else {
                return;
            };
            if advance_tutorial_ambient_fade(&mut ambient, time.delta_secs()) {
                sink.set_volume(Volume::Linear(tutorial_ambient_fade_volume(
                    false,
                    ambient.fade_step,
                )));
            }
            if ambient.fade_step == TUTORIAL_AMBIENT_FADE_STEPS {
                sink.stop();
                commands.entity(entity).despawn();
                ambient.current_cue = None;
                ambient.current_entity = None;
                if let Err(error) = begin_tutorial_ambient_target(&mut commands, &mut ambient) {
                    block_tutorial_ambient(&mut ambient, &mut runtime, error);
                }
            }
        }
        TutorialAmbientPhase::WaitingForSink => {
            let Some(entity) = ambient.current_entity else {
                block_tutorial_ambient(
                    &mut ambient,
                    &mut runtime,
                    "single-owner ambient entity disappeared before playback".to_owned(),
                );
                return;
            };
            let Ok((marker, sink)) = audio.get_mut(entity) else {
                return;
            };
            if Some(marker.cue) != ambient.current_cue {
                block_tutorial_ambient(
                    &mut ambient,
                    &mut runtime,
                    "single-owner ambient entity started with the wrong cue".to_owned(),
                );
                return;
            }
            let Some(mut sink) = sink else {
                return;
            };
            sink.set_volume(Volume::Linear(0.0));
            ambient.reset_transition_clock();
            ambient.phase = TutorialAmbientPhase::FadingIn;
        }
        TutorialAmbientPhase::FadingIn => {
            let Some(entity) = ambient.current_entity else {
                block_tutorial_ambient(
                    &mut ambient,
                    &mut runtime,
                    "single-owner ambient entity disappeared during fade-in".to_owned(),
                );
                return;
            };
            let Ok((marker, sink)) = audio.get_mut(entity) else {
                block_tutorial_ambient(
                    &mut ambient,
                    &mut runtime,
                    "single-owner ambient entity disappeared during fade-in".to_owned(),
                );
                return;
            };
            if Some(marker.cue) != ambient.current_cue {
                block_tutorial_ambient(
                    &mut ambient,
                    &mut runtime,
                    "single-owner ambient entity changed cue during fade-in".to_owned(),
                );
                return;
            }
            let Some(mut sink) = sink else {
                return;
            };
            if advance_tutorial_ambient_fade(&mut ambient, time.delta_secs()) {
                sink.set_volume(Volume::Linear(tutorial_ambient_fade_volume(
                    true,
                    ambient.fade_step,
                )));
            }
            if ambient.fade_step == TUTORIAL_AMBIENT_FADE_STEPS {
                ambient.target_cue = ambient.current_cue;
                ambient.phase = TutorialAmbientPhase::Stable;
            }
        }
    }
}
