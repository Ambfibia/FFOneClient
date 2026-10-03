//! Retrobution tutorial-only subtitle, illustration, and animated cursor overlay.
//!
//! This is deliberately independent from `GameplayUiPlugin`: the gameplay
//! HUD owns persistent player/minimap/chat chrome, while `cntutorialscript` owns
//! these transient instructions and pointers.

use crate::{
    gameplay_ui::GameplayUiRect,
    localization::{LocalizationSet, LocalizedText, UiTextAutoFit},
    mission_ui::{MissionUiModel, gameplay_chrome_visible},
};
use bevy::{prelude::*, ui::widget::NodeImageMode, window::PrimaryWindow};

pub const TUTORIAL_MOUSE_RECT: GameplayUiRect = GameplayUiRect::new(80.0, 370.0, 116.0, 154.0);
pub const TUTORIAL_RIGHT_RECT: GameplayUiRect = GameplayUiRect::new(186.0, 415.0, 88.0, 69.0);
pub const TUTORIAL_TEXT_RECT: GameplayUiRect = GameplayUiRect::new(0.0, 180.0, 1280.0, 60.0);
pub const TUTORIAL_INSTRUCTION_FONT_PATH: &str = "fonts/jeffe.otf";
pub const TUTORIAL_INSTRUCTION_FONT_SIZE: f32 = 16.0;

const TUTORIAL_ARROW_FREQUENCY: f32 = std::f32::consts::PI * 4.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TutorialIllustration {
    Mouse,
    LeftMouse,
    RightMouse,
    Move,
    MoveBackward,
    MoveForward,
    Jump,
    NanoOne,
}

impl TutorialIllustration {
    pub const ALL: [Self; 8] = [
        Self::Mouse,
        Self::LeftMouse,
        Self::RightMouse,
        Self::Move,
        Self::MoveBackward,
        Self::MoveForward,
        Self::Jump,
        Self::NanoOne,
    ];

    pub const fn asset_path(self) -> &'static str {
        match self {
            Self::Mouse => "ui/en/gameplay/tutorial/tut_mouse.png",
            Self::LeftMouse => "ui/en/gameplay/tutorial/tut_lmouse.png",
            Self::RightMouse => "ui/en/gameplay/tutorial/tut_rmouse.png",
            Self::Move => "ui/en/gameplay/tutorial/tut_move.png",
            Self::MoveBackward => "ui/en/gameplay/tutorial/tut_move_s.png",
            Self::MoveForward => "ui/en/gameplay/tutorial/tut_move_w.png",
            Self::Jump => "ui/en/gameplay/tutorial/tut_jump.png",
            Self::NanoOne => "ui/en/gameplay/tutorial/tut_one.png",
        }
    }

    pub const fn size(self) -> (f32, f32) {
        match self {
            Self::Mouse | Self::LeftMouse | Self::RightMouse => (116.0, 154.0),
            Self::Move | Self::MoveBackward | Self::MoveForward => (182.0, 131.0),
            Self::Jump => (298.0, 91.0),
            Self::NanoOne => (88.0, 91.0),
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Mouse => 0,
            Self::LeftMouse => 1,
            Self::RightMouse => 2,
            Self::Move => 3,
            Self::MoveBackward => 4,
            Self::MoveForward => 5,
            Self::Jump => 6,
            Self::NanoOne => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TutorialArrowDirection {
    Left,
    Right,
    Up,
    Down,
}

impl TutorialArrowDirection {
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Up, Self::Down];

    pub const fn asset_path(self) -> &'static str {
        match self {
            Self::Left => "ui/en/gameplay/tutorial/tut_left.png",
            Self::Right => "ui/en/gameplay/tutorial/tut_right.png",
            Self::Up => "ui/en/gameplay/tutorial/tut_up.png",
            Self::Down => "ui/en/gameplay/tutorial/tut_down.png",
        }
    }

    pub const fn size(self) -> (f32, f32) {
        match self {
            Self::Left => (90.0, 69.0),
            Self::Right => (88.0, 69.0),
            Self::Up | Self::Down => (69.0, 89.0),
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Up => 2,
            Self::Down => 3,
        }
    }
}

