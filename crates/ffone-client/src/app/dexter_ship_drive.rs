//! Dexter ship cutscene drive, audio, animation and cleanup.

use super::DexterHologramMaterial;
use super::{GamepadActionState, LegacyOptionAction};
use super::asset_residency::{AssetResidency, AssetResidencyGroupId, resident_group_probe};
use super::dexter_ship_scene::{
    DEXTER_SHIP_BACKGROUND_REFERENCE_HEIGHT, DEXTER_SHIP_BACKGROUND_REFERENCE_WIDTH,
    DEXTER_SHIP_BLACK_BAR_HEIGHT_PERCENT, DEXTER_SHIP_COMPUTER_ANIMATION_SECONDS,
    DEXTER_SHIP_COMPUTER_HIDE_SECONDS, DEXTER_SHIP_NAME_AUDIO, DEXTER_SHIP_NAME_FADE_END_SECONDS,
    DEXTER_SHIP_NAME_REVEAL_END_SECONDS, DEXTER_SHIP_RENDER_TEXTURE_SIZE,
    DEXTER_SHIP_TITLE_SECONDS_PER_CHARACTER, DEXTER_SHIP_TUTORIAL_AUDIO, DexterShipActor,
    DexterShipActorRole, DexterShipAnimationApplied, DexterShipAudioAction, DexterShipAudioCue,
    DexterShipAudioLoop, DexterShipBackground, DexterShipBlackOverlay, DexterShipCamera,
    DexterShipComputerEffect, DexterShipCutsceneRuntime, DexterShipEntity, DexterShipHologram,
    DexterShipSceneReady, DexterShipSceneRoot, DexterShipSubtitleDialogue,
    DexterShipSubtitleSpeaker, DexterShipTitle, dexter_ship_dexter_clip, dexter_ship_runtime_copy,
    dexter_ship_scene_text, set_dexter_ship_actor_clip, spawn_dexter_ship_audio,
};
use super::loading_screen::{GameplayLoadingPhase, GameplayLoadingState, ResourceLoadingScope};
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_session::TutorialSession;
use super::tutorial_startup::{TUTORIAL_START_ANGLE, TUTORIAL_START_SERVER_POSITION};
use bevy::{
    animation::RepeatAnimation, ecs::system::SystemParam, gltf::Gltf, prelude::*,
    window::PrimaryWindow,
};
use ffone_client::{
    coordinates::{unity_to_native_rotation, unity_to_native_vector},
    gameplay_audio::RetrobutionAudioMix,
    legacy_model_material::{
        LegacyMaterialApplied, LegacyMaterialMetadataError, LegacyModelMaterial,
    },
    localization::{Language, Localization, LocalizedText, VoiceLanguage},
    movement::advance_native_xorshift32,
    network::{CharacterEntryLocation0104, NetworkBridge, NetworkCommand},
    network_world_runtime::{NetworkNpcTextureVariantBound0104, NetworkNpcVisualIssue0104},
    semantic_audio::NativeAudioCatalog,
    tutorial_effects_runtime::TutorialEffectMaterialAnimation,
    tutorial_mission_content::TutorialMissionContent,
};

