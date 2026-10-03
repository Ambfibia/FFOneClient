//! Native presentation of the tutorial's location-title typewriter.
//!
//! Retrobution renders this title directly from `cntutorialscript::OnGUI`,
//! outside the ordinary gameplay HUD.  Keeping this as an independent Bevy UI
//! hierarchy is important because the choreography deliberately hides that HUD
//! while the title is on screen.

use bevy::{prelude::*, text::LineHeight, window::PrimaryWindow};

use crate::{
    localization::{Language, Localization, LocalizationSet, LocalizedText},
    tutorial_choreography_runtime::TutorialChoreographyPresentation,
};

/// Exact retained-mode font behind FusionFall's `LeftBigFont16` GUI style.
pub const TUTORIAL_CINEMATIC_TITLE_FONT_PATH: &str = "fonts/jeffe.otf";

pub const TUTORIAL_CINEMATIC_BAR_HEIGHT_FRACTION: f32 = 0.156_739_82;
pub const TUTORIAL_CINEMATIC_TITLE_LEFT: f32 = 50.0;
pub const TUTORIAL_CINEMATIC_TITLE_WIDTH: f32 = 200.0;
pub const TUTORIAL_CINEMATIC_TITLE_HEIGHT: f32 = 100.0;
/// Replacement-font calibration for clean fixed-raster `JEFFE___16` path 1012.
pub const TUTORIAL_CINEMATIC_TITLE_FONT_SIZE: f32 = 14.0;
pub const TUTORIAL_CINEMATIC_TITLE_LINE_HEIGHT: f32 = 16.451_999_66;
pub const TUTORIAL_CINEMATIC_TITLE_SHADOW_OFFSET: f32 = 1.0;
pub const TUTORIAL_CINEMATIC_TITLE_SHADOW_ALPHA_THRESHOLD: f32 = 0.9;

// Serialized `LeftBigFont16.m_Normal.m_TextColor` is very slightly below pure
// white in its green channel.  Unity multiplies this by GUI.color.
const TUTORIAL_CINEMATIC_TITLE_GREEN: f32 = 0.995_967_75;
const TUTORIAL_CINEMATIC_TITLE_Z_INDEX: i32 = 1_902;