/// Exact pivot passed to `FFGUIUtility.ScaleAroundPivot` before a tutorial
/// picture or cursor is drawn. Tutorial cues do not share one global pivot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TutorialCueScalePivot {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    Center,
    CenterTop,
    Point(Vec2),
}

impl TutorialCueScalePivot {
    #[must_use]
    pub const fn from_legacy(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::TopRight),
            1 => Some(Self::TopLeft),
            2 => Some(Self::BottomRight),
            3 => Some(Self::BottomLeft),
            4 => Some(Self::Center),
            5 => Some(Self::CenterTop),
            _ => None,
        }
    }

    fn screen_point(self, viewport: Vec2) -> Vec2 {
        match self {
            Self::TopRight => Vec2::new(viewport.x, 0.0),
            Self::TopLeft => Vec2::ZERO,
            Self::BottomRight => viewport,
            Self::BottomLeft => Vec2::new(0.0, viewport.y),
            Self::Center => viewport * 0.5,
            Self::CenterTop => Vec2::new(viewport.x * 0.5, 0.0),
            Self::Point(point) => point,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TutorialCuePosition {
    TopLeft { left: f32, top: f32 },
    BottomLeft { left: f32, bottom: f32 },
}

impl TutorialCuePosition {
    const fn left(self) -> f32 {
        match self {
            Self::TopLeft { left, .. } | Self::BottomLeft { left, .. } => left,
        }
    }

    const fn top(self, screen_height: f32, image_height: f32) -> f32 {
        match self {
            Self::TopLeft { top, .. } => top,
            Self::BottomLeft { bottom, .. } => screen_height - bottom - image_height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialIllustrationCue {
    pub illustration: TutorialIllustration,
    pub position: TutorialCuePosition,
    pub scale_pivot: TutorialCueScalePivot,
}

impl TutorialIllustrationCue {
    pub const fn at(illustration: TutorialIllustration, left: f32, top: f32) -> Self {
        Self {
            illustration,
            position: TutorialCuePosition::TopLeft { left, top },
            scale_pivot: TutorialCueScalePivot::TopLeft,
        }
    }

    pub const fn from_bottom(illustration: TutorialIllustration, left: f32, bottom: f32) -> Self {
        Self {
            illustration,
            position: TutorialCuePosition::BottomLeft { left, bottom },
            scale_pivot: TutorialCueScalePivot::BottomLeft,
        }
    }

    pub const fn mouse() -> Self {
        Self::from_bottom(TutorialIllustration::Mouse, 80.0, 196.0)
    }

    pub const fn left_mouse() -> Self {
        Self::from_bottom(TutorialIllustration::LeftMouse, 80.0, 196.0)
    }

    pub const fn right_mouse() -> Self {
        Self::from_bottom(TutorialIllustration::RightMouse, 80.0, 196.0)
    }

    pub const fn move_keys() -> Self {
        Self::from_bottom(TutorialIllustration::Move, 80.0, 199.0)
    }

    pub const fn move_backward() -> Self {
        Self::from_bottom(TutorialIllustration::MoveBackward, 80.0, 199.0)
    }

    pub const fn move_forward() -> Self {
        Self::from_bottom(TutorialIllustration::MoveForward, 80.0, 199.0)
    }

    pub const fn jump() -> Self {
        Self::from_bottom(TutorialIllustration::Jump, 0.0, 199.0)
    }

    pub const fn nano_one() -> Self {
        Self::from_bottom(TutorialIllustration::NanoOne, 80.0, 159.0)
    }

    pub const fn reference_rect(self, screen_height: f32) -> GameplayUiRect {
        let (width, height) = self.illustration.size();
        GameplayUiRect::new(
            self.position.left(),
            self.position.top(screen_height, height),
            width,
            height,
        )
    }

    #[must_use]
    pub const fn with_scale_pivot(mut self, scale_pivot: TutorialCueScalePivot) -> Self {
        self.scale_pivot = scale_pivot;
        self
    }

    pub(crate) fn render_rect(self, viewport: Vec2, ui_scale: f32) -> GameplayUiRect {
        scale_tutorial_cue_rect(
            self.reference_rect(viewport.y),
            self.scale_pivot,
            viewport,
            ui_scale,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialArrowCue {
    pub direction: TutorialArrowDirection,
    pub position: TutorialCuePosition,
    pub scale_pivot: TutorialCueScalePivot,
}

impl TutorialArrowCue {
    pub const fn at(direction: TutorialArrowDirection, left: f32, top: f32) -> Self {
        Self {
            direction,
            position: TutorialCuePosition::TopLeft { left, top },
            scale_pivot: TutorialCueScalePivot::TopLeft,
        }
    }

    pub const fn from_bottom(direction: TutorialArrowDirection, left: f32, bottom: f32) -> Self {
        Self {
            direction,
            position: TutorialCuePosition::BottomLeft { left, bottom },
            scale_pivot: TutorialCueScalePivot::BottomLeft,
        }
    }

    pub const fn mouse(direction: TutorialArrowDirection) -> Self {
        match direction {
            TutorialArrowDirection::Left => Self::from_bottom(direction, 0.0, 236.0),
            TutorialArrowDirection::Right => Self::from_bottom(direction, 186.0, 236.0),
            TutorialArrowDirection::Up => Self::from_bottom(direction, 107.0, 331.0),
            TutorialArrowDirection::Down => Self::from_bottom(direction, 107.0, 111.0),
        }
    }

    pub fn reference_rect(self, screen_height: f32, elapsed_secs: f32) -> GameplayUiRect {
        let offset = tutorial_arrow_offset(elapsed_secs);
        let position = animated_tutorial_arrow_position(self, offset);
        let (width, height) = self.direction.size();
        GameplayUiRect::new(
            position.left(),
            position.top(screen_height, height),
            width,
            height,
        )
    }

    #[must_use]
    pub const fn with_scale_pivot(mut self, scale_pivot: TutorialCueScalePivot) -> Self {
        self.scale_pivot = scale_pivot;
        self
    }

    pub(crate) fn render_rect(
        self,
        viewport: Vec2,
        ui_scale: f32,
        elapsed_secs: f32,
    ) -> GameplayUiRect {
        scale_tutorial_cue_rect(
            self.reference_rect(viewport.y, elapsed_secs),
            self.scale_pivot,
            viewport,
            ui_scale,
        )
    }
}

fn scale_tutorial_cue_rect(
    rect: GameplayUiRect,
    pivot: TutorialCueScalePivot,
    viewport: Vec2,
    ui_scale: f32,
) -> GameplayUiRect {
    let pivot = pivot.screen_point(viewport);
    GameplayUiRect::new(
        pivot.x + (rect.x - pivot.x) * ui_scale,
        pivot.y + (rect.y - pivot.y) * ui_scale,
        rect.width * ui_scale,
        rect.height * ui_scale,
    )
}

fn tutorial_arrow_offset(elapsed_secs: f32) -> f32 {
    -10.0 + (elapsed_secs * TUTORIAL_ARROW_FREQUENCY).sin() * 10.0
}

fn animated_tutorial_arrow_position(cue: TutorialArrowCue, offset: f32) -> TutorialCuePosition {
    match (cue.direction, cue.position) {
        (TutorialArrowDirection::Left, TutorialCuePosition::TopLeft { left, top }) => {
            TutorialCuePosition::TopLeft {
                left: left - offset,
                top,
            }
        }
        (TutorialArrowDirection::Left, TutorialCuePosition::BottomLeft { left, bottom }) => {
            TutorialCuePosition::BottomLeft {
                left: left - offset,
                bottom,
            }
        }
        (TutorialArrowDirection::Right, TutorialCuePosition::TopLeft { left, top }) => {
            TutorialCuePosition::TopLeft {
                left: left + offset,
                top,
            }
        }
        (TutorialArrowDirection::Right, TutorialCuePosition::BottomLeft { left, bottom }) => {
            TutorialCuePosition::BottomLeft {
                left: left + offset,
                bottom,
            }
        }
        (TutorialArrowDirection::Up, TutorialCuePosition::TopLeft { left, top }) => {
            TutorialCuePosition::TopLeft {
                left,
                top: top - offset,
            }
        }
        (TutorialArrowDirection::Up, TutorialCuePosition::BottomLeft { left, bottom }) => {
            TutorialCuePosition::BottomLeft {
                left,
                bottom: bottom + offset,
            }
        }
        (TutorialArrowDirection::Down, TutorialCuePosition::TopLeft { left, top }) => {
            TutorialCuePosition::TopLeft {
                left,
                top: top + offset,
            }
        }
        (TutorialArrowDirection::Down, TutorialCuePosition::BottomLeft { left, bottom }) => {
            TutorialCuePosition::BottomLeft {
                left,
                bottom: bottom - offset,
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TutorialUi {
    pub instruction: Option<String>,
    /// Exact `SubText2` layer at the top of the viewport.
    pub secondary_instruction: Option<String>,
    pub illustration: Option<TutorialIllustrationCue>,
    pub arrow: Option<TutorialArrowCue>,
}

impl TutorialUi {
    #[must_use]
    pub fn has_content(&self) -> bool {
        self.instruction.is_some()
            || self.secondary_instruction.is_some()
            || self.illustration.is_some()
            || self.arrow.is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct TutorialOverlayUiModel {
    /// Explicit lifecycle gate. Content alone can never resurrect this overlay.
    pub visible: bool,
    pub ui_scale: f32,
    pub tutorial: TutorialUi,
}

impl Default for TutorialOverlayUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            ui_scale: 1.0,
            tutorial: TutorialUi::default(),
        }
    }
}

impl TutorialOverlayUiModel {
    #[must_use]
    pub fn retrobution_reference_frame() -> Self {
        Self {
            visible: true,
            ui_scale: 1.0,
            tutorial: TutorialUi {
                instruction: Some("MOVE YOUR MOUSE TO THE RIGHT AND FIND THE MARKER.".to_owned()),
                secondary_instruction: None,
                illustration: Some(TutorialIllustrationCue::mouse()),
                arrow: Some(TutorialArrowCue::mouse(TutorialArrowDirection::Right)),
            },
        }
    }

    pub fn hide_and_clear(&mut self) {
        self.visible = false;
        self.tutorial = TutorialUi::default();
    }

    #[must_use]
    pub fn is_renderable(&self) -> bool {
        self.visible && self.tutorial.has_content()
    }
}

#[derive(Default)]
pub struct TutorialOverlayUiPlugin;

impl Plugin for TutorialOverlayUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<TutorialOverlayUiModel>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_tutorial_overlay,
            )
            .add_systems(
                Update,
                ((
                    bind_tutorial_overlay
                        .after(crate::mission_ui::MissionUiSet::Presentation)
                        .before(LocalizationSet::Apply),
                    layout_tutorial_overlay.after(bind_tutorial_overlay),
                ))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

#[derive(Clone, Resource)]
struct TutorialOverlayUiAssets {
    illustrations: [Handle<Image>; 8],
    arrows: [Handle<Image>; 4],
    instruction_font: Handle<Font>,
}

impl TutorialOverlayUiAssets {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            illustrations: TutorialIllustration::ALL
                .map(|illustration| asset_server.load(illustration.asset_path())),
            arrows: TutorialArrowDirection::ALL
                .map(|direction| asset_server.load(direction.asset_path())),
            instruction_font: asset_server.load(TUTORIAL_INSTRUCTION_FONT_PATH),
        }
    }
}

#[derive(Component)]
pub struct TutorialOverlayRoot;
#[derive(Component)]
struct TutorialInstructionLayout;
#[derive(Component)]
struct TutorialInstructionText;
#[derive(Component)]
struct TutorialInstructionShadow;
#[derive(Component)]
struct TutorialSecondaryInstructionLayout;
#[derive(Component)]
struct TutorialSecondaryInstructionText;
#[derive(Component)]
struct TutorialSecondaryInstructionShadow;
#[derive(Component)]
struct TutorialCueRoot;
#[derive(Component)]
struct TutorialIllustrationImage;
#[derive(Component)]
struct TutorialArrowImage;

pub(crate) fn tutorial_instruction_layout(top: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(top),
        width: percent(100),
        height: px(TUTORIAL_TEXT_RECT.height),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect {
            left: px(10),
            right: px(6),
            top: px(4),
            bottom: px(6),
        },
        ..default()
    }
}

fn instruction_font(font: &Handle<Font>) -> TextFont {
    TextFont {
        font: (font.clone()).into(),
        font_size: (TUTORIAL_INSTRUCTION_FONT_SIZE).into(),
        ..default()
    }
}

fn spawn_tutorial_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = TutorialOverlayUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            TutorialOverlayRoot,
            Pickable::IGNORE,
            // Unity IMGUI GUI.depth=5 is in front of the HUD's depths 8/10.
            GlobalZIndex(1_200),
        ))
        .with_children(|tutorial| {
            let instruction_text_font = instruction_font(&assets.instruction_font);
            let instruction_auto_fit = UiTextAutoFit::new(
                TUTORIAL_TEXT_RECT.width - 16.0,
                TUTORIAL_TEXT_RECT.height - 10.0,
                &(
                    instruction_text_font.clone(),
                    bevy::text::LineHeight::default(),
                ),
            );
            let instruction_rect = tutorial_instruction_layout(TUTORIAL_TEXT_RECT.y);
            tutorial
                .spawn((
                    instruction_rect.clone(),
                    UiTransform::default(),
                    TutorialInstructionLayout,
                    Pickable::IGNORE,
                ))
                .with_child((
                    Node {
                        width: percent(100),
                        ..default()
                    },
                    Text::new(""),
                    LocalizedText::new("ui.tutorial.overlay.instruction", "{instruction}")
                        .with_arg("instruction", ""),
                    instruction_text_font.clone(),
                    instruction_auto_fit.clone(),
                    TextColor(Color::BLACK),
                    TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                    UiTransform::from_translation(Val2::px(1.0, 1.0)),
                    TutorialInstructionShadow,
                    Pickable::IGNORE,
                ));
            tutorial
                .spawn((
                    instruction_rect,
                    UiTransform::default(),
                    TutorialInstructionLayout,
                    Pickable::IGNORE,
                ))
                .with_child((
                    Node {
                        width: percent(100),
                        ..default()
                    },
                    Text::new(""),
                    LocalizedText::new("ui.tutorial.overlay.instruction", "{instruction}")
                        .with_arg("instruction", ""),
                    instruction_text_font.clone(),
                    instruction_auto_fit.clone(),
                    TextColor(Color::WHITE),
                    TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                    TutorialInstructionText,
                    Pickable::IGNORE,
                ));
            let secondary_rect = tutorial_instruction_layout(0.0);
            tutorial
                .spawn((
                    secondary_rect.clone(),
                    UiTransform::default(),
                    TutorialSecondaryInstructionLayout,
                    Pickable::IGNORE,
                ))
                .with_child((
                    Node {
                        width: percent(100),
                        ..default()
                    },
                    Text::new(""),
                    LocalizedText::new(
                        "ui.tutorial.overlay.secondary_instruction",
                        "{instruction}",
                    )
                    .with_arg("instruction", ""),
                    instruction_text_font.clone(),
                    instruction_auto_fit.clone(),
                    TextColor(Color::BLACK),
                    TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                    UiTransform::from_translation(Val2::px(1.0, 1.0)),
                    TutorialSecondaryInstructionShadow,
                    Pickable::IGNORE,
                ));
            tutorial
                .spawn((
                    secondary_rect,
                    UiTransform::default(),
                    TutorialSecondaryInstructionLayout,
                    Pickable::IGNORE,
                ))
                .with_child((
                    Node {
                        width: percent(100),
                        ..default()
                    },
                    Text::new(""),
                    LocalizedText::new(
                        "ui.tutorial.overlay.secondary_instruction",
                        "{instruction}",
                    )
                    .with_arg("instruction", ""),
                    instruction_text_font,
                    instruction_auto_fit,
                    TextColor(Color::WHITE),
                    TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                    TutorialSecondaryInstructionText,
                    Pickable::IGNORE,
                ));
            tutorial
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    UiTransform::default(),
                    TutorialCueRoot,
                    Pickable::IGNORE,
                ))
                .with_children(|cue| {
                    cue.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            display: Display::None,
                            left: px(TUTORIAL_MOUSE_RECT.x),
                            bottom: px(196),
                            width: px(TUTORIAL_MOUSE_RECT.width),
                            height: px(TUTORIAL_MOUSE_RECT.height),
                            ..default()
                        },
                        ImageNode {
                            image: assets.illustrations[TutorialIllustration::Mouse.index()]
                                .clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        TutorialIllustrationImage,
                        Pickable::IGNORE,
                    ));
                    cue.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            display: Display::None,
                            left: px(TUTORIAL_RIGHT_RECT.x),
                            bottom: px(236),
                            width: px(TUTORIAL_RIGHT_RECT.width),
                            height: px(TUTORIAL_RIGHT_RECT.height),
                            ..default()
                        },
                        ImageNode {
                            image: assets.arrows[TutorialArrowDirection::Right.index()].clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        TutorialArrowImage,
                        Pickable::IGNORE,
                    ));
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn bind_tutorial_overlay(
    model: Res<TutorialOverlayUiModel>,
    mission_ui: Option<Res<MissionUiModel>>,
    assets: Res<TutorialOverlayUiAssets>,
    mut nodes: ParamSet<(
        Single<&mut Node, (With<TutorialOverlayRoot>, Without<TutorialCueRoot>)>,
        Single<&mut Node, (With<TutorialCueRoot>, Without<TutorialOverlayRoot>)>,
        Single<
            (&mut Node, &mut ImageNode),
            (With<TutorialIllustrationImage>, Without<TutorialArrowImage>),
        >,
        Single<
            (&mut Node, &mut ImageNode),
            (With<TutorialArrowImage>, Without<TutorialIllustrationImage>),
        >,
    )>,
    mut primary: ParamSet<(
        Single<
            &mut LocalizedText,
            (
                With<TutorialInstructionText>,
                Without<TutorialInstructionShadow>,
                Without<TutorialSecondaryInstructionText>,
                Without<TutorialSecondaryInstructionShadow>,
            ),
        >,
        Single<
            &mut LocalizedText,
            (
                With<TutorialInstructionShadow>,
                Without<TutorialInstructionText>,
                Without<TutorialSecondaryInstructionText>,
                Without<TutorialSecondaryInstructionShadow>,
            ),
        >,
    )>,
    mut secondary: ParamSet<(
        Single<
            &mut LocalizedText,
            (
                With<TutorialSecondaryInstructionText>,
                Without<TutorialSecondaryInstructionShadow>,
                Without<TutorialInstructionText>,
                Without<TutorialInstructionShadow>,
            ),
        >,
        Single<
            &mut LocalizedText,
            (
                With<TutorialSecondaryInstructionShadow>,
                Without<TutorialSecondaryInstructionText>,
                Without<TutorialInstructionText>,
                Without<TutorialInstructionShadow>,
            ),
        >,
    )>,
) {
    if !model.is_changed()
        && !mission_ui
            .as_ref()
            .is_some_and(|mission_ui| mission_ui.is_changed())
    {
        return;
    }
    nodes.p0().display = if gameplay_chrome_visible(model.is_renderable(), mission_ui.as_deref()) {
        Display::Flex
    } else {
        Display::None
    };

    let instruction = model.tutorial.instruction.as_deref().unwrap_or_default().to_uppercase();
    {
        let mut item = primary.p0();
        **item = LocalizedText::new("ui.tutorial.overlay.instruction", "{instruction}")
            .with_arg("instruction", instruction.clone());
    }
    {
        let mut item = primary.p1();
        **item = LocalizedText::new("ui.tutorial.overlay.instruction", "{instruction}")
            .with_arg("instruction", instruction);
    }
    let secondary_instruction = model
        .tutorial
        .secondary_instruction
        .as_deref()
        .unwrap_or_default().to_uppercase();
    {
        let mut item = secondary.p0();
        **item = LocalizedText::new("ui.tutorial.overlay.secondary_instruction", "{instruction}")
            .with_arg("instruction", secondary_instruction.clone());
    }
    {
        let mut item = secondary.p1();
        **item = LocalizedText::new("ui.tutorial.overlay.secondary_instruction", "{instruction}")
            .with_arg("instruction", secondary_instruction);
    }

    nodes.p1().display = if model.tutorial.illustration.is_some() || model.tutorial.arrow.is_some()
    {
        Display::Flex
    } else {
        Display::None
    };
    {
        let mut illustration = nodes.p2();
        let (node, image) = &mut *illustration;
        if let Some(cue) = model.tutorial.illustration {
            let (width, height) = cue.illustration.size();
            node.display = Display::Flex;
            apply_tutorial_cue_position(node, cue.position);
            node.width = px(width);
            node.height = px(height);
            image.image = assets.illustrations[cue.illustration.index()].clone();
        } else {
            node.display = Display::None;
        }
    }
    {
        let mut arrow = nodes.p3();
        let (node, image) = &mut *arrow;
        if let Some(cue) = model.tutorial.arrow {
            let (width, height) = cue.direction.size();
            node.display = Display::Flex;
            apply_tutorial_cue_position(node, cue.position);
            node.width = px(width);
            node.height = px(height);
            image.image = assets.arrows[cue.direction.index()].clone();
        } else {
            node.display = Display::None;
        }
    }
}

fn layout_tutorial_overlay(
    time: Res<Time>,
    model: Res<TutorialOverlayUiModel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut previous_static_layout: Local<
        Option<(
            u32,
            u32,
            u32,
            Option<TutorialIllustrationCue>,
            Option<TutorialArrowCue>,
        )>,
    >,
    mut layouts: ParamSet<(
        Query<(&mut Node, &mut UiTransform), With<TutorialInstructionLayout>>,
        Query<(&mut Node, &mut UiTransform), With<TutorialSecondaryInstructionLayout>>,
        Single<&mut Node, (With<TutorialIllustrationImage>, Without<TutorialArrowImage>)>,
        Single<&mut Node, (With<TutorialArrowImage>, Without<TutorialIllustrationImage>)>,
    )>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
        model.ui_scale
    } else {
        1.0
    };
    let layout_key = (
        viewport.x.to_bits(),
        viewport.y.to_bits(),
        scale.to_bits(),
        model.tutorial.illustration,
        model.tutorial.arrow,
    );
    if model.tutorial.arrow.is_none() && previous_static_layout.as_ref() == Some(&layout_key) {
        return;
    }
    *previous_static_layout = Some(layout_key);

    let instruction_top = legacy_tutorial_instruction_top(window.height());
    for (mut node, mut transform) in &mut layouts.p0() {
        node.top = px(instruction_top);
        transform.scale = Vec2::splat(scale);
    }
    for (mut node, mut transform) in &mut layouts.p1() {
        node.top = px(TUTORIAL_TEXT_RECT.height * (scale - 1.0) * 0.5);
        transform.scale = Vec2::splat(scale);
    }
    if let Some(cue) = model.tutorial.illustration {
        apply_tutorial_cue_rect(&mut layouts.p2(), cue.render_rect(viewport, scale));
    }
    if let Some(cue) = model.tutorial.arrow {
        apply_tutorial_cue_rect(
            &mut layouts.p3(),
            cue.render_rect(viewport, scale, time.elapsed_secs()),
        );
    }
}

pub(crate) fn legacy_tutorial_instruction_top(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return TUTORIAL_TEXT_RECT.y;
    }
    (viewport_height.round() as i32 / 4) as f32
}

fn apply_tutorial_cue_rect(node: &mut Node, rect: GameplayUiRect) {
    node.left = px(rect.x);
    node.top = px(rect.y);
    node.bottom = Val::Auto;
    node.width = px(rect.width);
    node.height = px(rect.height);
}

fn apply_tutorial_cue_position(node: &mut Node, position: TutorialCuePosition) {
    match position {
        TutorialCuePosition::TopLeft { left, top } => {
            node.left = px(left);
            node.top = px(top);
            node.bottom = Val::Auto;
        }
        TutorialCuePosition::BottomLeft { left, bottom } => {
            node.left = px(left);
            node.top = Val::Auto;
            node.bottom = px(bottom);
        }
    }
}

#[cfg(test)]
mod tests;