#[derive(SystemParam)]
pub(super) struct DexterShipDriveQueries<'w, 's> {
    pub(super) material_surfaces: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static NetworkNpcTextureVariantBound0104>,
            &'static MeshMaterial3d<LegacyModelMaterial>,
        ),
        With<LegacyMaterialApplied>,
    >,
    pub(super) material_assets: Res<'w, Assets<LegacyModelMaterial>>,
    pub(super) image_assets: Res<'w, Assets<Image>>,
    pub(super) standard_surfaces: Query<'w, 's, Entity, With<MeshMaterial3d<StandardMaterial>>>,
    pub(super) material_issues: Query<'w, 's, &'static NetworkNpcVisualIssue0104, With<DexterShipActor>>,
    pub(super) material_errors: Query<'w, 's, (Entity, &'static LegacyMaterialMetadataError)>,
    pub(super) localization: Res<'w, Localization>,
    pub(super) language: Res<'w, Language>,
    pub(super) audio_mix: Res<'w, RetrobutionAudioMix>,
    pub(super) bridge: Res<'w, NetworkBridge>,
    pub(super) tutorial: Res<'w, TutorialSession>,
    pub(super) actors: Query<
        'w,
        's,
        (
            Entity,
            &'static mut DexterShipActor,
            &'static mut Transform,
            &'static mut Visibility,
        ),
        (
            With<DexterShipActor>,
            Without<DexterShipComputerEffect>,
            Without<DexterShipHologram>,
        ),
    >,
    pub(super) cameras: Query<
        'w,
        's,
        &'static mut Transform,
        (
            With<DexterShipCamera>,
            Without<DexterShipActor>,
            Without<DexterShipComputerEffect>,
        ),
    >,
    pub(super) computer_displays: Query<
        'w,
        's,
        (
            Entity,
            &'static DexterShipComputerEffect,
            &'static mut TutorialEffectMaterialAnimation,
            &'static mut Visibility,
        ),
        (
            With<DexterShipComputerEffect>,
            Without<DexterShipActor>,
            Without<DexterShipCamera>,
            Without<DexterShipHologram>,
        ),
    >,
    pub(super) scene_roots: Query<'w, 's, Option<&'static DexterShipSceneReady>, With<DexterShipSceneRoot>>,
    pub(super) animation_players:
        Query<'w, 's, (Entity, Option<&'static DexterShipAnimationApplied>), With<AnimationPlayer>>,
    pub(super) parents: Query<'w, 's, &'static ChildOf>,
    pub(super) subtitle_speakers: Query<
        'w,
        's,
        &'static mut LocalizedText,
        (
            With<DexterShipSubtitleSpeaker>,
            Without<DexterShipSubtitleDialogue>,
            Without<DexterShipTitle>,
        ),
    >,
    pub(super) subtitle_dialogues: Query<
        'w,
        's,
        &'static mut LocalizedText,
        (
            With<DexterShipSubtitleDialogue>,
            Without<DexterShipSubtitleSpeaker>,
            Without<DexterShipTitle>,
        ),
    >,
    pub(super) titles: Query<
        'w,
        's,
        (
            &'static mut LocalizedText,
            &'static mut Node,
            &'static mut TextColor,
            &'static mut TextFont,
            &'static mut TextLayout,
        ),
        (
            With<DexterShipTitle>,
            Without<DexterShipSubtitleSpeaker>,
            Without<DexterShipSubtitleDialogue>,
            Without<DexterShipBackground>,
            Without<DexterShipHologram>,
        ),
    >,
    pub(super) overlays: Query<'w, 's, &'static mut BackgroundColor, With<DexterShipBlackOverlay>>,
    pub(super) backgrounds: Query<
        'w,
        's,
        &'static mut Node,
        (
            With<DexterShipBackground>,
            Without<DexterShipHologram>,
            Without<DexterShipTitle>,
        ),
    >,
    pub(super) holograms: Query<
        'w,
        's,
        (
            &'static mut Node,
            &'static MaterialNode<DexterHologramMaterial>,
            &'static mut Visibility,
        ),
        (
            With<DexterShipHologram>,
            Without<DexterShipBackground>,
            Without<DexterShipTitle>,
        ),
    >,
    pub(super) loops: Query<'w, 's, Entity, With<DexterShipAudioLoop>>,
    pub(super) audio: Query<'w, 's, Entity, (With<DexterShipEntity>, With<AudioPlayer>)>,
}

pub(super) fn drive_dexter_ship_audio(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    language: &str,
    mix: &RetrobutionAudioMix,
    elapsed: f32,
    fired: &mut u64,
    cues: &[DexterShipAudioCue],
    loops: &Query<Entity, With<DexterShipAudioLoop>>,
) {
    for (index, cue) in cues.iter().enumerate() {
        let mask = 1_u64 << index;
        if elapsed < cue.seconds || *fired & mask != 0 {
            continue;
        }
        *fired |= mask;
        match cue.action {
            DexterShipAudioAction::OneShot => spawn_dexter_ship_audio(
                commands,
                asset_server,
                catalog,
                language,
                mix,
                cue.true_name,
                false,
                false,
            ),
            DexterShipAudioAction::ReplaceLoop => {
                for entity in loops {
                    commands.entity(entity).despawn();
                }
                spawn_dexter_ship_audio(
                    commands,
                    asset_server,
                    catalog,
                    language,
                    mix,
                    cue.true_name,
                    true,
                    true,
                );
            }
            DexterShipAudioAction::AddLoop => spawn_dexter_ship_audio(
                commands,
                asset_server,
                catalog,
                language,
                mix,
                cue.true_name,
                true,
                true,
            ),
            DexterShipAudioAction::StopLoops => {
                for entity in loops {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub(super) fn stop_dexter_ship_audio(
    commands: &mut Commands,
    audio: &Query<Entity, (With<DexterShipEntity>, With<AudioPlayer>)>,
) {
    for entity in audio {
        commands.entity(entity).despawn();
    }
}

pub(super) fn dexter_ship_random_range(state: &mut u32, minimum: f32, maximum: f32) -> f32 {
    let draw = advance_native_xorshift32(state);
    let unit = (f64::from(draw) / f64::from(u32::MAX)) as f32;
    minimum + (maximum - minimum) * unit
}

pub(super) fn dexter_ship_random_int(state: &mut u32, exclusive_maximum: u32) -> u32 {
    let draw = advance_native_xorshift32(state);
    ((u64::from(draw) * u64::from(exclusive_maximum)) >> 32) as u32
}

pub(super) fn dexter_ship_background_layout(viewport: Vec2) -> (Vec2, Vec2) {
    let viewport = if viewport.is_finite() && viewport.cmpgt(Vec2::ZERO).all() {
        viewport
    } else {
        Vec2::new(
            DEXTER_SHIP_BACKGROUND_REFERENCE_WIDTH,
            DEXTER_SHIP_BACKGROUND_REFERENCE_HEIGHT,
        )
    };
    // Retrobution draws a centered 1920x1440 rectangle and never shrinks it.
    // Scale it further only when a larger viewport would expose an edge.
    let scale = (viewport.x / DEXTER_SHIP_BACKGROUND_REFERENCE_WIDTH)
        .max(viewport.y / DEXTER_SHIP_BACKGROUND_REFERENCE_HEIGHT)
        .max(1.0);
    let size = Vec2::new(
        DEXTER_SHIP_BACKGROUND_REFERENCE_WIDTH,
        DEXTER_SHIP_BACKGROUND_REFERENCE_HEIGHT,
    ) * scale;
    ((viewport - size) * 0.5, size)
}

pub(super) fn dexter_ship_typewriter_text(text: &str, elapsed_seconds: f32) -> String {
    if text.is_empty() || !elapsed_seconds.is_finite() || elapsed_seconds < 0.0 {
        return String::new();
    }
    let visible = (elapsed_seconds / DEXTER_SHIP_TITLE_SECONDS_PER_CHARACTER).floor() as usize + 1;
    text.chars().take(visible).collect()
}

pub(super) fn dexter_ship_name_title_alpha(elapsed_seconds: f32) -> f32 {
    if !elapsed_seconds.is_finite() || elapsed_seconds < 0.0 {
        return 0.0;
    }
    if elapsed_seconds <= DEXTER_SHIP_NAME_REVEAL_END_SECONDS {
        1.0
    } else {
        ((DEXTER_SHIP_NAME_FADE_END_SECONDS - elapsed_seconds)
            / (DEXTER_SHIP_NAME_FADE_END_SECONDS - DEXTER_SHIP_NAME_REVEAL_END_SECONDS))
            .clamp(0.0, 1.0)
    }
}

pub(super) fn dexter_ship_presentation_ready(
    expected_scene_roots: usize,
    scene_roots: usize,
    ready_scene_roots: usize,
    expected_actors: usize,
    actors: usize,
    ready_actors: usize,
    expected_computer_effects: usize,
    computer_effects: usize,
    ready_computer_effects: usize,
) -> bool {
    scene_roots == expected_scene_roots
        && ready_scene_roots == expected_scene_roots
        && actors == expected_actors
        && ready_actors == expected_actors
        && computer_effects == expected_computer_effects
        && ready_computer_effects == expected_computer_effects
}

pub(super) fn dexter_ship_animation_revision_ready(
    root: Entity,
    revision: u32,
    parents: &Query<&ChildOf>,
    animation_players: &Query<(Entity, Option<&DexterShipAnimationApplied>), With<AnimationPlayer>>,
) -> bool {
    let mut descendants = animation_players
        .iter()
        .filter(|(entity, _)| dexter_ship_descendant(*entity, root, parents))
        .peekable();
    descendants.peek().is_some()
        && descendants.all(|(_, applied)| applied.is_some_and(|applied| applied.0 == revision))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_dexter_ship_cutscene(
    mut commands: Commands,
    time: Res<Time>,
    input: (Res<ButtonInput<KeyCode>>, Option<Res<GamepadActionState>>),
    state: Res<State<ClientState>>,
    mut cutscene: ResMut<DexterShipCutsceneRuntime>,
    residency: Res<AssetResidency>,
    mut loading: ResMut<GameplayLoadingState>,
    mut next_state: ResMut<NextState<ClientState>>,
    asset_server: Res<AssetServer>,
    catalog: Res<NativeAudioCatalog>,
    voice_language: Res<VoiceLanguage>,
    content: Res<TutorialMissionContent>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut hologram_materials: ResMut<Assets<DexterHologramMaterial>>,
    mut runtime_status: ResMut<RuntimeStatus>,
    mut scene: DexterShipDriveQueries,
) {
    let (keys, pad) = input;
    let localization: &Localization = &scene.localization;
    let language: &Language = &scene.language;
    let name_scene = match state.get() {
        ClientState::CharacterCreateIntro => true,
        ClientState::TutorialIntro => false,
        _ => return,
    };
    if cutscene.tutorial_world_requested {
        // Keep the last opaque frame while the shard prepares the shared world.
        return;
    }
    if !cutscene.presentation_ready {
        for (entity, error) in &scene.material_errors {
            if scene
                .actors
                .iter()
                .any(|(root, _, _, _)| dexter_ship_descendant(entity, root, &scene.parents))
            {
                loading.block(format!("cutscene material conversion failed: {}", error.0));
                return;
            }
        }
        if let Some(issue) = scene.material_issues.iter().next() {
            loading.block(issue.detail.clone());
            return;
        }
        let asset_probe =
            resident_group_probe(&asset_server, &residency, AssetResidencyGroupId::DexterShip);
        if let Some(error) = asset_probe.blocker.clone() {
            runtime_status.message = format!("Cutscene startup blocked: {error}");
            loading.block(error);
            return;
        }
        let progress = asset_probe.progress();
        if !asset_probe.is_ready() {
            loading.loading(progress * 0.6, progress);
            return;
        }
        let expected_computer_effects = usize::from(!name_scene);
        let expected_scene_roots = 3 + expected_computer_effects;
        let scene_roots = scene.scene_roots.iter().count();
        let ready_scene_roots = scene
            .scene_roots
            .iter()
            .filter(|ready| ready.is_some())
            .count();
        let actors = scene.actors.iter().count();
        let ready_actors = scene
            .actors
            .iter()
            .filter(|(root, actor, _, _)| {
                let mut surfaces = scene
                    .material_surfaces
                    .iter()
                    .filter(|(entity, _, _)| dexter_ship_descendant(*entity, *root, &scene.parents))
                    .peekable();
                let materials_ready = surfaces.peek().is_some()
                    && surfaces.all(|(_, bound, handle)| {
                        bound.is_some()
                            && scene
                                .material_assets
                                .get(&handle.0)
                                .is_some_and(|material| {
                                    material
                                        .base_texture
                                        .as_ref()
                                        .is_none_or(|image| scene.image_assets.contains(image.id()))
                                })
                    })
                    && !scene
                        .standard_surfaces
                        .iter()
                        .any(|entity| dexter_ship_descendant(entity, *root, &scene.parents));
                materials_ready
                    && dexter_ship_animation_revision_ready(
                        *root,
                        actor.revision,
                        &scene.parents,
                        &scene.animation_players,
                    )
            })
            .count();
        let computer_effects = scene.computer_displays.iter().count();
        let ready_computer_effects = scene
            .computer_displays
            .iter()
            .filter(|(root, effect, _, _)| {
                dexter_ship_animation_revision_ready(
                    *root,
                    effect.revision,
                    &scene.parents,
                    &scene.animation_players,
                )
            })
            .count();
        let presentation_total = expected_scene_roots + 3 + expected_computer_effects;
        let presentation_completed = ready_scene_roots + ready_actors + ready_computer_effects;
        let presentation_progress =
            presentation_completed as f32 / presentation_total.max(1) as f32;
        if !dexter_ship_presentation_ready(
            expected_scene_roots,
            scene_roots,
            ready_scene_roots,
            3,
            actors,
            ready_actors,
            expected_computer_effects,
            computer_effects,
            ready_computer_effects,
        ) {
            let phase = if ready_scene_roots != expected_scene_roots {
                GameplayLoadingPhase::SceneAssembly
            } else {
                GameplayLoadingPhase::PresentationBinding
            };
            loading.prepare(
                phase,
                0.6 + presentation_progress * 0.3,
                presentation_progress,
            );
            return;
        }
        // Reveal below the still-opaque loader so the render sub-app has a
        // complete preparation window with the exact first poses already live.
        for (_, _, _, mut visibility) in &mut scene.actors {
            *visibility = Visibility::Inherited;
        }
        for (_, _, mut material_animation, mut visibility) in &mut scene.computer_displays {
            material_animation.set_paused(true);
            *visibility = Visibility::Inherited;
        }
        if !loading.settle_render_presentation() {
            return;
        }
        loading.finish();
        cutscene.presentation_ready = true;
        for (_, _, mut material_animation, _) in &mut scene.computer_displays {
            material_animation.set_paused(false);
        }
    }
    if !cutscene.audio_started {
        spawn_dexter_ship_audio(
            &mut commands,
            &asset_server,
            &catalog,
            &voice_language.effective,
            &scene.audio_mix,
            "CharacterCreation_Loop",
            true,
            false,
        );
        cutscene.audio_started = true;
    }
    cutscene.elapsed += time.delta_secs();
    let elapsed = cutscene.elapsed;
    // Exact per-frame Update contract from cnDexterShipEventScene. The old
    // client flickers the hologram alpha and scrolls the MainTex vertically
    // with Unity RandomRange noise before drawing the 512x512 target.
    let hologram_alpha = if cutscene.hologram_color_flag {
        let noise = dexter_ship_random_range(&mut cutscene.hologram_random_state, -0.05, 0.05);
        0.8 * ((f32::sin(elapsed * std::f32::consts::PI)
            + f32::cos(elapsed * std::f32::consts::PI * 0.7)
            + noise)
            * 7.0)
            .abs()
            .min(1.0)
    } else {
        0.8
    };
    if dexter_ship_random_int(&mut cutscene.hologram_random_state, 10) == 1 {
        cutscene.hologram_color_flag = !cutscene.hologram_color_flag;
    }
    let hologram_offset_y = (f32::sin(elapsed * std::f32::consts::PI * 0.7)
        + f32::cos(elapsed * std::f32::consts::PI)
        + dexter_ship_random_range(&mut cutscene.hologram_random_state, -0.1, 0.1))
        * 5.0;
    let cues = if name_scene {
        DEXTER_SHIP_NAME_AUDIO
    } else {
        DEXTER_SHIP_TUTORIAL_AUDIO
    };
    drive_dexter_ship_audio(
        &mut commands,
        &asset_server,
        &catalog,
        &voice_language.effective,
        &scene.audio_mix,
        elapsed,
        &mut cutscene.fired_audio,
        cues,
        &scene.loops,
    );

    let subtitle_line = if name_scene {
        if elapsed < 2.0 {
            None
        } else if elapsed < 6.5 {
            Some(2)
        } else {
            Some(3)
        }
    } else if elapsed < 5.5 {
        Some(4)
    } else if elapsed < 11.8 {
        Some(5)
    } else if elapsed < 18.8 {
        Some(6)
    } else if elapsed < 23.8 {
        Some(7)
    } else if elapsed < 24.9 {
        Some(8)
    } else if elapsed < 27.4 {
        Some(9)
    } else if elapsed < 31.9 {
        Some(10)
    } else {
        None
    };
    let subtitle =
        subtitle_line.map(|line| dexter_ship_scene_text(&content, localization, language, line));
    let (speaker, dialogue) = subtitle
        .as_deref()
        .and_then(|line| line.split_once(':'))
        .map(|(name, dialogue)| (format!("{}:", name.trim()), dialogue.trim().to_owned()))
        .unwrap_or_else(|| (String::new(), subtitle.unwrap_or_default()));
    for mut localized in &mut scene.subtitle_speakers {
        *localized = dexter_ship_runtime_copy(speaker.clone());
    }
    for mut localized in &mut scene.subtitle_dialogues {
        *localized = dexter_ship_runtime_copy(dialogue.clone());
    }
    for (mut title, mut node, mut color, mut font, mut layout) in &mut scene.titles {
        let title_alpha = dexter_ship_name_title_alpha(elapsed);
        *title = if name_scene && title_alpha > 0.0 {
            dexter_ship_runtime_copy(dexter_ship_typewriter_text(
                &dexter_ship_scene_text(&content, localization, language, 1),
                elapsed,
            ))
        } else if !name_scene && elapsed >= 33.0 {
            LocalizedText::new("ui.tutorial.cutscene.time_jump_error", "TIME JUMP ERROR")
        } else {
            dexter_ship_runtime_copy("")
        };
        if !name_scene && elapsed >= 33.0 {
            node.left = Val::Percent(50.0);
            node.top = Val::Percent(50.0);
            node.bottom = Val::Auto;
            node.width = Val::Px(200.0);
            node.height = Val::Px(20.0);
            node.margin = UiRect::new(Val::Px(-100.0), Val::Px(0.0), Val::Px(-10.0), Val::Px(0.0));
            color.0 = Color::srgba(
                1.0,
                0.0,
                0.0,
                f32::sin(elapsed * std::f32::consts::PI * 2.0).abs(),
            );
            font.font_size = 14.0.into();
            layout.justify = Justify::Center;
        } else {
            node.left = Val::Px(50.0);
            node.top = Val::Auto;
            node.bottom = Val::Percent(DEXTER_SHIP_BLACK_BAR_HEIGHT_PERCENT);
            node.width = Val::Px(300.0);
            node.height = Val::Px(100.0);
            node.margin = UiRect::ZERO;
            color.0 = Color::srgba(1.0, 1.0, 1.0, title_alpha);
            font.font_size = 16.0.into();
            layout.justify = Justify::Left;
        }
    }

    let black_alpha = if name_scene {
        if elapsed <= DEXTER_SHIP_NAME_REVEAL_END_SECONDS {
            1.0
        } else {
            ((DEXTER_SHIP_NAME_FADE_END_SECONDS - elapsed)
                / (DEXTER_SHIP_NAME_FADE_END_SECONDS - DEXTER_SHIP_NAME_REVEAL_END_SECONDS))
                .clamp(0.0, 1.0)
        }
    } else {
        ((elapsed - 31.9) / 0.6).clamp(0.0, 1.0)
    };
    for mut overlay in &mut scene.overlays {
        overlay.0 = Color::srgba(0.0, 0.0, 0.0, black_alpha);
    }

    for (_, mut actor, mut transform, _) in &mut scene.actors {
        match actor.role {
            DexterShipActorRole::Dexter => {
                let clip = dexter_ship_dexter_clip(name_scene, elapsed);
                set_dexter_ship_actor_clip(&mut actor, clip);
                let run = ((elapsed - 28.9) / 1.0).clamp(0.0, 1.0);
                transform.translation = unity_to_native_vector(Vec3::new(-2.0 * run, 0.0, 0.0));
                transform.rotation = if elapsed >= 27.4 {
                    unity_to_native_rotation(Quat::from_rotation_y(-70.0_f32.to_radians()))
                } else {
                    Quat::IDENTITY
                };
            }
            DexterShipActorRole::Computress => {
                let clip = if !name_scene && elapsed >= 27.4 {
                    "stand2"
                } else if !name_scene && elapsed >= 10.5 {
                    "event2"
                } else if !name_scene && elapsed >= 5.5 {
                    "event1"
                } else {
                    "stand1"
                };
                set_dexter_ship_actor_clip(&mut actor, clip);
                transform.translation = unity_to_native_vector(Vec3::new(0.5, -0.2, -0.5));
                transform.rotation = Quat::IDENTITY;
            }
            DexterShipActorRole::DeeDee => {
                let mut position = Vec3::new(5.0, -0.8, -2.0);
                let mut rotation = Quat::IDENTITY;
                let mut clip = "stand1";
                if !name_scene && (12.3..14.3).contains(&elapsed) {
                    let phase = ((elapsed - 12.3) / 2.0).clamp(0.0, 1.0);
                    let factor = (f32::sin(phase * std::f32::consts::PI) + 0.5).min(1.0);
                    position = Vec3::new(-4.1, 0.5, -2.0) + Vec3::X * 1.6 * factor;
                    rotation = Quat::from_rotation_z(-70.0_f32.to_radians());
                } else if !name_scene && (14.3..15.8).contains(&elapsed) {
                    position = Vec3::new(4.0, 0.5, -2.0);
                } else if !name_scene && (15.8..17.8).contains(&elapsed) {
                    let phase = ((elapsed - 15.8) / 2.0).clamp(0.0, 1.0);
                    let factor = (f32::sin(phase * std::f32::consts::PI) + 0.5).min(1.0);
                    position = Vec3::new(0.0, 4.0 - 0.8 * factor, -2.0);
                    rotation = Quat::from_rotation_z(std::f32::consts::PI);
                } else if !name_scene && (17.8..20.3).contains(&elapsed) {
                    position = Vec3::new(4.0, 0.5, -2.0);
                } else if !name_scene && (20.3..22.3).contains(&elapsed) {
                    let phase = ((elapsed - 20.3) / 2.0).clamp(0.0, 1.0);
                    let factor = (f32::sin(phase * std::f32::consts::PI) + 0.5).min(1.0);
                    position = Vec3::new(4.0 - 1.4 * factor, 0.5, -2.0);
                    rotation = Quat::from_rotation_z(70.0_f32.to_radians());
                } else if !name_scene && (22.3..24.3).contains(&elapsed) {
                    let phase = ((elapsed - 22.3) / 2.0).clamp(0.0, 1.0);
                    position = Vec3::new(2.0 - 4.0 * phase, -0.8, -2.0);
                    rotation = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
                    clip = "walk";
                } else if !name_scene && elapsed >= 24.3 {
                    position = Vec3::new(-2.0, -0.8, -2.0);
                }
                set_dexter_ship_actor_clip(&mut actor, clip);
                transform.translation = unity_to_native_vector(position);
                transform.rotation = unity_to_native_rotation(rotation);
            }
        }
    }

    let mut screen_shake = Vec2::ZERO;
    if !name_scene && (15.8..32.5).contains(&elapsed) {
        let amount = if elapsed >= 29.9 {
            3.0
        } else if elapsed >= 27.4 {
            1.5
        } else {
            0.5
        };
        screen_shake.x = (f32::sin(elapsed * std::f32::consts::PI * 7.0)
            + f32::cos(elapsed * std::f32::consts::PI * 9.0))
            + dexter_ship_random_range(&mut cutscene.hologram_random_state, -0.2, 0.2);
        screen_shake.x *= amount * 15.0;
        screen_shake.y = (f32::sin(elapsed * std::f32::consts::PI * 3.0)
            + f32::cos(elapsed * std::f32::consts::PI * 5.0))
            + dexter_ship_random_range(&mut cutscene.hologram_random_state, -0.3, 0.3);
        screen_shake.y *= amount * 10.0;
    }
    for mut camera in &mut scene.cameras {
        camera.translation = Vec3::new(0.0, 1.0, 2.0);
        camera.look_at(Vec3::Y, Vec3::Y);
    }
    for (_, _, _, mut visibility) in &mut scene.computer_displays {
        *visibility = if !name_scene && elapsed < DEXTER_SHIP_COMPUTER_HIDE_SECONDS {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let viewport = windows
        .single()
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::new(
            DEXTER_SHIP_BACKGROUND_REFERENCE_WIDTH,
            DEXTER_SHIP_BACKGROUND_REFERENCE_HEIGHT,
        ));
    let (background_origin, background_size) = dexter_ship_background_layout(viewport);
    for mut background in &mut scene.backgrounds {
        background.left = Val::Px(background_origin.x + screen_shake.x);
        background.top = Val::Px(background_origin.y + screen_shake.y);
        background.width = Val::Px(background_size.x);
        background.height = Val::Px(background_size.y);
        background.margin = UiRect::ZERO;
    }
    let hologram_visible = if name_scene {
        elapsed >= 2.0
    } else {
        elapsed < 32.5
    };
    for (mut node, material, mut visibility) in &mut scene.holograms {
        node.margin.left =
            Val::Px(-(DEXTER_SHIP_RENDER_TEXTURE_SIZE as f32) * 0.5 + screen_shake.x);
        node.margin.top = Val::Px(
            -(DEXTER_SHIP_RENDER_TEXTURE_SIZE as f32) * 0.5 + screen_shake.y - hologram_offset_y,
        );
        *visibility = if hologram_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Some(mut material) = hologram_materials.get_mut(&material.0) {
            material.uniform.color = Vec4::new(1.0, 1.0, 1.0, hologram_alpha);
            material.uniform.offset = Vec4::new(
                0.0,
                hologram_offset_y / DEXTER_SHIP_RENDER_TEXTURE_SIZE as f32,
                0.0,
                0.0,
            );
        }
    }

    let duration = if name_scene { 18.5 } else { 36.0 };
    // ConfigurableInput key 5 is the original SPACE cutscene skip action.
    // Keep Escape as a native emergency exit, but do not require it for the
    // authored prompt and timing contract.
    if keys.just_pressed(KeyCode::Space)
        || pad.as_ref().is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Jump))
        || keys.just_pressed(KeyCode::Escape)
        || elapsed >= duration
    {
        // Primary `SkipEventScene` calls `VoiceOff` before leaving the scene.
        // Tutorial entry is now asynchronous, so OnExit(TutorialIntro) can be
        // delayed by the shard; stop every scene-owned source immediately and
        // clear the matching subtitle presentation at the same completion edge.
        stop_dexter_ship_audio(&mut commands, &scene.audio);
        cutscene.audio_started = false;
        for mut localized in &mut scene.subtitle_speakers {
            *localized = dexter_ship_runtime_copy("");
        }
        for mut localized in &mut scene.subtitle_dialogues {
            *localized = dexter_ship_runtime_copy("");
        }

        if name_scene {
            runtime_status.message =
                "Dexter introduction complete; opening character creation".to_owned();
            next_state.set(ClientState::CharacterCreate);
            return;
        }

        match request_tutorial_world_after_cutscene(
            &scene.tutorial,
            &scene.bridge,
            &mut runtime_status,
            &mut loading,
            &mut cutscene,
        ) {
            Ok(()) => {
                runtime_status.message =
                    "Time-capsule sequence complete; entering the shared gameplay world".to_owned();
            }
            Err(error) => {
                runtime_status.message = format!("Tutorial world entry failed: {error}");
                loading.block(error);
                next_state.set(ClientState::CharacterSelect);
            }
        }
    }
}

pub(super) fn tutorial_world_entry_command(tutorial: &TutorialSession) -> Result<NetworkCommand, String> {
    let character = tutorial
        .character()
        .ok_or_else(|| "Tutorial sequence finished without a selected character".to_owned())?;
    Ok(NetworkCommand::SelectCharacter {
        pc_uid: character.pc_uid,
        location: CharacterEntryLocation0104::Scripted {
            position: TUTORIAL_START_SERVER_POSITION,
            angle: TUTORIAL_START_ANGLE,
        },
    })
}

pub(super) fn request_tutorial_world_after_cutscene(
    tutorial: &TutorialSession,
    bridge: &NetworkBridge,
    runtime: &mut RuntimeStatus,
    loading: &mut GameplayLoadingState,
    cutscene: &mut DexterShipCutsceneRuntime,
) -> Result<(), String> {
    let command = tutorial_world_entry_command(tutorial)?;
    let NetworkCommand::SelectCharacter { pc_uid, .. } = command else {
        unreachable!("tutorial entry only builds SelectCharacter")
    };
    runtime.roster.pending_character_entry_uid = Some(pc_uid);
    if let Err(error) = bridge.send(command) {
        runtime.roster.pending_character_entry_uid = None;
        return Err(error);
    }
    cutscene.tutorial_world_requested = true;
    loading.begin(ResourceLoadingScope::Tutorial);
    Ok(())
}

pub(super) fn apply_dexter_ship_animations(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut loading: ResMut<GameplayLoadingState>,
    cutscene: Res<DexterShipCutsceneRuntime>,
    parents: Query<&ChildOf>,
    actors: Query<(Entity, &DexterShipActor)>,
    mut computer_effects: Query<(Entity, &mut DexterShipComputerEffect)>,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&DexterShipAnimationApplied>,
    )>,
) {
    for (actor_root, actor) in &actors {
        let Some(gltf) = gltfs.get(&actor.gltf) else {
            continue;
        };
        let Some(clip) = gltf.named_animations.get(actor.clip).cloned() else {
            loading.block(format!(
                "cutscene actor {:?} NPC {} has no animation {:?} in its table-selected model",
                actor.role,
                actor.role.npc_type(),
                actor.clip
            ));
            continue;
        };
        for (entity, mut player, applied) in &mut players {
            if !dexter_ship_descendant(entity, actor_root, &parents) {
                continue;
            }
            if applied.is_some_and(|applied| applied.0 == actor.revision) {
                if cutscene.presentation_ready {
                    player.resume_all();
                } else {
                    player.pause_all();
                }
                continue;
            }
            let (graph, node) = AnimationGraph::from_clip(clip.clone());
            player.stop_all();
            // These are complete authored performances, not idle cycles. In
            // particular observe ends before its 16.5s scene cue; looping it
            // briefly snaps Dexter back to the opening pose before the cut.
            let repeat = if actor.role == DexterShipActorRole::Dexter
                && matches!(actor.clip, "observe" | "excellent" | "deedeeno")
            {
                RepeatAnimation::Never
            } else {
                RepeatAnimation::Forever
            };
            player.start(node).set_repeat(repeat);
            if !cutscene.presentation_ready {
                player.pause_all();
            }
            commands.entity(entity).insert((
                AnimationGraphHandle(graphs.add(graph)),
                DexterShipAnimationApplied(actor.revision),
            ));
        }
    }
    for (effect_root, effect) in &mut computer_effects {
        let Some(gltf) = gltfs.get(&effect.gltf) else {
            continue;
        };
        let Some(clip) = gltf.named_animations.get("nif-default").cloned() else {
            continue;
        };
        for (entity, mut player, applied) in &mut players {
            if !dexter_ship_descendant(entity, effect_root, &parents) {
                continue;
            }
            if applied.is_some_and(|applied| applied.0 == effect.revision) {
                if cutscene.presentation_ready {
                    player.resume_all();
                } else {
                    player.pause_all();
                }
                continue;
            }
            let (graph, node) = AnimationGraph::from_clip(clip.clone());
            player.stop_all();
            player
                .start(node)
                .set_repeat(RepeatAnimation::Forever)
                .set_seek_time(
                    cutscene
                        .elapsed
                        .rem_euclid(DEXTER_SHIP_COMPUTER_ANIMATION_SECONDS),
                );
            if !cutscene.presentation_ready {
                player.pause_all();
            }
            commands.entity(entity).insert((
                AnimationGraphHandle(graphs.add(graph)),
                DexterShipAnimationApplied(effect.revision),
            ));
        }
    }
}

pub(super) use ffone_client::scene_hierarchy::is_descendant_of as dexter_ship_descendant;

pub(super) fn cleanup_dexter_ship_cutscene(
    mut commands: Commands,
    mut runtime: ResMut<DexterShipCutsceneRuntime>,
    entities: Query<Entity, With<DexterShipEntity>>,
) {
    for entity in &entities {
        // One-shot cutscene audio uses PlaybackSettings::DESPAWN and can
        // retire itself in the same frame as TutorialIntro exits.  The query
        // observes the source entity before deferred commands are applied, so
        // a plain despawn races that audio teardown and reports a stale-entity
        // command at the tutorial ownership boundary.  The cutscene is already
        // leaving; silently discard only that redundant cleanup command.
        commands
            .entity(entity)
            .queue_silenced(|entity: EntityWorldMut| {
                entity.despawn();
            });
    }
    *runtime = DexterShipCutsceneRuntime::default();
}
