//! Gameplay loading screen: legacy layout constants, tips, loading state and its UI.

use super::tutorial_startup::TutorialStartupProbe;
use bevy::{
    prelude::*,
    sprite::{SliceScaleMode, TextureSlicer},
    text::{FontSmoothing, LineHeight},
    window::PrimaryWindow,
};
use ffone_client::{
    localization::LocalizedText,
    movement::advance_native_xorshift32,
    option_ui::{OPTION_CHALET_FONT_PATH, OPTION_JEFFE_FONT_PATH},
};

pub(super) const LEGACY_LOADING_BACKGROUND_PATH: &str = "ui/en/gameplay/loading/load.png";
pub(super) const LEGACY_LOADING_WINDOW_PATH: &str = "ui/en/gameplay/loading/loadingwindow.png";
pub(super) const LEGACY_LOADING_BAR_PATH: &str = "ui/en/gameplay/loading/loadingbar.png";
pub(super) const LEGACY_LOADING_BACKGROUND_WIDTH: f32 = 1_920.0;
pub(super) const LEGACY_LOADING_BACKGROUND_HEIGHT: f32 = 1_012.0;
pub(super) const LEGACY_LOADING_WINDOW_WIDTH: f32 = 509.0;
pub(super) const LEGACY_LOADING_WINDOW_HEIGHT: f32 = 309.0;
pub(super) const LEGACY_LOADING_BAR_LEFT: f32 = 30.0;
pub(super) const LEGACY_LOADING_BAR_WIDTH: f32 = 448.0;
pub(super) const LEGACY_LOADING_BAR_HEIGHT: f32 = 17.0;
pub(super) const LEGACY_LOADING_CURRENT_BAR_BOTTOM: f32 = 75.0;
pub(super) const LEGACY_LOADING_OVERALL_BAR_BOTTOM: f32 = 35.0;
pub(super) const LEGACY_LOADING_JEFFE_FONT_SIZE: f32 = 14.0;
pub(super) const LEGACY_LOADING_JEFFE_LINE_HEIGHT: f32 = 13.71;
pub(super) const LEGACY_LOADING_CHALET_FONT_SIZE: f32 = 14.0;
pub(super) const LEGACY_LOADING_CHALET_LINE_HEIGHT: f32 = 14.084;
pub(super) const GAMEPLAY_LOADING_CAMERA_ORDER: isize = 10_000;
/// Reveal the prepared presentation below the opaque loader for two complete
/// render extractions before opening it to the player. Three Update visits are
/// required because the first visit performs the reveal itself.
pub(super) const GAMEPLAY_LOADING_RENDER_SETTLE_FRAMES: u8 = 3;