#[derive(Debug, Clone, Copy, PartialEq)]
struct CinematicTitleRect {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct CinematicTitleView {
    text: String,
    rect: CinematicTitleRect,
    alpha: f32,
    visible: bool,
    shadow_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
struct TutorialCinematicTitleRoot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
enum TutorialCinematicTitleLabel {
    Shadow,
    Foreground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
enum TutorialCinematicTitleText {
    Shadow,
    Foreground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum TutorialCinematicTitleSet {
    Bind,
}

pub struct TutorialCinematicTitlePlugin;

impl Plugin for TutorialCinematicTitlePlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.add_systems(
            crate::ui_startup::NativeGameplayUiStartup,
            spawn_tutorial_cinematic_title,
        )
        .add_systems(
            Update,
            (sync_tutorial_cinematic_title
                .in_set(TutorialCinematicTitleSet::Bind)
                .before(LocalizationSet::Apply))
            .in_set(crate::ui_startup::NativeUiStartupSet),
        );
    }
}

fn spawn_tutorial_cinematic_title(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(TUTORIAL_CINEMATIC_TITLE_FONT_PATH);
    commands
        .spawn((
            TutorialCinematicTitleRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
            ZIndex(TUTORIAL_CINEMATIC_TITLE_Z_INDEX),
        ))
        .with_children(|root| {
            spawn_title_label(
                root,
                TutorialCinematicTitleLabel::Shadow,
                TutorialCinematicTitleText::Shadow,
                font.clone(),
                Color::BLACK,
            );
            spawn_title_label(
                root,
                TutorialCinematicTitleLabel::Foreground,
                TutorialCinematicTitleText::Foreground,
                font,
                Color::srgba(1.0, TUTORIAL_CINEMATIC_TITLE_GREEN, 1.0, 1.0),
            );
        });
}

fn spawn_title_label(
    parent: &mut ChildSpawnerCommands,
    label: TutorialCinematicTitleLabel,
    text_marker: TutorialCinematicTitleText,
    font: Handle<Font>,
    color: Color,
) {
    parent
        .spawn((
            label,
            title_label_node(0.0, 0.0),
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_child((
            text_marker,
            Text::new(""),
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
            (
                TextFont {
                    font: (font).into(),
                    font_size: (TUTORIAL_CINEMATIC_TITLE_FONT_SIZE).into(),
                    ..default()
                },
                LineHeight::Px(TUTORIAL_CINEMATIC_TITLE_LINE_HEIGHT),
            ),
            TextColor(color),
            TextLayout::default().with_justify(Justify::Left),
            Pickable::IGNORE,
        ));
}

fn title_label_node(left: f32, top: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(left),
        top: px(top),
        width: px(TUTORIAL_CINEMATIC_TITLE_WIDTH),
        height: px(TUTORIAL_CINEMATIC_TITLE_HEIGHT),
        // `LeftBigFont16` is MiddleLeft with this exact serialized padding.
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        padding: UiRect {
            left: px(10),
            right: px(6),
            top: px(4),
            bottom: px(6),
        },
        ..default()
    }
}

fn sync_tutorial_cinematic_title(
    presentation: Res<TutorialChoreographyPresentation>,
    localization: Res<Localization>,
    language: Res<Language>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<
        &mut Visibility,
        (
            With<TutorialCinematicTitleRoot>,
            Without<TutorialCinematicTitleLabel>,
        ),
    >,
    mut labels: Query<
        (&TutorialCinematicTitleLabel, &mut Node, &mut Visibility),
        Without<TutorialCinematicTitleRoot>,
    >,
    mut texts: Query<(
        &TutorialCinematicTitleText,
        &mut LocalizedText,
        &mut TextColor,
    )>,
) {
    let Ok(mut root_visibility) = roots.single_mut() else {
        return;
    };
    let Ok(window) = windows.single() else {
        *root_visibility = Visibility::Hidden;
        return;
    };

    let localized = presentation.subtitle_key.map(|key| {
        localization.text(
            &language,
            &LocalizedText::new(key, "TECH SQUARE.%sTHE FUTURE."),
        )
    });
    let view = cinematic_title_view(
        localized.as_deref(),
        presentation.subtitle_visible_characters,
        presentation.subtitle_alpha,
        window.resolution.height(),
    );

    *root_visibility = if view.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for (label, mut node, mut visibility) in &mut labels {
        let shadow = *label == TutorialCinematicTitleLabel::Shadow;
        let offset = if shadow {
            TUTORIAL_CINEMATIC_TITLE_SHADOW_OFFSET
        } else {
            0.0
        };
        node.left = px(view.rect.left + offset);
        node.top = px(view.rect.top + offset);
        node.width = px(view.rect.width);
        node.height = px(view.rect.height);
        *visibility = if view.visible && (!shadow || view.shadow_visible) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for (marker, mut localized_text, mut color) in &mut texts {
        let current = localized_text.args.get("text");
        if current.map(String::as_str) != Some(view.text.as_str()) {
            *localized_text =
                LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &view.text);
        }
        color.0 = match marker {
            TutorialCinematicTitleText::Shadow => Color::BLACK,
            TutorialCinematicTitleText::Foreground => Color::srgba(
                1.0,
                TUTORIAL_CINEMATIC_TITLE_GREEN,
                1.0,
                if view.shadow_visible { 1.0 } else { view.alpha },
            ),
        };
    }
}

fn cinematic_title_view(
    localized: Option<&str>,
    visible_characters: u16,
    alpha: f32,
    screen_height: f32,
) -> CinematicTitleView {
    let alpha = sanitize_alpha(alpha);
    let normalized = localized.map(normalize_subtitle).unwrap_or_default();
    // The choreography ends at the canonical English count. Finish longer
    // translations on that same frame so their final characters are not lost.
    let text = if visible_characters >= 24 {
        normalized
    } else {
        take_unicode_characters(&normalized, usize::from(visible_characters))
    };
    let visible = alpha > 0.0 && !text.is_empty();
    CinematicTitleView {
        text,
        rect: cinematic_title_rect(screen_height),
        alpha,
        visible,
        shadow_visible: visible && alpha > TUTORIAL_CINEMATIC_TITLE_SHADOW_ALPHA_THRESHOLD,
    }
}

fn normalize_subtitle(localized: &str) -> String {
    localized.replace("%s", "\n")
}

fn take_unicode_characters(text: &str, count: usize) -> String {
    text.chars().take(count).collect()
}

fn sanitize_alpha(alpha: f32) -> f32 {
    if alpha.is_finite() {
        alpha.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn cinematic_title_rect(screen_height: f32) -> CinematicTitleRect {
    CinematicTitleRect {
        left: TUTORIAL_CINEMATIC_TITLE_LEFT,
        top: screen_height
            - TUTORIAL_CINEMATIC_BAR_HEIGHT_FRACTION * screen_height
            - TUTORIAL_CINEMATIC_TITLE_HEIGHT,
        width: TUTORIAL_CINEMATIC_TITLE_WIDTH,
        height: TUTORIAL_CINEMATIC_TITLE_HEIGHT,
    }
}

#[cfg(test)]
mod tests;
