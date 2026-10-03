//! GUISkin image slicing, text styling and button/label spawn helpers.

use super::assets::{
    ACCEPT_BUTTON, ACTIVE_DIALOG, CENTERED_MENU_JEFFE_FONT_SIZE,
    CENTERED_MENU_JEFFE_VERTICAL_SCALE, END_DIALOG, JEFFE_14_LINE_HEIGHT,
    JEFFE_14_REPLACEMENT_FONT_SIZE, JEFFE_14_SOURCE_FONT_PATH_ID, JOURNAL_ACTIVE_PANEL,
    JOURNAL_ACTIVE_TAB, JOURNAL_ACTIVE_TAB_OVER, JOURNAL_ALLOW_RIGHT, JOURNAL_COMPLETED_PANEL,
    JOURNAL_COMPLETED_TAB, JOURNAL_COMPLETED_TAB_OVER, JOURNAL_EMPTY_BOX, JOURNAL_NANO_POWER,
    JOURNAL_NPC_ICON_BACK, JOURNAL_REWARD_BOX, JOURNAL_RIGHT_FRAME, JOURNAL_SELECTED_MISSION_BACK,
    JOURNAL_TRACKED_MISSION_BACK, JOURNAL_WINDOW, MISSION_BACK, MISSION_BODY, MISSION_BOTTOM,
    MISSION_BOTTOM_MULTI, MISSION_BUTTON, MISSION_BUTTON_OVER, MISSION_TOP, MissionUiAssets,
    NPC_MULTI_WINDOW, NPC_WINDOW, OFFER_DIALOG, SYSTEM_DIALOG_BOX,
};
use super::components::{
    JournalAuxButtonKind, JournalContentRect, MenuButtonStyle, MissionTextVerticalAnchor,
    MissionUiControl, MissionUiView, NanocomMenuButton, NpcActionButton,
};
use super::labels::{
    JOURNAL_MISSION_DETAILS_LABEL, JOURNAL_MISSION_OFFER_LABEL, JOURNAL_MISSION_SUMMARY_LABEL,
    JOURNAL_MY_NOTES_LABEL, mission_ui_localized_text,
};
use super::layout::MissionUiRect;
use super::model::{JournalListTab, MissionJournalUi, MissionUiModel};
use crate::{
    character_selection_ui::{
        CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH, CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        CHARACTER_SELECTION_CANCEL_NORMAL_PATH,
    },
    gui_skin::{
        GuiStyle as MissionGuiStyle, gui_effective_font as mission_gui_effective_font,
        gui_style as mission_gui_style,
    },
    localization::UiTextAutoFit,
};
use bevy::{
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::{LineBreak, LineHeight, TextLayoutInfo},
    ui::{FocusPolicy, widget::NodeImageMode},
};

pub(super) fn stretch(image: &Handle<Image>) -> ImageNode {
    ImageNode {
        image: image.clone(),
        image_mode: NodeImageMode::Stretch,
        ..default()
    }
}