pub(super) const GAMEPLAY_LOADING_TIPS: [(&str, &str); 18] = [
    (
        "ui.loading.tip.01",
        "Always keep an eye on your active Nano's stamina. If your Nano gets tired, it will need to rest for a little while.",
    ),
    (
        "ui.loading.tip.02",
        "Keyboard shortcuts are useful! Press \"I\" to access your pack, \"P\" to access your mail and \"M\" to access the map.",
    ),
    (
        "ui.loading.tip.03",
        "Remember: shirts, pants and shoes are worn for protection. Items for your head, face and back are worn for fun!",
    ),
    (
        "ui.loading.tip.04",
        "Buy Power Items from Dexbot shopkeepers if you want to change your Nano's active power.",
    ),
    (
        "ui.loading.tip.05",
        "There are four different kinds of C.R.A.T.E.s. Standard, Special, Sooper and Sooper Dooper.",
    ),
    (
        "ui.loading.tip.06",
        "Defeating Fuse's monsters isn't the only way to collect Fusion Matter! Try running a race inside an Infected Zone.",
    ),
    (
        "ui.loading.tip.07",
        "Feed gumballs to your Nanos to make them temporarily more powerful!",
    ),
    (
        "ui.loading.tip.08",
        "Not sure where to go next? Check your email. Your guide will often send you helpful messages.",
    ),
    (
        "ui.loading.tip.09",
        "Remember: Adaptium beats Blastons, Blastons beat Cosmix, and Cosmix beats Adaptium.",
    ),
    (
        "ui.loading.tip.10",
        "If your Nano is low on stamina, hit the \"C\" key to use a Nano Potion.",
    ),
    (
        "ui.loading.tip.11",
        "If you want to swap out your Nanos or change their powers, visit a Nano Station!",
    ),
    (
        "ui.loading.tip.12",
        "Selling tip: click the Right Mouse button when selecting an item from your pack to automatically sell it to a shopkeeper.",
    ),
    (
        "ui.loading.tip.13",
        "Got too many items in your pack? Stop at Morbucks Savings and Loan to store your extra gear.",
    ),
    (
        "ui.loading.tip.14",
        "There are six different kinds of weapons: pistols, rifles, shatterguns, rocket, melee and thrown.",
    ),
    (
        "ui.loading.tip.15",
        "Ouch! Remember to watch out for toxic Fusion Matter patches on the ground.",
    ),
    (
        "ui.loading.tip.16",
        "Having trouble with the mouse? Use the arrow keys to move around instead. Press the \"Z\" key to attack and the \"X\" key to use a Nano power.",
    ),
    (
        "ui.loading.tip.17",
        "Need help finding your friends? Add your friends to your buddy list, then you can teleport to them using the Buddy Warp command.",
    ),
    (
        "ui.loading.tip.18",
        "Once you have acquired your Eduardo Nano, you will be ready to use the time machine to return to the past.",
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResourceLoadingScope {
    Login,
    CharacterSelection,
    CharacterCreation,
    Cutscene,
    Tutorial,
    World,
}

impl ResourceLoadingScope {
    pub(super) fn localized_step(self) -> LocalizedText {
        match self {
            Self::Login => LocalizedText::new("ui.loading.login", "LOADING LOGIN ASSETS"),
            Self::CharacterSelection => LocalizedText::new(
                "ui.loading.character_selection",
                "LOADING CHARACTER SELECTION ASSETS",
            ),
            Self::CharacterCreation => LocalizedText::new(
                "ui.loading.character_creation",
                "LOADING CHARACTER CREATION ASSETS",
            ),
            Self::Cutscene => LocalizedText::new("ui.loading.cutscene", "LOADING CUTSCENE ASSETS"),
            Self::Tutorial => LocalizedText::new("ui.loading.tutorial", "LOADING TUTORIAL ASSETS"),
            Self::World => LocalizedText::new("ui.loading.world", "LOADING WORLD AREA"),
        }
    }

    pub(super) fn localized_current_resource(self) -> LocalizedText {
        match self {
            Self::Login => {
                LocalizedText::new("ui.loading.current.login", "DOWNLOADING Login Presentation")
            }
            Self::CharacterSelection => LocalizedText::new(
                "ui.loading.current.character_selection",
                "DOWNLOADING Character Selection Data",
            ),
            Self::CharacterCreation => LocalizedText::new(
                "ui.loading.current.character_creation",
                "DOWNLOADING Character Gear",
            ),
            Self::Cutscene => {
                LocalizedText::new("ui.loading.current.cutscene", "DOWNLOADING Cutscene Data")
            }
            Self::Tutorial => {
                LocalizedText::new("ui.loading.current.tutorial", "DOWNLOADING Tutorial Data")
            }
            Self::World => LocalizedText::new("ui.loading.current.world", "DOWNLOADING World Data"),
        }
    }
}

#[derive(Debug, Resource)]
pub(super) struct GameplayLoadingState {
    pub(super) last_startup_probe: Option<TutorialStartupProbe>,
    pub(super) visible: bool,
    pub(super) scope: Option<ResourceLoadingScope>,
    pub(super) overall_progress: f32,
    pub(super) phase_progress: f32,
    pub(super) phase: GameplayLoadingPhase,
    pub(super) blocked: Option<String>,
    pub(super) render_settle_frames: u8,
    pub(super) tip_random_state: u32,
    pub(super) tip_index: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum GameplayLoadingPhase {
    #[default]
    Assets,
    SceneAssembly,
    PresentationBinding,
    RenderExtraction,
}

impl Default for GameplayLoadingState {
    fn default() -> Self {
        Self {
            last_startup_probe: None,
            visible: false,
            scope: None,
            overall_progress: 0.0,
            phase_progress: 0.0,
            phase: GameplayLoadingPhase::Assets,
            blocked: None,
            render_settle_frames: 0,
            tip_random_state: 0x4c4f_4144,
            tip_index: 0,
        }
    }
}

impl GameplayLoadingState {
    pub(super) fn begin(&mut self, scope: ResourceLoadingScope) {
        self.last_startup_probe = None;
        self.visible = true;
        self.scope = Some(scope);
        self.overall_progress = 0.0;
        self.phase_progress = 0.0;
        self.phase = GameplayLoadingPhase::Assets;
        self.blocked = None;
        self.render_settle_frames = 0;
        // CnGuiLoadingScreenTips calls Random.Range(0, arrTips.Length - 1).
        // Unity's integer upper bound is exclusive, so preserve the source
        // quirk that leaves the final line unreachable.
        self.tip_index = advance_native_xorshift32(&mut self.tip_random_state) as usize
            % GAMEPLAY_LOADING_TIPS.len().saturating_sub(1).max(1);
    }

    pub(super) fn loading(&mut self, overall_progress: f32, phase_progress: f32) {
        self.visible = true;
        // Aggregate readiness can be reported in a slightly different order
        // by wgpu on adjacent frames. The legacy overall bar never moved
        // backwards, while the current-package bar restarted for each phase.
        self.overall_progress = self.overall_progress.max(overall_progress.clamp(0.0, 0.99));
        self.phase_progress = phase_progress.clamp(0.0, 0.99);
        self.phase = GameplayLoadingPhase::Assets;
        self.blocked = None;
        self.render_settle_frames = 0;
    }

    pub(super) fn prepare(&mut self, phase: GameplayLoadingPhase, overall_progress: f32, phase_progress: f32) {
        self.visible = true;
        self.overall_progress = self.overall_progress.max(overall_progress.clamp(0.0, 0.99));
        self.phase_progress = phase_progress.clamp(0.0, 0.99);
        self.phase = phase;
        self.blocked = None;
        self.render_settle_frames = 0;
    }

    pub(super) fn settle_render_presentation(&mut self) -> bool {
        self.visible = true;
        self.overall_progress = self.overall_progress.max(0.99);
        self.phase_progress = self.phase_progress.max(0.99);
        self.phase = GameplayLoadingPhase::RenderExtraction;
        self.blocked = None;
        self.render_settle_frames = self.render_settle_frames.saturating_add(1);
        self.render_settle_frames >= GAMEPLAY_LOADING_RENDER_SETTLE_FRAMES
    }

    pub(super) fn block(&mut self, error: String) {
        self.visible = true;
        self.blocked = Some(error);
        self.render_settle_frames = 0;
    }

    pub(super) fn finish(&mut self) {
        self.overall_progress = 1.0;
        self.phase_progress = 1.0;
        self.phase = GameplayLoadingPhase::Assets;
        self.blocked = None;
        self.render_settle_frames = 0;
        self.visible = false;
        self.scope = None;
    }

    pub(super) fn localized_tip(&self) -> LocalizedText {
        let (key, fallback) =
            GAMEPLAY_LOADING_TIPS[self.tip_index.min(GAMEPLAY_LOADING_TIPS.len() - 1)];
        LocalizedText::new(key, fallback)
    }

    pub(super) fn localized_current_resource(&self) -> LocalizedText {
        if let Some(error) = self.blocked.as_deref() {
            return LocalizedText::new("ui.loading.error", "LOADING STOPPED: {error}")
                .with_arg("error", error);
        }
        match self.phase {
            GameplayLoadingPhase::Assets => {}
            GameplayLoadingPhase::SceneAssembly => {
                return LocalizedText::new("ui.loading.phase.scene", "ASSEMBLING Scene");
            }
            GameplayLoadingPhase::PresentationBinding => {
                return LocalizedText::new(
                    "ui.loading.phase.presentation",
                    "PREPARING Characters and Animations",
                );
            }
            GameplayLoadingPhase::RenderExtraction => {
                return LocalizedText::new("ui.loading.phase.render", "PREPARING First Frame");
            }
        }
        self.scope.map_or_else(
            || LocalizedText::new("ui.loading.overall", "DOWNLOADING Required Resources"),
            ResourceLoadingScope::localized_current_resource,
        )
    }

    pub(super) fn localized_step(&self) -> LocalizedText {
        self.scope.map_or_else(
            || LocalizedText::new("ui.loading.world", "LOADING WORLD AREA"),
            ResourceLoadingScope::localized_step,
        )
    }
}

#[derive(Component)]
pub(super) struct GameplayLoadingCamera;

#[derive(Component)]
pub(super) struct GameplayLoadingRoot;

#[derive(Component)]
pub(super) struct GameplayLoadingBackground;

#[derive(Clone, Copy, Component)]
pub(super) enum GameplayLoadingBar {
    Overall,
    Phase,
}

#[derive(Clone, Copy, Component)]
pub(super) enum GameplayLoadingTextRole {
    Tip,
    CurrentResource,
    Step,
}

pub(super) fn spawn_gameplay_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // AssetLoaderSkin pathId 1377 assigns JEFFE___14 (pathId 903) to the
    // ordinary Label style and ChaletBook-Regular (pathId 1115) to GameTips.
    // Use the approved Cyrillic-capable native replacements for those two
    // distinct source font roles.
    let label_font = asset_server.load(OPTION_JEFFE_FONT_PATH);
    let tip_font = asset_server.load(OPTION_CHALET_FONT_PATH);
    // Character preview and modal cameras intentionally render above the
    // normal HUD camera. Loading is a presentation barrier, so it owns a
    // dedicated final camera and cannot be painted over by a stale preview.
    let camera = commands
        .spawn((
            Name::new("Retrobution gameplay loading UI camera"),
            GameplayLoadingCamera,
            Camera2d,
            Camera {
                order: GAMEPLAY_LOADING_CAMERA_ORDER,
                clear_color: ClearColorConfig::None,
                ..default()
            },
        ))
        .id();
    commands
        .spawn((
            Name::new("Retrobution gameplay loading screen"),
            GameplayLoadingRoot,
            UiTargetCamera(camera),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            Pickable::default(),
            ZIndex(1_000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("Exact Retrobution load.png ScaleAndCrop"),
                GameplayLoadingBackground,
                ImageNode::new(asset_server.load(LEGACY_LOADING_BACKGROUND_PATH)),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(LEGACY_LOADING_BACKGROUND_WIDTH),
                    height: Val::Px(LEGACY_LOADING_BACKGROUND_HEIGHT),
                    ..default()
                },
            ));
            root.spawn((
                Name::new("Exact Retrobution loading window"),
                ImageNode {
                    image: asset_server.load(LEGACY_LOADING_WINDOW_PATH),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: [4.0, 4.0, 3.0, 3.0].into(),
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 1.0,
                    }),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(LEGACY_LOADING_WINDOW_WIDTH),
                    height: Val::Px(LEGACY_LOADING_WINDOW_HEIGHT),
                    ..default()
                },
            ))
            .with_children(|window| {
                window.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(120.0),
                    bottom: Val::Px(135.0),
                    width: Val::Px(350.0),
                    height: Val::Px(112.0),
                    padding: UiRect::right(Val::Px(15.0)),
                    align_items: AlignItems::Center,
                    ..default()
                }).with_child((
                    Name::new("Retrobution loading tip"),
                    GameplayLoadingTextRole::Tip,
                    Text::new(GAMEPLAY_LOADING_TIPS[0].1),
                    LocalizedText::new(GAMEPLAY_LOADING_TIPS[0].0, GAMEPLAY_LOADING_TIPS[0].1),
                    (
                        TextFont {
                            font: (tip_font.clone()).into(),
                            font_size: (LEGACY_LOADING_CHALET_FONT_SIZE).into(),
                            font_smoothing: FontSmoothing::AntiAliased,
                            ..default()
                        },
                        LineHeight::Px(LEGACY_LOADING_CHALET_LINE_HEIGHT),
                    ),
                    TextColor(Color::srgb(0.109_803_92, 0.211_764_71, 0.392_156_87)),
                    TextLayout::default().with_justify(Justify::Left),
                    Node {
                        width: Val::Percent(100.0),
                        ..default()
                    },
                ));
                window.spawn((
                    GameplayLoadingTextRole::CurrentResource,
                    Text::new("DOWNLOADING Character Gear"),
                    LocalizedText::new(
                        "ui.loading.current.character_creation",
                        "DOWNLOADING Character Gear",
                    ),
                    (
                        TextFont {
                            font: (label_font.clone()).into(),
                            font_size: (LEGACY_LOADING_JEFFE_FONT_SIZE).into(),
                            font_smoothing: FontSmoothing::AntiAliased,
                            ..default()
                        },
                        LineHeight::Px(LEGACY_LOADING_JEFFE_LINE_HEIGHT),
                    ),
                    TextColor(Color::srgb(0.717_647_1, 0.854_901_97, 0.866_666_7)),
                    TextLayout::default().with_justify(Justify::Center),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(29.0),
                        bottom: Val::Px(94.0),
                        width: Val::Px(450.0),
                        height: Val::Px(19.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ));
                for (kind, name, bottom) in [
                    (
                        GameplayLoadingBar::Phase,
                        "Retrobution current package bar",
                        LEGACY_LOADING_CURRENT_BAR_BOTTOM,
                    ),
                    (
                        GameplayLoadingBar::Overall,
                        "Retrobution overall package bar",
                        LEGACY_LOADING_OVERALL_BAR_BOTTOM,
                    ),
                ] {
                    window
                        .spawn((
                            Name::new(name),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(LEGACY_LOADING_BAR_LEFT),
                                bottom: Val::Px(bottom),
                                width: Val::Px(LEGACY_LOADING_BAR_WIDTH),
                                height: Val::Px(LEGACY_LOADING_BAR_HEIGHT),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                        ))
                        .with_child((
                            Name::new("Exact nine-sliced Retrobution loading bar fill"),
                            kind,
                            ImageNode {
                                image: asset_server.load(LEGACY_LOADING_BAR_PATH),
                                image_mode: NodeImageMode::Sliced(TextureSlicer {
                                    border: [3.0, 15.0, 2.0, 2.0].into(),
                                    center_scale_mode: SliceScaleMode::Stretch,
                                    sides_scale_mode: SliceScaleMode::Stretch,
                                    max_corner_scale: 1.0,
                                }),
                                ..default()
                            },
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                bottom: Val::Px(0.0),
                                width: Val::Px(15.0),
                                height: Val::Px(LEGACY_LOADING_BAR_HEIGHT),
                                ..default()
                            },
                        ));
                }
                window.spawn((
                    GameplayLoadingTextRole::Step,
                    Text::new("LOADING CHARACTER CREATION ASSETS"),
                    LocalizedText::new(
                        "ui.loading.character_creation",
                        "LOADING CHARACTER CREATION ASSETS",
                    ),
                    (
                        TextFont {
                            font: (label_font.clone()).into(),
                            font_size: (LEGACY_LOADING_JEFFE_FONT_SIZE).into(),
                            font_smoothing: FontSmoothing::AntiAliased,
                            ..default()
                        },
                        LineHeight::Px(LEGACY_LOADING_JEFFE_LINE_HEIGHT),
                    ),
                    TextColor(Color::srgb(0.717_647_1, 0.854_901_97, 0.866_666_7)),
                    TextLayout::default().with_justify(Justify::Center),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(29.0),
                        bottom: Val::Px(54.0),
                        width: Val::Px(450.0),
                        height: Val::Px(19.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ));
            });
        });
}

