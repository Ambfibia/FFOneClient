//! Exact native presentation for the tutorial's type-9 Nanocom message.
//!
//! The legacy `cnGUINanocom` queue owns message lifetime independently from
//! the tutorial coroutine. Scene events only hide and freeze the head; they
//! do not consume it. This module preserves that boundary; the caller supplies
//! semantic title/body identities and the shared localization system resolves
//! the currently selected language.

use std::collections::VecDeque;

use bevy::{
    audio::{PlaybackSettings, Volume},
    prelude::*,
    text::{LineBreak, LineHeight},
    window::PrimaryWindow,
};

use crate::localization::{LocalizationSet, LocalizedText, LocalizedTextCase, UiTextAutoFit};

pub const TUTORIAL_NANOCOM_NPC_TYPE: i32 = 2_671;
pub const TUTORIAL_NANOCOM_MESSAGE_TYPE: i32 = 9;
pub const TUTORIAL_NANOCOM_HEAD_LIFETIME_SECONDS: f32 = 10.0;
pub const TUTORIAL_NANOCOM_REVEAL_SECONDS: f32 = 0.5;

/// Deterministic destination installed by `install-gameplay-ui` from the
/// exact `nanocom_message_npc--53a2832bc101e387.png` source bytes.
pub const TUTORIAL_NANOCOM_NPC_FRAME_PATH: &str = "ui/en/gameplay/nanocom/message/npc.png";
/// Existing semantic icon. NPC 2671 resolves to NPC-table row 88, icon type
/// 4/number 87; this path is referenced directly and must not be republished.
pub const TUTORIAL_NANOCOM_NUMBUH_TWO_ICON_PATH: &str = "icons/entities/npc/npcicon_87.png";
pub const TUTORIAL_NANOCOM_TITLE_FONT_PATH: &str = "fonts/jeffe.otf";
pub const TUTORIAL_NANOCOM_BODY_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";
pub const TUTORIAL_NANOCOM_SLIDE_IN_PATH: &str = "audio/sfx/ui/comm_slidein.ogg";
pub const TUTORIAL_NANOCOM_SLIDE_OUT_PATH: &str = "audio/sfx/ui/comm_slideout.ogg";

pub const TUTORIAL_NANOCOM_MESSAGE_WIDTH: f32 = 372.0;
pub const TUTORIAL_NANOCOM_MESSAGE_HEIGHT: f32 = 122.0;
pub const TUTORIAL_NANOCOM_WINDOW_WIDTH: f32 = 174.0;
pub const TUTORIAL_NANOCOM_REVEALED_RIGHT_MARGIN: f32 = 122.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialNanocomRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl TutorialNanocomRect {
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
}

pub const TUTORIAL_NANOCOM_NPC_FRAME_RECT: TutorialNanocomRect =
    TutorialNanocomRect::new(51.0, 3.0, 321.0, 119.0);
pub const TUTORIAL_NANOCOM_NPC_ICON_RECT: TutorialNanocomRect =
    TutorialNanocomRect::new(60.0, 15.0, 64.0, 64.0);
pub const TUTORIAL_NANOCOM_TITLE_RECT: TutorialNanocomRect =
    TutorialNanocomRect::new(130.0, 2.0, 200.0, 30.0);
pub const TUTORIAL_NANOCOM_TEXT_RECT: TutorialNanocomRect =
    TutorialNanocomRect::new(120.0, 25.0, 180.0, 70.0);
