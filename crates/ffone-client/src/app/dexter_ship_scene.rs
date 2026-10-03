//! Dexter ship cutscene components, constants, audio tables and spawning.

use super::asset_residency::SharedTutorialEffectLibrary;
use super::loading_screen::{GameplayLoadingState, ResourceLoadingScope};
use super::state::ClientState;
use super::{DexterHologramMaterial, DexterHologramUniform};
use bevy::{
    audio::Volume,
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
    render::render_resource::TextureFormat,
    text::LineHeight,
    world_serialization::WorldInstanceReady,
};
use ffone_client::{
    coordinates::unity_to_native_vector,
    gameplay_audio::{GameplaySfxAudio, RetrobutionAudioMix},
    localization::{
        Language, Localization, LocalizedText, LocalizedVoice, localized_tutorial_scene_text,
    },
    network_world_runtime::{
        NetworkNpcVisualCatalog0104, NetworkNpcVisualCatalogState0104,
        NetworkNpcVisualDefinition0104, NpcSceneTextureOverrides0104,
    },
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    tutorial_effects_runtime::TutorialEffectMaterialAnimation,
    tutorial_mission_content::TutorialMissionContent,
    tutorial_voice_subtitles::{
        TUTORIAL_SKIP_LABEL, TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH,
        TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH,
    },
};

#[derive(Debug, Resource)]
pub(super) struct DexterShipCutsceneRuntime {
    pub(super) elapsed: f32,
    pub(super) fired_audio: u64,
    pub(super) hologram_random_state: u32,
    pub(super) hologram_color_flag: bool,
    pub(super) presentation_ready: bool,
    pub(super) audio_started: bool,
    pub(super) tutorial_world_requested: bool,
}

impl Default for DexterShipCutsceneRuntime {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            fired_audio: 0,
            hologram_random_state: 0xd37e_2510,
            hologram_color_flag: false,
            presentation_ready: false,
            audio_started: false,
            tutorial_world_requested: false,
        }
    }
}

#[derive(Component)]
pub(super) struct DexterShipEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DexterShipActorRole {
    Dexter,
    DeeDee,
    Computress,
}

impl DexterShipActorRole {
    pub(super) const ALL: [Self; 3] = [Self::Dexter, Self::DeeDee, Self::Computress];

    pub(super) const fn npc_type(self) -> i32 {
        match self {
            Self::Dexter => 728,
            Self::DeeDee => 701,
            Self::Computress => 730,
        }
    }

    pub(super) fn definition(
        self,
        catalog: &NetworkNpcVisualCatalog0104,
    ) -> Result<&NetworkNpcVisualDefinition0104, String> {
        catalog.get(self.npc_type()).ok_or_else(|| {
            format!(
                "cutscene actor {self:?} NPC {} has no validated table model route",
                self.npc_type()
            )
        })
    }
}