pub(super) fn sync_gameplay_loading_screen(
    loading: Res<GameplayLoadingState>,
    mut audio: ResMut<ffone_client::world_audio::RetrobutionLoadingAudioState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<&mut Visibility, With<GameplayLoadingRoot>>,
    mut backgrounds: Query<
        &mut Node,
        (With<GameplayLoadingBackground>, Without<GameplayLoadingBar>),
    >,
    mut bars: Query<(&GameplayLoadingBar, &mut Node), Without<GameplayLoadingBackground>>,
    mut copy: Query<(&GameplayLoadingTextRole, &mut LocalizedText, &mut TextColor)>,
) {
    let visible = loading.visible;
    audio.active = visible;
    for mut visibility in &mut roots {
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if !visible {
        return;
    }
    if let Ok(window) = windows.single() {
        let scale = (window.width() / LEGACY_LOADING_BACKGROUND_WIDTH)
            .max(window.height() / LEGACY_LOADING_BACKGROUND_HEIGHT);
        let width = LEGACY_LOADING_BACKGROUND_WIDTH * scale;
        let height = LEGACY_LOADING_BACKGROUND_HEIGHT * scale;
        for mut node in &mut backgrounds {
            node.width = Val::Px(width);
            node.height = Val::Px(height);
            node.left = Val::Px((window.width() - width) * 0.5);
            node.top = Val::Px((window.height() - height) * 0.5);
        }
    }
    for (kind, mut node) in &mut bars {
        let progress = match kind {
            GameplayLoadingBar::Overall => loading.overall_progress,
            GameplayLoadingBar::Phase => loading.phase_progress,
        };
        node.width = Val::Px(15.0 + LEGACY_LOADING_BAR_WIDTH * progress.clamp(0.0, 1.0));
    }
    let tip = loading.localized_tip();
    let current = loading.localized_current_resource();
    let step = loading.localized_step();
    for (role, mut localized, mut color) in &mut copy {
        match role {
            GameplayLoadingTextRole::Tip => {
                localized.clone_from(&tip);
                color.0 = Color::srgb(0.109_803_92, 0.211_764_71, 0.392_156_87);
            }
            GameplayLoadingTextRole::CurrentResource => {
                localized.clone_from(&current);
                color.0 = if loading.blocked.is_some() {
                    Color::srgb(1.0, 0.35, 0.25)
                } else {
                    Color::srgb(0.717_647_1, 0.854_901_97, 0.866_666_7)
                };
            }
            GameplayLoadingTextRole::Step => {
                localized.clone_from(&step);
                color.0 = Color::srgb(0.717_647_1, 0.854_901_97, 0.866_666_7);
            }
        }
    }
}