/// Bevy UI text does not reproduce Unity IMGUI's `GUIStyle.padding` paint
/// offset on the same text node. Flatten the serialized messagetext padding
/// L10/T4/R6/B6 into the actual glyph layout rectangle.
pub const TUTORIAL_NANOCOM_TEXT_CONTENT_RECT: TutorialNanocomRect =
    TutorialNanocomRect::new(130.0, 29.0, 164.0, 60.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialNanocomPadding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl TutorialNanocomPadding {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialNanocomTextStyle {
    pub legacy_name: &'static str,
    pub source_font_name: &'static str,
    pub source_font_path_id: i64,
    pub semantic_font_path: &'static str,
    pub font_size: f32,
    pub line_height: f32,
    pub color: [f32; 4],
    pub legacy_alignment: i32,
    pub padding: TutorialNanocomPadding,
    pub justify: Justify,
    pub line_break: LineBreak,
}

/// `MenuBigFont14`, used by `RenderNanoMessage` for the NPC title.
pub const TUTORIAL_NANOCOM_TITLE_STYLE: TutorialNanocomTextStyle = TutorialNanocomTextStyle {
    legacy_name: "MenuBigFont14",
    source_font_name: "JEFFE___14",
    source_font_path_id: 903,
    semantic_font_path: TUTORIAL_NANOCOM_TITLE_FONT_PATH,
    // JEFFE___14's fixed raster cap height maps to JEFFE.otf at 12 px.
    font_size: 12.0,
    line_height: 13.71,
    color: [0.0, 1.0, 1.0, 1.0],
    legacy_alignment: 3,
    padding: TutorialNanocomPadding::ZERO,
    justify: Justify::Left,
    line_break: LineBreak::WordBoundary,
};

/// `MenuMessageText`, used by `RenderNanoMessage` for the message body.
pub const TUTORIAL_NANOCOM_BODY_STYLE: TutorialNanocomTextStyle = TutorialNanocomTextStyle {
    legacy_name: "MenuMessageText",
    source_font_name: "ChaletBook-Regular Small",
    source_font_path_id: 1_018,
    semantic_font_path: TUTORIAL_NANOCOM_BODY_FONT_PATH,
    // The primary Computress crop calibrates the replacement raster bounds at
    // 11 px; the recovered 164 px content rectangle preserves clean wrapping.
    font_size: 11.0,
    line_height: 12.072,
    color: [1.0, 0.995_967_75, 1.0, 1.0],
    legacy_alignment: 0,
    padding: TutorialNanocomPadding::new(10.0, 4.0, 6.0, 6.0),
    justify: Justify::Left,
    line_break: LineBreak::WordBoundary,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TutorialNanocomMessage {
    pub npc_type: i32,
    pub message_type: i32,
    pub title: LocalizedText,
    pub body: LocalizedText,
}

#[derive(Debug, Default, Resource)]
pub struct TutorialNanocomMessageQueue {
    pending: VecDeque<TutorialNanocomMessage>,
}

impl TutorialNanocomMessageQueue {
    pub fn enqueue_type_9_numbuh_two(&mut self, title: LocalizedText, body: LocalizedText) {
        self.pending.push_back(TutorialNanocomMessage {
            npc_type: TUTORIAL_NANOCOM_NPC_TYPE,
            message_type: TUTORIAL_NANOCOM_MESSAGE_TYPE,
            title: title.into(),
            body: body.into(),
        });
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    fn pop_front(&mut self) -> Option<TutorialNanocomMessage> {
        self.pending.pop_front()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct TutorialNanocomMessageContext {
    pub scene_event_active: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TutorialNanocomMessageSound {
    SlideIn,
    SlideOut,
}

#[derive(Debug, Default, Resource)]
pub struct TutorialNanocomMessageSoundQueue {
    pending: VecDeque<TutorialNanocomMessageSound>,
}

impl TutorialNanocomMessageSoundQueue {
    pub fn pop_front(&mut self) -> Option<TutorialNanocomMessageSound> {
        self.pending.pop_front()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }
}

#[derive(Clone, Debug, Resource)]
pub struct TutorialNanocomMessageState {
    active: Option<TutorialNanocomMessage>,
    head_remaining_seconds: f32,
    reveal_remaining: f32,
}

impl Default for TutorialNanocomMessageState {
    fn default() -> Self {
        Self {
            active: None,
            head_remaining_seconds: 0.0,
            reveal_remaining: 0.0,
        }
    }
}

impl TutorialNanocomMessageState {
    #[must_use]
    pub fn active(&self) -> Option<&TutorialNanocomMessage> {
        self.active.as_ref()
    }

    #[must_use]
    pub const fn head_remaining_seconds(&self) -> f32 {
        self.head_remaining_seconds
    }

    #[must_use]
    pub fn reveal_parameter(&self) -> f32 {
        self.reveal_remaining.clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn is_visible(&self, context: TutorialNanocomMessageContext) -> bool {
        self.active.is_some() && !context.scene_event_active
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    fn tick(
        &mut self,
        queue: &mut TutorialNanocomMessageQueue,
        sounds: &mut TutorialNanocomMessageSoundQueue,
        context: TutorialNanocomMessageContext,
        delta_seconds: f32,
    ) {
        if context.scene_event_active || !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return;
        }
        if self.active.is_none() {
            let Some(message) = queue.pop_front() else {
                return;
            };
            self.active = Some(message);
            self.head_remaining_seconds = TUTORIAL_NANOCOM_HEAD_LIFETIME_SECONDS;
            self.reveal_remaining = 1.0;
            sounds
                .pending
                .push_back(TutorialNanocomMessageSound::SlideIn);
        }
        self.head_remaining_seconds -= delta_seconds;
        self.reveal_remaining =
            (self.reveal_remaining - delta_seconds / TUTORIAL_NANOCOM_REVEAL_SECONDS).max(0.0);
        if self.head_remaining_seconds < 0.0 {
            self.active = None;
            self.reveal_remaining = 0.0;
            sounds
                .pending
                .push_back(TutorialNanocomMessageSound::SlideOut);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialNanocomMessageGeometry {
    pub panel: TutorialNanocomRect,
    pub frame: TutorialNanocomRect,
    pub icon: TutorialNanocomRect,
    pub title: TutorialNanocomRect,
    pub body: TutorialNanocomRect,
}

#[must_use]
pub fn tutorial_nanocom_message_geometry(
    viewport_width: f32,
    reveal_remaining: f32,
) -> TutorialNanocomMessageGeometry {
    let remaining = reveal_remaining.clamp(0.0, 1.0);
    let squared = remaining * remaining;
    let sine = (squared * std::f32::consts::FRAC_PI_2).sin();
    let easing = sine * sine;
    let panel = TutorialNanocomRect::new(
        viewport_width - TUTORIAL_NANOCOM_MESSAGE_WIDTH - TUTORIAL_NANOCOM_REVEALED_RIGHT_MARGIN
            + easing * TUTORIAL_NANOCOM_MESSAGE_WIDTH,
        0.0,
        (1.0 - easing) * TUTORIAL_NANOCOM_MESSAGE_WIDTH,
        TUTORIAL_NANOCOM_MESSAGE_HEIGHT,
    );
    TutorialNanocomMessageGeometry {
        panel,
        frame: TUTORIAL_NANOCOM_NPC_FRAME_RECT,
        icon: TUTORIAL_NANOCOM_NPC_ICON_RECT,
        title: TUTORIAL_NANOCOM_TITLE_RECT,
        body: TUTORIAL_NANOCOM_TEXT_RECT,
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum TutorialNanocomMessageSet {
    Context,
    Tick,
    Bind,
    Audio,
}

#[derive(Clone, Copy, Debug, Component, Eq, PartialEq)]
enum TutorialNanocomMessageElement {
    Root,
    Panel,
    Frame,
    Icon,
    Title,
    Body,
}

pub struct TutorialNanocomMessagePlugin;

impl Plugin for TutorialNanocomMessagePlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<TutorialNanocomMessageQueue>()
            .init_resource::<TutorialNanocomMessageState>()
            .init_resource::<TutorialNanocomMessageContext>()
            .init_resource::<TutorialNanocomMessageSoundQueue>()
            .configure_sets(
                Update,
                (
                    TutorialNanocomMessageSet::Context,
                    TutorialNanocomMessageSet::Tick,
                    TutorialNanocomMessageSet::Bind,
                    TutorialNanocomMessageSet::Audio,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_tutorial_nanocom_message_ui,
            )
            .add_systems(
                Update,
                (tick_tutorial_nanocom_message.in_set(TutorialNanocomMessageSet::Tick))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (sync_tutorial_nanocom_message_ui
                    .in_set(TutorialNanocomMessageSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (play_tutorial_nanocom_message_sounds.in_set(TutorialNanocomMessageSet::Audio))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

fn absolute_rect(rect: TutorialNanocomRect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.left),
        top: px(rect.top),
        width: px(rect.width),
        height: px(rect.height),
        ..default()
    }
}

fn spawn_tutorial_nanocom_message_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let frame = asset_server.load(TUTORIAL_NANOCOM_NPC_FRAME_PATH);
    let icon = asset_server.load(TUTORIAL_NANOCOM_NUMBUH_TWO_ICON_PATH);
    let title_font = asset_server.load(TUTORIAL_NANOCOM_TITLE_STYLE.semantic_font_path);
    let body_font = asset_server.load(TUTORIAL_NANOCOM_BODY_STYLE.semantic_font_path);
    let title_text_font = (
        TextFont {
            font: (title_font).into(),
            font_size: (TUTORIAL_NANOCOM_TITLE_STYLE.font_size).into(),
            ..default()
        },
        LineHeight::Px(TUTORIAL_NANOCOM_TITLE_STYLE.line_height),
    );
    let title_auto_fit = UiTextAutoFit::new(
        TUTORIAL_NANOCOM_TITLE_RECT.width,
        TUTORIAL_NANOCOM_TITLE_RECT.height,
        &title_text_font,
    );
    let body_text_font = (
        TextFont {
            font: (body_font).into(),
            font_size: (TUTORIAL_NANOCOM_BODY_STYLE.font_size).into(),
            ..default()
        },
        LineHeight::Px(TUTORIAL_NANOCOM_BODY_STYLE.line_height),
    );
    let body_auto_fit = UiTextAutoFit::new(
        TUTORIAL_NANOCOM_TEXT_CONTENT_RECT.width,
        TUTORIAL_NANOCOM_TEXT_CONTENT_RECT.height,
        &body_text_font,
    );
    commands
        .spawn((
            TutorialNanocomMessageElement::Root,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(-1),
        ))
        .with_children(|root| {
            root.spawn((
                TutorialNanocomMessageElement::Panel,
                absolute_rect(TutorialNanocomRect::new(
                    0.0,
                    0.0,
                    TUTORIAL_NANOCOM_MESSAGE_WIDTH,
                    TUTORIAL_NANOCOM_MESSAGE_HEIGHT,
                )),
                Visibility::Hidden,
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn((
                    TutorialNanocomMessageElement::Frame,
                    absolute_rect(TUTORIAL_NANOCOM_NPC_FRAME_RECT),
                    ImageNode::new(frame),
                    Pickable::IGNORE,
                ));
                // `FusionFallHUDSkin.BIGFont14` is TextAnchor.MiddleLeft.
                // Keep the serialized rectangle as the alignment container
                // and vertically center the JEFFE___14 text inside it.
                panel
                    .spawn((Node {
                        align_items: AlignItems::Center,
                        ..absolute_rect(TUTORIAL_NANOCOM_TITLE_RECT)
                    },))
                    .with_child((
                        TutorialNanocomMessageElement::Title,
                        Text::new(""),
                        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                        LocalizedTextCase::Uppercase,
                        title_text_font,
                        title_auto_fit,
                        TextColor(Color::srgba(
                            TUTORIAL_NANOCOM_TITLE_STYLE.color[0],
                            TUTORIAL_NANOCOM_TITLE_STYLE.color[1],
                            TUTORIAL_NANOCOM_TITLE_STYLE.color[2],
                            TUTORIAL_NANOCOM_TITLE_STYLE.color[3],
                        )),
                        TextLayout::new(
                            TUTORIAL_NANOCOM_TITLE_STYLE.justify,
                            TUTORIAL_NANOCOM_TITLE_STYLE.line_break,
                        ),
                        Pickable::IGNORE,
                    ));
                panel.spawn((
                    TutorialNanocomMessageElement::Body,
                    absolute_rect(TUTORIAL_NANOCOM_TEXT_CONTENT_RECT),
                    Text::new(""),
                    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                    body_text_font,
                    body_auto_fit,
                    TextColor(Color::srgba(
                        TUTORIAL_NANOCOM_BODY_STYLE.color[0],
                        TUTORIAL_NANOCOM_BODY_STYLE.color[1],
                        TUTORIAL_NANOCOM_BODY_STYLE.color[2],
                        TUTORIAL_NANOCOM_BODY_STYLE.color[3],
                    )),
                    TextLayout::new(
                        TUTORIAL_NANOCOM_BODY_STYLE.justify,
                        TUTORIAL_NANOCOM_BODY_STYLE.line_break,
                    ),
                    Pickable::IGNORE,
                ));
                // `RenderNanoMessage` draws the portrait after both labels.
                panel.spawn((
                    TutorialNanocomMessageElement::Icon,
                    absolute_rect(TUTORIAL_NANOCOM_NPC_ICON_RECT),
                    ImageNode::new(icon),
                    Pickable::IGNORE,
                ));
            });
        });
}

fn tick_tutorial_nanocom_message(
    time: Res<Time>,
    context: Res<TutorialNanocomMessageContext>,
    mut queue: ResMut<TutorialNanocomMessageQueue>,
    mut state: ResMut<TutorialNanocomMessageState>,
    mut sounds: ResMut<TutorialNanocomMessageSoundQueue>,
) {
    state.tick(&mut queue, &mut sounds, *context, time.delta_secs());
}

fn sync_tutorial_nanocom_message_ui(
    windows: Query<&Window, With<PrimaryWindow>>,
    context: Res<TutorialNanocomMessageContext>,
    state: Res<TutorialNanocomMessageState>,
    mut elements: Query<(
        &TutorialNanocomMessageElement,
        &mut Node,
        Option<&mut Visibility>,
        Option<&mut LocalizedText>,
    )>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let geometry = tutorial_nanocom_message_geometry(window.width(), state.reveal_parameter());
    for (element, mut node, visibility, text) in &mut elements {
        match element {
            TutorialNanocomMessageElement::Root => {}
            TutorialNanocomMessageElement::Panel => {
                node.left = px(geometry.panel.left);
                node.top = px(geometry.panel.top);
                node.width = px(geometry.panel.width);
                node.height = px(geometry.panel.height);
                node.overflow = Overflow::clip();
                if let Some(mut visibility) = visibility {
                    *visibility = if state.is_visible(*context) {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            TutorialNanocomMessageElement::Frame => {}
            TutorialNanocomMessageElement::Icon => {}
            TutorialNanocomMessageElement::Title => {
                if let (Some(active), Some(mut localized)) = (state.active(), text) {
                    if *localized != active.title {
                        *localized = active.title.clone();
                    }
                }
            }
            TutorialNanocomMessageElement::Body => {
                if let (Some(active), Some(mut localized)) = (state.active(), text) {
                    if *localized != active.body {
                        *localized = active.body.clone();
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct TutorialNanocomMessageAudio;

fn play_tutorial_nanocom_message_sounds(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sounds: ResMut<TutorialNanocomMessageSoundQueue>,
    context: Res<TutorialNanocomMessageContext>,
    mut sources: Query<
        (&mut PlaybackSettings, Option<&AudioSink>),
        With<TutorialNanocomMessageAudio>,
    >,
) {
    let paused = context.scene_event_active;
    for (mut settings, sink) in &mut sources {
        if settings.paused != paused {
            settings.paused = paused;
        }
        if let Some(sink) = sink {
            if paused && !sink.is_paused() {
                sink.pause();
            } else if !paused && sink.is_paused() {
                sink.play();
            }
        }
    }
    if paused {
        return;
    }
    while let Some(sound) = sounds.pop_front() {
        let path = match sound {
            TutorialNanocomMessageSound::SlideIn => TUTORIAL_NANOCOM_SLIDE_IN_PATH,
            TutorialNanocomMessageSound::SlideOut => TUTORIAL_NANOCOM_SLIDE_OUT_PATH,
        };
        commands.spawn((
            TutorialNanocomMessageAudio,
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(1.0)),
        ));
    }
}

pub fn cleanup_tutorial_nanocom_message(world: &mut World) {
    world.resource_mut::<TutorialNanocomMessageQueue>().clear();
    world.resource_mut::<TutorialNanocomMessageState>().clear();
    *world.resource_mut::<TutorialNanocomMessageContext>() =
        TutorialNanocomMessageContext::default();
    *world.resource_mut::<TutorialNanocomMessageSoundQueue>() =
        TutorialNanocomMessageSoundQueue::default();
}

#[cfg(test)]
mod tests;
