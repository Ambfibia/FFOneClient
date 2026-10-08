use super::*;

pub(super) fn spawn_inspector_panel(parent: &mut ChildSpawnerCommands, fonts: &EditorFonts) {
    parent
        .spawn((
            panel_node(px(EDITOR_INSPECTOR_WIDTH), fonts.panel.clone()),
            InspectorScroll,
            RelativeCursorPosition::default(),
            ScrollPosition::default(),
        ))
        .with_children(|panel| {
            panel.spawn(editor_text(
                fonts,
                "ui.editor.inspector.title",
                "CHARACTER DATA",
                18.0,
                Color::srgb(0.83, 0.96, 1.0),
                true,
            ));
            spawn_equipment_controls(panel, fonts);
            spawn_action_button(panel, fonts, EditorAction::IconGenerator,
                "ui.editor.icons.open", "Icon generator · 128 × 128", 300.0);
            panel.spawn((NpcInspectorTabs,Node{width:percent(100),column_gap:px(4),flex_shrink:0.,..default()})).with_children(|p|{
                for (tab,key,en) in [(NpcInspectorTab::Details,"ui.editor.details","Details"),(NpcInspectorTab::Animations,"ui.editor.npc.animations","Animations"),(NpcInspectorTab::Edit,"ui.editor.npc.edit","Edit")] {
                    spawn_action_button(p,fonts,EditorAction::NpcInspector(tab),key,en,97.);
                }
            });
            spawn_action_button(
                panel,
                fonts,
                EditorAction::ToggleDetails,
                "ui.editor.details",
                "Details",
                150.0,
            );
            for (role, key, fallback) in [
                (
                    DynamicTextRole::RuntimeStatus,
                    "ui.editor.status.loading",
                    "● LOADING NATIVE ASSET",
                ),
                (
                    DynamicTextRole::InspectorIdentity,
                    "ui.editor.inspector.identity",
                    "Semantic ID\n{semantic}\n\nLogical root\n{logical}",
                ),
                (
                    DynamicTextRole::InspectorSource,
                    "ui.editor.inspector.source",
                    "XDT row: {row}\nNetwork ID: {network}\nSource: {source}",
                ),
                (
                    DynamicTextRole::InspectorGeometry,
                    "ui.editor.inspector.geometry",
                    "Scale: {scale}\nHeight: {height}\nStyle: {style}\nLevel / set: {level}",
                ),
                (
                    DynamicTextRole::InspectorTextures,
                    "ui.editor.inspector.textures",
                    "Main texture: {main}\nSub texture: {sub}",
                ),
                (DynamicTextRole::InspectorPlacements, "ui.editor.npc.placements", "{placements}"),
            ] {
                panel.spawn((
                    editor_text(
                        fonts,
                        key,
                        fallback,
                        if matches!(role, DynamicTextRole::RuntimeStatus) {
                            12.0
                        } else {
                            13.0
                        },
                        if matches!(role, DynamicTextRole::RuntimeStatus) {
                            Color::srgb(0.55, 0.94, 0.16)
                        } else {
                            Color::srgb(0.67, 0.84, 0.89)
                        },
                        false,
                    ),
                    role,
                    DetailsSection(!matches!(role, DynamicTextRole::RuntimeStatus)),
                ));
            }
            panel.spawn((
                Node {
                    width: percent(100),
                    height: px(1),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.22, 0.7, 0.8, 0.32)),
            ));
            panel.spawn((NpcEditSection,Node{width:percent(100),flex_shrink:0.,row_gap:px(6),flex_direction:FlexDirection::Column,..default()}));
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        min_height: px(0),
                        row_gap: px(8),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    CharacterSection,
                ))
                .with_children(|panel| {
                    panel.spawn(editor_text(
                        fonts,
                        "ui.editor.animations.title",
                        "ANIMATION CLIPS",
                        15.0,
                        Color::srgb(0.84, 0.97, 1.0),
                        true,
                    ));
                    panel
                        .spawn(Node {
                            width: percent(100),
                            height: px(32),
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|poses| {
                            spawn_action_button(
                                poses,
                                fonts,
                                EditorAction::DefaultPose,
                                "ui.editor.pose.default",
                                "DEFAULT POSE",
                                143.0,
                            );
                            spawn_action_button(
                                poses,
                                fonts,
                                EditorAction::TPose,
                                "ui.editor.pose.tpose",
                                "T-POSE",
                                143.0,
                            );
                        });
                    panel
                        .spawn(Node {
                            width: percent(100),
                            flex_grow: 1.0,
                            min_height: px(0),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(5),
                            ..default()
                        })
                        .with_children(|list| {
                            for slot in 0..ANIMATION_SLOTS {
                                list.spawn((
                                    Button,
                                    EditorAction::AnimationSlot(slot),
                                    AnimationSlot(slot),
                                    Node {
 border_radius: BorderRadius::all(px(5)),
                                        width: percent(100),
                                        height: px(31),
                                        min_height: px(27),
                                        align_items: AlignItems::Center,
                                        padding: UiRect::horizontal(px(10)),
                                        border: UiRect::all(px(1)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.024, 0.066, 0.081, 0.94)),
                                    BorderColor::all(Color::srgba(0.15, 0.4, 0.47, 0.55)),

                                    EditorButtonSkin,
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        editor_text(
                                            fonts,
                                            "ui.editor.animation.clip",
                                            "{clip}",
                                            12.0,
                                            Color::srgb(0.75, 0.9, 0.94),
                                            false,
                                        ),
                                        DynamicTextRole::AnimationSlot(slot),
                                        EditorButtonLabel,
                                    ));
                                });
                            }
                        });
                    spawn_pager(
                        panel,
                        fonts,
                        EditorAction::AnimationPreviousPage,
                        EditorAction::AnimationNextPage,
                        DynamicTextRole::AnimationPage,
                    );
                    panel.spawn(editor_text(
                        fonts,
                        "ui.editor.help",
                        "↑/↓: select · LMB: orbit · RMB/MMB: pan · Wheel: zoom · D/T: pose · ←/→: clip · F9: language",
                        11.0,
                        Color::srgb(0.4, 0.66, 0.73),
                        false,
                    ));
                });
        });
}