pub(super) fn sliced(image: &Handle<Image>, border: BorderRect) -> ImageNode {
    ImageNode {
        image: image.clone(),
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border,
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}

pub(super) fn skin_style_border(skin: &str, style: &str) -> Option<BorderRect> {
    let border = mission_gui_style(skin, style)?.border;
    Some(BorderRect {
        min_inset: Vec2::new(border.left as f32, border.top as f32),
        max_inset: Vec2::new(border.right as f32, border.bottom as f32),
    })
}

pub(super) fn source_style_border(path: &str) -> Option<BorderRect> {
    let (skin, style) = match path {
        MISSION_TOP => ("FusionFallInteractionSkin", "M_Top"),
        MISSION_BODY => ("FusionFallInteractionSkin", "mission_body"),
        MISSION_BOTTOM => ("FusionFallInteractionSkin", "M_Bottom"),
        MISSION_BOTTOM_MULTI => ("FusionFallInteractionSkin", "M_Bottom_Func"),
        MISSION_BUTTON | MISSION_BUTTON_OVER => ("FusionFallInteractionSkin", "m_button"),
        NPC_WINDOW => ("FusionFallInteractionSkin", "single_win"),
        NPC_MULTI_WINDOW => ("FusionFallInteractionSkin", "multi_win"),
        MISSION_BACK => ("FusionFallMissionSkinR", "missionback"),
        JOURNAL_SELECTED_MISSION_BACK => ("FusionFallMissionSkinR", "textArea"),
        JOURNAL_TRACKED_MISSION_BACK => ("FusionFallMissionSkinR", "textField"),
        OFFER_DIALOG => ("FusionFallMissionSkin", "offerdlg"),
        ACTIVE_DIALOG => ("FusionFallMissionSkin", "activedlg"),
        END_DIALOG => ("FusionFallMissionSkin", "enddlg"),
        ACCEPT_BUTTON => ("FusionFallMissionSkin", "acceptbut"),
        JOURNAL_REWARD_BOX => ("FusionFallMissionSkin", "textField"),
        JOURNAL_WINDOW => ("FusionFallMissionSkin", "box"),
        JOURNAL_NPC_ICON_BACK => ("FusionFallMissionSkin", "npcicon"),
        JOURNAL_ALLOW_RIGHT => ("FusionFallMissionSkinR", "AllowTab"),
        JOURNAL_ACTIVE_PANEL => ("FusionFallMissionSkinR", "Active_Tab"),
        JOURNAL_COMPLETED_PANEL => ("FusionFallMissionSkinR", "Compl_Tab"),
        JOURNAL_ACTIVE_TAB | JOURNAL_ACTIVE_TAB_OVER => ("FusionFallMissionSkinR", "Tab1"),
        JOURNAL_COMPLETED_TAB => ("FusionFallMissionSkinR", "Tab2"),
        JOURNAL_COMPLETED_TAB_OVER => ("FusionFallMissionSkinR", "Tab2"),
        JOURNAL_RIGHT_FRAME => ("FusionFallMissionSkinR", "window"),
        JOURNAL_EMPTY_BOX => ("FusionFallMissionSkin", "EmptyBox"),
        JOURNAL_NANO_POWER => ("FusionFallMissionSkin", "NanoBox"),
        SYSTEM_DIALOG_BOX => ("FusionFallSysMessageSkin", "box"),
        CHARACTER_SELECTION_BLUE_BUTTON_PATH | CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH => {
            ("FusionFallSysMessageSkin", "button")
        }
        CHARACTER_SELECTION_CANCEL_NORMAL_PATH => ("FusionFallSysMessageSkin", "CancelButton"),
        _ => return None,
    };
    skin_style_border(skin, style)
}

pub(super) fn styled_image(image: &Handle<Image>, path: &str) -> ImageNode {
    source_style_border(path).map_or_else(|| stretch(image), |border| sliced(image, border))
}

pub(super) fn skin_style_background_rect(rect: MissionUiRect, style: &MissionGuiStyle) -> MissionUiRect {
    MissionUiRect::new(
        rect.x - style.overflow.left as f32,
        rect.y - style.overflow.top as f32,
        rect.width + style.overflow.left as f32 + style.overflow.right as f32,
        rect.height + style.overflow.top as f32 + style.overflow.bottom as f32,
    )
}

pub(super) fn mission_text_font(
    assets: &MissionUiAssets,
    skin_name: &str,
    style_name: &str,
) -> (TextFont, LineHeight) {
    let source = mission_gui_effective_font(skin_name, style_name)
        .expect("converted Retrobution GUI style must resolve its effective font");
    let (handle, semantic_size, line_height) = match source.path_id {
        // The original Unity assets are fixed raster fonts. These sizes are
        // calibrated vector counterparts, not copies of their asset names.
        JEFFE_14_SOURCE_FONT_PATH_ID => (
            &assets.replacement_jeffe_font,
            JEFFE_14_REPLACEMENT_FONT_SIZE,
            JEFFE_14_LINE_HEIGHT,
        ),
        948 => (&assets.replacement_jeffe_font, 8.0, 10.968_000_41),
        953 => (&assets.replacement_jeffe_font, 10.0, 12.071_999_55),
        1_012 => (&assets.replacement_jeffe_font, 14.0, 16.451_999_66),
        // `FusionFallChatSkin.BigFont70` uses the fixed-raster JEFFE___72
        // object for the full-screen Warp Away countdown.
        1_129 => (&assets.replacement_jeffe_font, 70.0, 82.259_994_51),
        1_115 => (&assets.font, 14.0, 14.083_999_63),
        // `ChaletBook-Regular Small`, inherited through `GUISkin.m_Font`.
        1_018 => (&assets.font, 12.0, 12.071_999_55),
        _ => (&assets.font, 12.0, 12.071_999_55),
    };
    (
        TextFont {
            font: (handle.clone()).into(),
            font_size: (semantic_size).into(),
            ..default()
        },
        LineHeight::Px(line_height),
    )
}

pub(super) fn mission_text_color(style: &MissionGuiStyle) -> Color {
    mission_text_state_color(style, "normal")
}

pub(super) fn mission_text_state_color(style: &MissionGuiStyle, state: &str) -> Color {
    let color = &style
        .states
        .get(state)
        .or_else(|| style.states.get("normal"))
        .expect("converted Retrobution GUI style must contain its normal state")
        .text_color;
    Color::srgba(
        color.r as f32,
        color.g as f32,
        color.b as f32,
        color.a as f32,
    )
}

pub(super) fn mission_text_justify(alignment: i64) -> Justify {
    match alignment {
        1 | 4 | 7 => Justify::Center,
        2 | 5 | 8 => Justify::Right,
        _ => Justify::Left,
    }
}

pub(super) fn mission_text_vertical_anchor(
    alignment: i64,
    view: Option<MissionUiView>,
) -> Option<MissionTextVerticalAnchor> {
    // `cnMissionJournal.DoWindowLeft` paints this text through
    // `GUILayout.BeginArea` + `GUILayout.Label`. Its label receives an
    // intrinsic layout height at the top of the scroll content; the 100 px
    // rectangle in FFOne is the viewport, not a direct middle-aligned label.
    if view == Some(MissionUiView::JournalDescription) {
        return None;
    }

    match alignment {
        3..=5 => Some(MissionTextVerticalAnchor::Middle),
        6..=8 => Some(MissionTextVerticalAnchor::Lower),
        _ => None,
    }
}

pub(super) fn mission_text_vertical_offset(
    anchor: MissionTextVerticalAnchor,
    available_height: f32,
    text_top: f32,
    text_height: f32,
) -> f32 {
    let free_height = (available_height - text_height).max(0.0);
    let aligned_top = match anchor {
        MissionTextVerticalAnchor::Middle => free_height * 0.5,
        MissionTextVerticalAnchor::Lower => free_height,
    };
    aligned_top - text_top
}

pub(super) fn mission_text_translation_px(
    anchor: MissionTextVerticalAnchor,
    available_height: f32,
    text_top: f32,
    text_height: f32,
    inverse_scale_factor: f32,
    vertical_scale: f32,
) -> f32 {
    mission_text_vertical_offset(anchor, available_height, text_top, text_height)
        * inverse_scale_factor
        * vertical_scale
}

pub(super) fn mission_visible_text_bounds(layout: &TextLayoutInfo) -> (f32, f32) {
    let bounds = layout.glyphs.iter().fold(None, |bounds, glyph| {
        let (top, bottom) =
            mission_glyph_vertical_bounds(glyph.position.y, glyph.atlas_info.rect.height());
        Some(match bounds {
            Some((current_top, current_bottom)) => {
                (f32::min(current_top, top), f32::max(current_bottom, bottom))
            }
            None => (top, bottom),
        })
    });
    bounds.map_or((0.0, layout.size.y), |(top, bottom)| {
        (top, (bottom - top).max(0.0))
    })
}

pub(super) fn mission_glyph_vertical_bounds(center_y: f32, height: f32) -> (f32, f32) {
    let half_height = height * 0.5;
    (center_y - half_height, center_y + half_height)
}

pub(super) fn mission_text_content_rect(rect: MissionUiRect, style: &MissionGuiStyle) -> MissionUiRect {
    let offset_x = style.content_offset.x as f32;
    let offset_y = style.content_offset.y as f32;
    let left = style.padding.left as f32 + offset_x;
    let right = style.padding.right as f32 - offset_x;
    let top = style.padding.top as f32 + offset_y;
    let bottom = style.padding.bottom as f32 - offset_y;
    MissionUiRect::new(
        rect.x + left,
        rect.y + top,
        (rect.width - left - right).max(0.0),
        (rect.height - top - bottom).max(0.0),
    )
}

pub(super) fn mission_text_layout_rect(
    rect: MissionUiRect,
    style: &MissionGuiStyle,
    font: &(TextFont, LineHeight),
) -> MissionUiRect {
    let mut content = mission_text_content_rect(rect, style);
    // Unity clips GUIStyle text against the outer control rectangle. Bevy
    // clips it against the Text node itself, so the source's vertical padding
    // can otherwise reduce a one-line 20 px label to a 10 px clip and erase
    // the converted JEFFE glyphs entirely (notably `Sel_Bar`).
    let line_height = match font.1 {
        LineHeight::Px(px) => px,
        LineHeight::RelativeToFont(scale) => scale * font.0.font_size.eval(Vec2::ZERO, 16.0),
    };
    let height_to_outer_bottom = (rect.y + rect.height - content.y).max(0.0);
    content.height = content.height.max(line_height.min(height_to_outer_bottom));
    content
}

pub(super) fn skin_label(
    parent: &mut ChildSpawnerCommands,
    rect: MissionUiRect,
    text: impl Into<String>,
    skin_name: &str,
    style_name: &str,
    assets: &MissionUiAssets,
    view: Option<MissionUiView>,
    color_override: Option<Color>,
) {
    skin_label_with_font_size(
        parent,
        rect,
        text,
        skin_name,
        style_name,
        assets,
        view,
        color_override,
        None,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn skin_label_with_font_size(
    parent: &mut ChildSpawnerCommands,
    rect: MissionUiRect,
    text: impl Into<String>,
    skin_name: &str,
    style_name: &str,
    assets: &MissionUiAssets,
    view: Option<MissionUiView>,
    color_override: Option<Color>,
    font_size_override: Option<f32>,
) {
    let text = text.into();
    let localized = mission_ui_localized_text(text.clone());
    let style = mission_gui_style(skin_name, style_name)
        .expect("converted Retrobution GUI style must remain available");
    // Bevy does not apply a Text node's `Node.padding` as Unity IMGUI's
    // GUIStyle paint offset. Flatten the source padding into the glyph layout
    // rectangle, the same way the Nanocom message adapter does.
    let mut text_font = mission_text_font(assets, skin_name, style_name);
    if let Some(font_size) = font_size_override {
        text_font.0.font_size = font_size.into();
    }
    let mut content_rect = mission_text_layout_rect(rect, style, &text_font);
    let center_control_caption = (skin_name == "FusionFallInteractionSkin"
        && matches!(style_name, "button" | "Sel_Bar"))
        || (skin_name == "FusionFallMissionSkinR" && matches!(style_name, "Tab1" | "Tab2"));
    if center_control_caption {
        content_rect.y = rect.y;
        content_rect.height = rect.height;
    }
    let auto_fit = UiTextAutoFit::new(content_rect.width, content_rect.height, &text_font);
    let wraps_mission = matches!(view, Some(MissionUiView::NpcMissionRowText(_)));
    let vertical_anchor = if wraps_mission {
        None
    } else if center_control_caption {
        Some(MissionTextVerticalAnchor::Middle)
    } else {
        mission_text_vertical_anchor(style.alignment, view)
    };
    let mut entity = parent.spawn((
        Node {
            height: if wraps_mission { Val::Auto } else { px(content_rect.height) },
            overflow: if style.text_clipping == 1 && !wraps_mission {
                Overflow::clip()
            } else {
                Overflow::visible()
            },
            ..content_rect.node()
        },
        Text::new(text),
        localized,
        text_font,
        TextColor(color_override.unwrap_or_else(|| mission_text_color(style))),
        TextLayout::new(
            mission_text_justify(style.alignment),
            if style.word_wrap != 0 || wraps_mission {
                LineBreak::WordBoundary
            } else {
                LineBreak::NoWrap
            },
        ),
        // Interaction uses ui_focus_system, which does not inspect Pickable.
        // A caption must let the containing button receive the mouse press.
        FocusPolicy::Pass,
        Pickable::IGNORE,
    ));
    if !wraps_mission {
        entity.insert(auto_fit);
    }
    if let Some(vertical_anchor) = vertical_anchor {
        entity.insert((vertical_anchor, UiTransform::default()));
    }
    insert_journal_content_rect(&mut entity, content_rect, view);
    if let Some(view) = view {
        entity.insert(view);
    }
}

pub(super) fn align_mission_text_vertically(
    mut texts: Query<(
        &MissionTextVerticalAnchor,
        &ComputedNode,
        &TextLayoutInfo,
        &mut UiTransform,
    )>,
) {
    for (anchor, computed, layout, mut transform) in &mut texts {
        // Unity TextAnchor aligns the visible fixed-raster glyphs. Centering
        // Bevy's replacement-font line box also includes its invisible
        // ascent/descent space, which leaves button captions visibly off-axis.
        // Use the actual positioned-glyph ink bounds so EN and RU labels share
        // the same source-owned Middle/Lower alignment without a locale- or
        // tutorial-specific magic offset.
        let (text_top, text_height) = mission_visible_text_bounds(layout);
        // Keep the subpixel result. A centered odd-height source content rect
        // legitimately produces a half-pixel offset; rounding it reintroduces
        // a visible bias. UiTransform scales around the node center, so the
        // ink correction must follow the replacement font's vertical scale.
        let offset = mission_text_translation_px(
            *anchor,
            computed.size().y,
            text_top,
            text_height,
            computed.inverse_scale_factor(),
            transform.scale.y,
        );
        let desired = px(offset);
        if transform.translation.y != desired {
            transform.translation.y = desired;
        }
    }
}

pub(super) fn centered_menu_skin_label(
    parent: &mut ChildSpawnerCommands,
    rect: MissionUiRect,
    text: impl Into<String>,
    skin_name: &str,
    style_name: &str,
    assets: &MissionUiAssets,
    color_override: Option<Color>,
) {
    let text = text.into();
    let localized = mission_ui_localized_text(text.clone());
    let style = mission_gui_style(skin_name, style_name)
        .expect("converted Retrobution GUI menu style must remain available");
    debug_assert_eq!(
        style.alignment, 4,
        "Retrobution Enter-menu labels must remain MiddleCenter"
    );
    let content_rect = mission_text_content_rect(rect, style);
    let source_font = mission_gui_effective_font(skin_name, style_name)
        .expect("converted Retrobution menu style must resolve its effective font");
    let mut text_font = mission_text_font(assets, skin_name, style_name);
    // Source `JEFFE___14` is a fixed-raster font whose glyphs are wider than
    // the approved vector replacement at the same visible height. Rendering
    // the replacement at 14 px preserves the clean horizontal advances; the
    // measured 0.70 vertical adapter restores the source's 7 px glyph bounds.
    let text_transform = if source_font.path_id == JEFFE_14_SOURCE_FONT_PATH_ID {
        text_font.0.font_size = CENTERED_MENU_JEFFE_FONT_SIZE.into();
        UiTransform::from_scale(Vec2::new(1.0, CENTERED_MENU_JEFFE_VERTICAL_SCALE))
    } else {
        UiTransform::IDENTITY
    };
    let auto_fit = UiTextAutoFit::new(content_rect.width, content_rect.height, &text_font);

    parent
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: if style.text_clipping == 1 {
                    Overflow::clip()
                } else {
                    Overflow::visible()
                },
                ..content_rect.node()
            },
            Pickable::IGNORE,
        ))
        .with_child((
            Text::new(text),
            localized,
            text_font,
            auto_fit,
            TextColor(color_override.unwrap_or_else(|| mission_text_color(style))),
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            MissionTextVerticalAnchor::Middle,
            text_transform,
            Pickable::IGNORE,
        ));
}