#[derive(Component)]
pub(super) struct DexterShipActor {
    pub(super) role: DexterShipActorRole,
    pub(super) gltf: Handle<Gltf>,
    pub(super) clip: &'static str,
    pub(super) revision: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DexterShipAnimationApplied(pub(super) u32);

#[derive(Component)]
pub(super) struct DexterShipSceneRoot;

#[derive(Component)]
pub(super) struct DexterShipSceneReady;

#[derive(Component)]
pub(super) struct DexterShipCamera;

#[derive(Component)]
pub(super) struct DexterShipSubtitleSpeaker;

#[derive(Component)]
pub(super) struct DexterShipSubtitleDialogue;

#[derive(Component)]
pub(super) struct DexterShipTitle;

#[derive(Component)]
pub(super) struct DexterShipBlackOverlay;

#[derive(Component)]
pub(super) struct DexterShipBackground;

#[derive(Component)]
pub(super) struct DexterShipHologram;

#[derive(Component)]
pub(super) struct DexterShipComputerEffect {
    pub(super) gltf: Handle<Gltf>,
    pub(super) revision: u32,
}

#[derive(Component)]
pub(super) struct DexterShipAudioLoop;

pub(super) const DEXTER_SHIP_BACKGROUND_PATH: &str = "ui/en/character/creation/background/CharCreationBG.png";
pub(super) const DEXTER_SHIP_HOLOGRAM_DECAL_PATH: &str = "ui/en/character/creation/background/holo_line.png";
pub(super) const DEXTER_SHIP_RENDER_TEXTURE_SIZE: u32 = 512;
pub(super) const DEXTER_SHIP_BLACK_BAR_HEIGHT_PERCENT: f32 = 15.673_982;
pub(super) const DEXTER_SHIP_BACKGROUND_REFERENCE_WIDTH: f32 = 1_920.0;
pub(super) const DEXTER_SHIP_BACKGROUND_REFERENCE_HEIGHT: f32 = 1_440.0;
pub(super) const DEXTER_SHIP_TITLE_SECONDS_PER_CHARACTER: f32 = 0.05;
pub(super) const DEXTER_SHIP_NAME_REVEAL_END_SECONDS: f32 = 2.0;
pub(super) const DEXTER_SHIP_NAME_FADE_END_SECONDS: f32 = 3.0;
pub(super) const DEXTER_SHIP_COMPUTER_HIDE_SECONDS: f32 = 27.4;
pub(super) const DEXTER_SHIP_COMPUTER_ANIMATION_SECONDS: f32 = 3.333_331_6;
pub(super) const DEXTER_SHIP_RENDER_TARGET_CLEAR: Color =
    Color::srgba(0.192_156_87, 0.301_960_8, 0.474_509_8, 0.0);
pub(super) const DEXTER_SHIP_COMPUTER_EFFECT_PATH: &str = "map/shared/effects/models/es740/m_inven.glb";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DexterShipAudioAction {
    OneShot,
    ReplaceLoop,
    AddLoop,
    StopLoops,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DexterShipAudioCue {
    pub(super) seconds: f32,
    pub(super) true_name: &'static str,
    pub(super) action: DexterShipAudioAction,
}

pub(super) const DEXTER_SHIP_NAME_AUDIO: &[DexterShipAudioCue] = &[
    DexterShipAudioCue {
        seconds: 0.0,
        true_name: "DexlabsText_Typed",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 2.0,
        true_name: "HologramOn",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 2.0,
        true_name: "Machine_Running",
        action: DexterShipAudioAction::ReplaceLoop,
    },
    DexterShipAudioCue {
        seconds: 2.0,
        true_name: "Dexter_CCHello",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 6.5,
        true_name: "Dexter_CCInput",
        action: DexterShipAudioAction::OneShot,
    },
];

pub(super) const DEXTER_SHIP_TUTORIAL_AUDIO: &[DexterShipAudioCue] = &[
    DexterShipAudioCue {
        seconds: 0.0,
        true_name: "HologramOn",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 0.0,
        true_name: "Machine_Running",
        action: DexterShipAudioAction::ReplaceLoop,
    },
    DexterShipAudioCue {
        seconds: 0.0,
        true_name: "Dexter_CC01",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 0.0,
        true_name: "Beeping",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 5.5,
        true_name: "Computress_CC01",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 6.5,
        true_name: "Beeping",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 11.8,
        true_name: "Dexter_CCEnginePwr",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 11.8,
        true_name: "Beeping",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 12.3,
        true_name: "DeeDee_Laugh_01",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 15.8,
        true_name: "Engine",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 15.8,
        true_name: "Vibration1",
        action: DexterShipAudioAction::ReplaceLoop,
    },
    DexterShipAudioCue {
        seconds: 15.8,
        true_name: "DeeDee_Laugh02",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 18.8,
        true_name: "Dexter_CCReadings",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 20.3,
        true_name: "DeeDee_Laugh_03",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 23.3,
        true_name: "DeeDee_Laugh_01",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 23.8,
        true_name: "Dexter_CCLaunch",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 24.9,
        true_name: "DeeDee_CCButton",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 26.9,
        true_name: "Button",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 27.4,
        true_name: "Vibration2",
        action: DexterShipAudioAction::ReplaceLoop,
    },
    DexterShipAudioCue {
        seconds: 27.4,
        true_name: "Warning_Siren",
        action: DexterShipAudioAction::AddLoop,
    },
    DexterShipAudioCue {
        seconds: 27.4,
        true_name: "Dexter_CCDeeDeeNo",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 30.9,
        true_name: "Thunderous_Crash",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 32.5,
        true_name: "",
        action: DexterShipAudioAction::StopLoops,
    },
    DexterShipAudioCue {
        seconds: 33.0,
        true_name: "HologramOff",
        action: DexterShipAudioAction::OneShot,
    },
    DexterShipAudioCue {
        seconds: 33.0,
        true_name: "TimejumpText_Typed",
        action: DexterShipAudioAction::OneShot,
    },
];

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_dexter_ship_cutscene(
    mut commands: Commands,
    state: Res<State<ClientState>>,
    mut runtime: ResMut<DexterShipCutsceneRuntime>,
    asset_server: Res<AssetServer>,
    npc_catalog: Res<NetworkNpcVisualCatalogState0104>,
    mut loading: ResMut<GameplayLoadingState>,
    effect_library: Option<Res<SharedTutorialEffectLibrary>>,
    mut images: ResMut<Assets<Image>>,
    mut hologram_materials: ResMut<Assets<DexterHologramMaterial>>,
    cutscene_entities: Query<(), With<DexterShipEntity>>,
) {
    if !dexter_ship_spawn_ready(
        *state.get(),
        effect_library.is_some(),
        !cutscene_entities.is_empty(),
    ) {
        return;
    }
    let effect_library = effect_library.expect("spawn readiness requires the effect library");
    let Some(npc_catalog) = npc_catalog.catalog.as_ref() else {
        return;
    };
    for role in DexterShipActorRole::ALL {
        if let Err(error) = role.definition(npc_catalog) {
            loading.block(error);
            return;
        }
    }
    *runtime = DexterShipCutsceneRuntime::default();
    loading.begin(ResourceLoadingScope::Cutscene);
    let name_scene = *state.get() == ClientState::CharacterCreateIntro;
    let chalet_font: Handle<Font> = asset_server.load(TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH);
    let jeffe_font: Handle<Font> = asset_server.load(TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH);
    let render_target = images.add(Image::new_target_texture(
        DEXTER_SHIP_RENDER_TEXTURE_SIZE,
        DEXTER_SHIP_RENDER_TEXTURE_SIZE,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    let hologram = hologram_materials.add(DexterHologramMaterial {
        uniform: DexterHologramUniform {
            color: Vec4::new(1.0, 1.0, 1.0, 0.8),
            offset: Vec4::ZERO,
        },
        scene_texture: render_target.clone(),
        decal_texture: asset_server.load(DEXTER_SHIP_HOLOGRAM_DECAL_PATH),
    });

    spawn_dexter_ship_actor(
        &mut commands,
        &asset_server,
        DexterShipActorRole::Dexter,
        DexterShipActorRole::Dexter.definition(npc_catalog).unwrap(),
        Vec3::ZERO,
        dexter_ship_dexter_clip(name_scene, 0.0),
    );
    spawn_dexter_ship_actor(
        &mut commands,
        &asset_server,
        DexterShipActorRole::DeeDee,
        DexterShipActorRole::DeeDee.definition(npc_catalog).unwrap(),
        Vec3::new(5.0, -0.8, -2.0),
        "stand1",
    );
    spawn_dexter_ship_actor(
        &mut commands,
        &asset_server,
        DexterShipActorRole::Computress,
        DexterShipActorRole::Computress
            .definition(npc_catalog)
            .unwrap(),
        Vec3::new(0.5, -0.2, -0.5),
        "stand1",
    );
    if !name_scene {
        let material_animation = effect_library
            .0
            .effect_material_animation(740)
            .expect("validated ES740 closure must provide its exact material animation");
        spawn_dexter_ship_computer_effect(&mut commands, &asset_server, material_animation);
    }

    let target = Vec3::Y;
    commands.spawn((
        Name::new("Dexter ship event camera"),
        DexterShipEntity,
        DexterShipCamera,
        Camera3d::default(),
        (
            Camera {
                order: -1,

                // Exact authored camera clear: the RGB survives additive legacy
                // passes, while alpha remains zero outside opaque actors.
                clear_color: ClearColorConfig::Custom(DEXTER_SHIP_RENDER_TARGET_CLEAR),
                ..default()
            },
            bevy::camera::RenderTarget::from(render_target.clone()),
        ),
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0_f32.to_radians(),
            near: 0.001,
            far: 1000.0,
            ..default()
        }),
        Transform::from_xyz(0.0, 1.0, 2.0).looking_at(target, Vec3::Y),
    ));
    // GameplayUiPlugin owns the single default UI camera for the whole
    // application. A second IsDefaultUiCamera made the cutscene root choose a
    // camera nondeterministically, which intermittently hid its background and
    // material pass.
    commands.spawn((
        Name::new("Dexter ship event light"),
        DexterShipEntity,
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands
        .spawn((
            Name::new("Dexter ship event overlay"),
            DexterShipEntity,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ZIndex(100),
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("Dexter ship exact charcreationBG"),
                DexterShipBackground,
                ImageNode::new(asset_server.load(DEXTER_SHIP_BACKGROUND_PATH))
                    .with_mode(NodeImageMode::Stretch),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(DEXTER_SHIP_BACKGROUND_REFERENCE_WIDTH),
                    height: Val::Px(DEXTER_SHIP_BACKGROUND_REFERENCE_HEIGHT),
                    margin: UiRect {
                        left: Val::Px(-DEXTER_SHIP_BACKGROUND_REFERENCE_WIDTH * 0.5),
                        top: Val::Px(-DEXTER_SHIP_BACKGROUND_REFERENCE_HEIGHT * 0.5),
                        ..default()
                    },
                    ..default()
                },
            ));
            root.spawn((
                Name::new("Dexter ship exact 512x512 hologram pass"),
                DexterShipHologram,
                MaterialNode(hologram),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(DEXTER_SHIP_RENDER_TEXTURE_SIZE as f32),
                    height: Val::Px(DEXTER_SHIP_RENDER_TEXTURE_SIZE as f32),
                    margin: UiRect {
                        left: Val::Px(-(DEXTER_SHIP_RENDER_TEXTURE_SIZE as f32) * 0.5),
                        top: Val::Px(-(DEXTER_SHIP_RENDER_TEXTURE_SIZE as f32) * 0.5),
                        ..default()
                    },
                    ..default()
                },
                if name_scene {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                },
            ));
            root.spawn((
                DexterShipBlackOverlay,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
            for top in [true, false] {
                root.spawn((
                    Name::new(if top {
                        "Dexter ship top black bar"
                    } else {
                        "Dexter ship bottom black bar"
                    }),
                    Node {
                        position_type: PositionType::Absolute,
                        top: top.then_some(Val::Px(0.0)).unwrap_or(Val::Auto),
                        bottom: (!top).then_some(Val::Px(0.0)).unwrap_or(Val::Auto),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(DEXTER_SHIP_BLACK_BAR_HEIGHT_PERCENT),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ));
            }
            root.spawn((
                Name::new("Dexter ship skip label"),
                Text::new(TUTORIAL_SKIP_LABEL),
                LocalizedText::new("ui.tutorial.skip", TUTORIAL_SKIP_LABEL),
                (
                    TextFont {
                        font: (chalet_font.clone()).into(),
                        font_size: (12.0).into(),
                        ..default()
                    },
                    LineHeight::Px(12.072),
                ),
                TextColor(Color::srgb(1.0, 0.995_967_75, 1.0)),
                TextLayout::default().with_justify(Justify::Center),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                    ..default()
                },
            ));
            root.spawn((
                DexterShipTitle,
                Text::new(""),
                dexter_ship_runtime_copy(""),
                TextFont {
                    font: (jeffe_font.clone()).into(),
                    font_size: (16.0).into(),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::default().with_justify(Justify::Left),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(50.0),
                    bottom: Val::Percent(DEXTER_SHIP_BLACK_BAR_HEIGHT_PERCENT),
                    width: Val::Px(300.0),
                    height: Val::Px(100.0),
                    ..default()
                },
            ));
            root.spawn((
                Name::new("Dexter ship subtitle area"),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Percent(19.778_48),
                    width: Val::Percent(60.443_04),
                    height: Val::Percent(DEXTER_SHIP_BLACK_BAR_HEIGHT_PERCENT),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|subtitle| {
                subtitle.spawn((
                    DexterShipSubtitleSpeaker,
                    Text::new(""),
                    dexter_ship_runtime_copy(""),
                    (
                        TextFont {
                            font: (jeffe_font).into(),
                            font_size: (12.0).into(),
                            ..default()
                        },
                        LineHeight::Px(12.072),
                    ),
                    TextColor(Color::srgb(1.0, 0.995_967_75, 1.0)),
                    TextLayout::default().with_justify(Justify::Center),
                    Node {
                        align_self: AlignSelf::Stretch,
                        flex_shrink: 0.0,
                        margin: UiRect::all(Val::Px(4.0)),
                        padding: UiRect::new(
                            Val::Px(6.0),
                            Val::Px(6.0),
                            Val::Px(2.0),
                            Val::Px(0.0),
                        ),
                        ..default()
                    },
                ));
                subtitle.spawn((
                    DexterShipSubtitleDialogue,
                    Text::new(""),
                    dexter_ship_runtime_copy(""),
                    (
                        TextFont {
                            font: (chalet_font).into(),
                            font_size: (14.0).into(),
                            ..default()
                        },
                        LineHeight::Px(14.084),
                    ),
                    TextColor(Color::srgb(1.0, 0.995_967_75, 1.0)),
                    TextLayout::default().with_justify(Justify::Center),
                    Node {
                        align_self: AlignSelf::Stretch,
                        flex_shrink: 0.0,
                        margin: UiRect::all(Val::Px(4.0)),
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(0.0)),
                        ..default()
                    },
                ));
            });
        });
}

pub(super) fn dexter_ship_spawn_ready(
    state: ClientState,
    effect_library_ready: bool,
    cutscene_exists: bool,
) -> bool {
    matches!(
        state,
        ClientState::CharacterCreateIntro | ClientState::TutorialIntro
    ) && effect_library_ready
        && !cutscene_exists
}

pub(super) fn spawn_dexter_ship_actor(
    commands: &mut Commands,
    asset_server: &AssetServer,
    role: DexterShipActorRole,
    definition: &NetworkNpcVisualDefinition0104,
    unity_position: Vec3,
    clip: &'static str,
) {
    let gltf = asset_server.load(definition.glb.clone());
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(definition.glb.clone()));
    let root = commands
        .spawn((
            Name::new(format!("Dexter ship {role:?}")),
            DexterShipEntity,
            NpcSceneTextureOverrides0104::from(definition),
            DexterShipActor {
                role,
                gltf,
                clip,
                revision: 1,
            },
            Transform::from_translation(unity_to_native_vector(unity_position)),
            // AssetServer readiness precedes scene instantiation and the first
            // animation evaluation. Keep the complete actor opaque until the
            // cutscene presentation gate has observed both.
            Visibility::Hidden,
        ))
        .id();
    commands
        .spawn((
            DexterShipSceneRoot,
            ChildOf(root),
            WorldAssetRoot(scene),
            // `cnDexterShipEventScene` instantiates the authored +Z-forward model
            // directly and places its camera on +Z.  Unlike gameplay actor roots,
            // this isolated cinematic root must not receive the Bevy-forward
            // half-turn a second time.
            Transform::from_scale(Vec3::splat(definition.table_scale)),
            Visibility::Inherited,
        ))
        .observe(mark_dexter_ship_scene_ready);
}

/// Instantiates the complete ES[740] prefab at Dexter's authored position.
///
/// Retrobution's `cnDexterShipEventScene` uses `InstantiateEffect(740,
/// Dexter.transform.position, Dexter.transform.rotation)`. The recovered GLB
/// retains all 26 mesh renderers, legacy material passes, hierarchy and the
/// `nif-default` animation, so no runtime reconstruction or camera-facing
/// approximation is required.
pub(super) fn spawn_dexter_ship_computer_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    material_animation: TutorialEffectMaterialAnimation,
) {
    let gltf = asset_server.load(DEXTER_SHIP_COMPUTER_EFFECT_PATH);
    let scene =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(DEXTER_SHIP_COMPUTER_EFFECT_PATH));
    let root = commands
        .spawn((
            Name::new("Dexter ship exact ES740 inventory computer"),
            DexterShipEntity,
            DexterShipComputerEffect { gltf, revision: 1 },
            material_animation.paused(),
            Transform::IDENTITY,
            Visibility::Hidden,
        ))
        .id();
    commands
        .spawn((
            DexterShipSceneRoot,
            ChildOf(root),
            WorldAssetRoot(scene),
            Transform::IDENTITY,
            Visibility::Inherited,
        ))
        .observe(mark_dexter_ship_scene_ready);
}