pub(super) fn spawn_pager(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    previous: EditorAction,
    next: EditorAction,
    role: DynamicTextRole,
) {
    parent
        .spawn(Node {
            width: percent(100),
            height: px(32),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|pager| {
            spawn_action_button(
                pager,
                fonts,
                previous,
                "ui.editor.page.previous",
                "PREV",
                76.0,
            );
            pager.spawn((
                editor_text(
                    fonts,
                    "ui.editor.page.value",
                    "{page} / {pages}",
                    12.0,
                    Color::srgb(0.55, 0.78, 0.84),
                    false,
                ),
                role,
            ));
            spawn_action_button(pager, fonts, next, "ui.editor.page.next", "NEXT", 76.0);
        });
}

pub(super) fn spawn_action_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    action: EditorAction,
    key: &'static str,
    fallback: &'static str,
    width: f32,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                border_radius: BorderRadius::all(px(5)),
                width: px(width),
                height: px(30),
                min_height: px(30),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(px(7)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.034, 0.095, 0.115, 0.96)),
            BorderColor::all(Color::srgba(0.2, 0.68, 0.78, 0.58)),
            EditorButtonSkin,
        ))
        .with_children(|button| {
            button.spawn((
                editor_text(
                    fonts,
                    key,
                    fallback,
                    11.5,
                    Color::srgb(0.82, 0.95, 0.98),
                    true,
                ),
                EditorButtonLabel,
            ));
        });
}

pub(super) fn spawn_action_button_with_role(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    action: EditorAction,
    key: &'static str,
    fallback: &'static str,
    width: f32,
    role: DynamicTextRole,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                border_radius: BorderRadius::all(px(5)),
                width: px(width),
                height: px(30),
                min_height: px(30),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(px(7)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.034, 0.095, 0.115, 0.96)),
            BorderColor::all(Color::srgba(0.2, 0.68, 0.78, 0.58)),
            EditorButtonSkin,
        ))
        .with_children(|button| {
            button.spawn((
                editor_text(
                    fonts,
                    key,
                    fallback,
                    11.5,
                    Color::srgb(0.82, 0.95, 0.98),
                    true,
                ),
                role,
                EditorButtonLabel,
            ));
        });
}