pub(super) fn insert_journal_content_rect(
    entity: &mut EntityCommands,
    rect: MissionUiRect,
    view: Option<MissionUiView>,
) {
    if matches!(
        view,
        Some(
            MissionUiView::JournalMissionName
                | MissionUiView::JournalMissionDifficulty
                | MissionUiView::JournalNpcName
                | MissionUiView::JournalNpcPosition
        )
    ) {
        entity.insert(JournalContentRect::standard(rect));
    } else if matches!(view, Some(MissionUiView::JournalObjectiveHeader)) {
        entity.insert(JournalContentRect::objective_header(rect));
    } else if matches!(view, Some(MissionUiView::JournalDescription)) {
        entity.insert(JournalContentRect::description(rect));
    } else if matches!(
        view,
        Some(
            MissionUiView::JournalRewardHeader
                | MissionUiView::JournalRewardAmount
                | MissionUiView::JournalRewardCard
        )
    ) {
        entity.insert(JournalContentRect::reward(rect));
    } else if matches!(
        view,
        Some(MissionUiView::JournalCashAmount | MissionUiView::JournalCashCard)
    ) {
        entity.insert(JournalContentRect::cash(rect));
    }
}

pub(super) fn journal_managed_label(label: &str) -> String {
    format!("{label}:")
}