pub(super) fn mark_dexter_ship_scene_ready(event: On<WorldInstanceReady>, mut commands: Commands) {
    commands
        .entity(event.event().entity)
        .insert(DexterShipSceneReady);
}

pub(super) fn dexter_ship_audio_path(
    catalog: &NativeAudioCatalog,
    language: &str,
    true_name: &str,
) -> Option<(String, NativeAudioCategory)> {
    let asset = catalog
        .character_creation_assets()
        .find(|asset| asset.true_name.eq_ignore_ascii_case(true_name))
        .or_else(|| catalog.by_true_name(true_name).into_iter().next())?;
    let path = if asset.category == NativeAudioCategory::Voice {
        catalog.path_for_locale(asset, language)?.to_owned()
    } else {
        asset.path.clone()
    };
    Some((path, asset.category))
}

pub(super) fn spawn_dexter_ship_audio(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    language: &str,
    mix: &RetrobutionAudioMix,
    true_name: &str,
    looping: bool,
    mark_effect_loop: bool,
) {
    let Some((path, category)) = dexter_ship_audio_path(catalog, language, true_name) else {
        warn!("exact Dexter ship audio {true_name:?} is absent from the semantic catalog");
        return;
    };
    let gain = match category {
        NativeAudioCategory::Music => mix.music,
        NativeAudioCategory::Ambient => mix.ambient,
        NativeAudioCategory::Voice => mix.voice,
        NativeAudioCategory::Sfx => mix.effects,
    }
    .clamp(0.0, 1.0);
    let mut entity = commands.spawn((
        Name::new(format!("Dexter ship audio {true_name}")),
        DexterShipEntity,
        AudioPlayer::new(asset_server.load(path)),
        if looping {
            PlaybackSettings::LOOP.with_volume(Volume::Linear(gain))
        } else {
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain))
        },
    ));
    if category == NativeAudioCategory::Voice {
        entity.insert(LocalizedVoice::by_true_name(true_name));
    } else if category == NativeAudioCategory::Sfx {
        entity.insert(GameplaySfxAudio);
    }
    if mark_effect_loop {
        entity.insert(DexterShipAudioLoop);
    }
}

pub(super) fn set_dexter_ship_actor_clip(actor: &mut DexterShipActor, clip: &'static str) {
    if actor.clip != clip {
        actor.clip = clip;
        actor.revision = actor.revision.wrapping_add(1).max(1);
    }
}

/// Named event clips from the current character-creation sequence. The scene
/// owns their timing; model and texture selection comes from the NPC table.
pub(super) fn dexter_ship_dexter_clip(name_scene: bool, elapsed: f32) -> &'static str {
    if name_scene {
        if elapsed < 2.0 {
            "stand1"
        } else if elapsed < 16.5 {
            "observe"
        } else {
            "cutscene_stand_1"
        }
    } else if elapsed < 27.4 {
        "excellent"
    } else if elapsed < 28.9 {
        "deedeeno"
    } else {
        "busyrun"
    }
}

pub(super) fn dexter_ship_scene_text(
    content: &TutorialMissionContent,
    localization: &Localization,
    language: &Language,
    line: i32,
) -> String {
    let fallback = content
        .scene_text(1, line)
        .map(str::to_owned)
        .unwrap_or_else(|_| format!("Missing original cut-scene line 1:{line}"));
    localization.text(language, &localized_tutorial_scene_text(1, line, &fallback))
}

pub(super) fn dexter_ship_runtime_copy(text: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", text)
}