pub(super) fn journal_objective_label(model: &MissionUiModel) -> &'static str {
    match &model.journal {
        MissionJournalUi::Allow(_) => JOURNAL_MISSION_OFFER_LABEL,
        MissionJournalUi::Other(_) if model.journal_tab == JournalListTab::Completed => {
            JOURNAL_MY_NOTES_LABEL
        }
        MissionJournalUi::Other(_) => JOURNAL_MISSION_DETAILS_LABEL,
        MissionJournalUi::Reward { .. } | MissionJournalUi::Hidden => JOURNAL_MISSION_SUMMARY_LABEL,
    }
}

pub(super) fn text_button(
    parent: &mut ChildSpawnerCommands,
    rect: MissionUiRect,
    text: &str,
    control: MissionUiControl,
    style: MenuButtonStyle,
    assets: &MissionUiAssets,
) {
    let (skin_name, style_name) = style.skin_and_style();
    let normal = if style.is_red() {
        &assets.red_button
    } else {
        &assets.blue_button
    };
    let image = skin_style_border(skin_name, style_name)
        .map_or_else(|| stretch(normal), |border| sliced(normal, border));
    parent
        .spawn((
            Button,
            rect.node(),
            image,
            control,
            NanocomMenuButton(style),
        ))
        .with_children(|button| {
            centered_menu_skin_label(
                button,
                MissionUiRect::new(0.0, 0.0, rect.width, rect.height),
                text,
                skin_name,
                style_name,
                assets,
                None,
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn journal_aux_text_button(
    parent: &mut ChildSpawnerCommands,
    rect: MissionUiRect,
    text: &str,
    style_name: &str,
    kind: JournalAuxButtonKind,
    view: MissionUiView,
    control: MissionUiControl,
    assets: &MissionUiAssets,
) {
    let normal = match kind {
        JournalAuxButtonKind::Blue => &assets.blue_button,
        JournalAuxButtonKind::Red => &assets.red_button,
        JournalAuxButtonKind::Cancel => &assets.cancel_normal,
        JournalAuxButtonKind::Help => &assets.journal_help,
    };
    let image = skin_style_border("FusionFallMissionSkin", style_name)
        .map_or_else(|| stretch(normal), |border| sliced(normal, border));
    parent
        .spawn((Button, rect.node(), image, control, view, kind))
        .with_children(|button| {
            skin_label(
                button,
                MissionUiRect::new(0.0, 0.0, rect.width, rect.height),
                text,
                "FusionFallMissionSkin",
                style_name,
                assets,
                None,
                None,
            );
        });
}

pub(super) fn npc_action_button(
    parent: &mut ChildSpawnerCommands,
    rect: MissionUiRect,
    text: &str,
    icon: &Handle<Image>,
    control: MissionUiControl,
    view: Option<MissionUiView>,
    content_views: Option<(MissionUiView, MissionUiView)>,
    assets: &MissionUiAssets,
) {
    let mut entity = parent.spawn((
        Button,
        rect.node(),
        sliced(
            &assets.npc_blue_button,
            skin_style_border("FusionFallInteractionSkin", "button")
                .expect("Retrobution interaction button border"),
        ),
        control,
        NpcActionButton,
    ));
    if let Some(view) = view {
        entity.insert(view);
    }
    entity.with_children(|button| {
        let mut icon_entity = button.spawn((
            MissionUiRect::new(9.0, 7.5, 38.0, 33.0).node(),
            ImageNode::new(icon.clone()),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        if let Some((_, icon_view)) = content_views {
            icon_entity.insert(icon_view);
        }
        skin_label(
            button,
            MissionUiRect::new(50.0, 0.0, rect.width - 58.0, rect.height),
            text,
            "FusionFallInteractionSkin",
            "button",
            assets,
            content_views.map(|(text_view, _)| text_view),
            None,
        );
    });
}

pub(super) fn centered_node(rect: MissionUiRect) -> Node {
    Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..rect.node()
    }
}
