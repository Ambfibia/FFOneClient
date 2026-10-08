use super::*;
use ffone_client::option_ui::OPTION_BIG_LABEL_BORDER;

#[derive(Component)]
pub(super) struct ButtonTint(pub(super) Color);
#[derive(Component)]
pub(super) struct GameButton(Color);

pub(super) fn hover(
    fonts:Option<Res<EditorFonts>>,
    mut buttons: Query<(&Interaction, &ButtonTint, &mut BackgroundColor,Option<&GameButton>,Option<&mut ImageNode>), Changed<Interaction>>,
) {
    for (interaction, tint, mut color,skinned,image) in &mut buttons {
        color.0 = if *interaction == Interaction::None {
            tint.0
        } else {
            Color::srgb(0.08, 0.23, 0.42)
        };
        if let Some(skinned)=skinned{if let (Some(fonts),Some(mut image))=(fonts.as_ref(),image){
            image.image=if *interaction==Interaction::Hovered{fonts.button_hover.clone()}else{fonts.button.clone()};
            image.color=if *interaction==Interaction::None{skinned.0}else{Color::srgb(0.25,0.43,0.64)};
        }}
    }
}

pub(super) fn label(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    text: impl Into<String>,
    size: f32,
    color: Color,
) {
    let mut bundle = editor_text(fonts, "ui.editor.xdt.value", "{value}", size, color, false);
    bundle.localized = bundle.localized.with_arg("value", text.into());
    parent.spawn((
        bundle,
        Node {
            width: percent(100),
            min_width: px(0),
            flex_shrink: 0.,
            ..default()
        },
    ));
}
pub(super) fn button(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    action: Action,
    key: &str,
    fallback: &str,
    selected: bool,
    width: f32,
) {
    let primary = matches!(action, Action::Save | Action::Add | Action::Create | Action::Mission(mission_workspace::Command::NewMission));
    let background = if selected || primary {
        Color::srgb(0.045, 0.25, 0.43)
    } else {
        Color::srgb(0.03, 0.075, 0.15)
    };
    let mut image=sliced_image(fonts.button.clone(),OPTION_BIG_LABEL_BORDER);
    image.color=if selected||primary{Color::srgb(0.55,0.8,1.)}else{Color::srgb(0.30,0.46,0.63)};
    parent
        .spawn((
            Button,
            action,
            ButtonTint(background),
            GameButton(image.color),
            image,
            Node {
                min_height: px(34),
                height: px(34),
                width: if width == 0. { percent(100) } else { px(width) },
                min_width: px(0),
                flex_shrink: 0.,
                padding: UiRect::horizontal(px(10)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(Color::srgba(0.12,0.45,0.65,0.4)),
            BackgroundColor(background),
        ))
        .with_children(|p| {
            p.spawn((
                single_line_text(editor_text(
                    fonts,
                    key,
                    fallback,
                    14.,
                    mission_skin::TEXT,
                    false,
                )),
                Node {
                    min_width: px(0),
                    width: percent(100),
                    ..default()
                },
            ));
        });
}
pub(super) fn dynamic_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    action: Action,
    text: String,
    selected: bool,
    width: f32,
) {
    let background = if selected {
        Color::srgb(0.06, 0.21, 0.36)
    } else if matches!(action, Action::Row(_) | Action::Sort(_)) {
        Color::NONE
    } else {
        Color::srgb(0.035, 0.075, 0.14)
    };
    parent
        .spawn((
            Button,
            action,
            ButtonTint(background),
            Node {
                height: px(30),
                min_height: px(30),
                width: if width == 0. { percent(100) } else { px(width) },
                min_width: px(0),
                flex_shrink: 0.,
                padding: UiRect::horizontal(px(10)),
                border_radius: BorderRadius::all(px(4)),
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(background),
        ))
        .with_children(|p| {
            let mut bundle = single_line_text(editor_text(
                fonts,
                "ui.editor.xdt.value",
                "{value}",
                15.,
                Color::srgb(0.87, 0.91, 0.94),
                false,
            ));
            bundle.localized = bundle.localized.with_arg("value", text);
            p.spawn((
                bundle,
                Node {
                    width: percent(100),
                    min_width: px(0),
                    ..default()
                },
            ));
        });
}

// Values can wrap inside a bounded inspector row; toolbar and grid text stays on one line.
pub(super) fn property(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    action: Action,
    name: String,
    value: String,
    selected: bool,
) {
    let background = if selected {
        Color::srgb(0.04, 0.19, 0.32)
    } else {
        Color::srgb(0.035, 0.073, 0.13)
    };
    parent
        .spawn((
            Button,
            action,
            ButtonTint(background),
            Node {
                width: percent(100),
                min_height: px(38),
                flex_shrink: 0.,
                padding: UiRect::axes(px(10), px(8)),
                column_gap: px(12),
                align_items: AlignItems::Start,
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            BackgroundColor(background),
        ))
        .with_children(|line| {
            line.spawn(Node {
                width: percent(43),
                min_width: px(0),
                flex_shrink: 0.,
                ..default()
            })
            .with_children(|p| label(p, fonts, name, 14., Color::srgb(0.60, 0.69, 0.75)));
            line.spawn(Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(0),
                ..default()
            })
            .with_children(|p| label(p, fonts, value, 15., Color::srgb(0.90, 0.94, 0.96)));
        });
}
pub(super) fn field(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    editor: &XdtEditor,
    focus: Focus,
    height: f32,
) {
    let active = editor.focus.as_ref() == Some(&focus);
    let text = if active {
        editor.edit.clone()
    } else {
        match focus {
            Focus::Locale(index)=>editor.workspace.quick_text.as_ref().map(|d|d.values[index].clone()).unwrap_or_default(),
            Focus::Tables => editor.table_search.clone(),
            Focus::Search => editor.search.clone(),
            Focus::Columns => editor.column_search.clone(),
            Focus::ReferenceSearch => editor.reference_search.clone(),
            Focus::NewName => editor
                .draft
                .as_ref()
                .and_then(|d| d.name.as_ref())
                .map(|n| n.text.clone())
                .unwrap_or_default(),
            Focus::Draft(ref f) => editor
                .draft
                .as_ref()
                .and_then(|d| d.value.get(f))
                .map(display)
                .unwrap_or_default(),
            _ => String::new(),
        }
    };
    let placeholder = if !active && text.is_empty() {
        match focus {
            Focus::Tables => Some(("table_placeholder", "Find a table…")),
            Focus::Search => Some(("record_placeholder", "Name or ID…")),
            Focus::Columns => Some(("parameter_placeholder", "Find a parameter…")),
            Focus::ReferenceSearch => Some(("reference_placeholder", "Name or ID…")),
            Focus::NewName => Some(("name_placeholder", "Enter the new record name…")),
            _ => None,
        }
    } else {
        None
    };
    parent
        .spawn((
            Button,
            Action::Field(focus.clone()),
            sliced_image(fonts.textfield.clone(),OPTION_TEXT_FIELD_BORDER),
            Node {
                flex_grow: 1.,
                min_width: px(0),
                height: px(height),
                max_height: px(height),
                flex_shrink: 0.,
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                overflow: Overflow::scroll(),
                ..default()
            },
            ScrollRegion(3),
            input::FieldScrollKey(input::field_scroll_key(editor, &focus)),
            ScrollPosition(editor.workspace.field_scrolls.get(&input::field_scroll_key(editor, &focus))
                .copied().unwrap_or_default()),
            BorderColor::all(if active {
                mission_skin::CYAN
            } else {
                Color::srgb(0.12, 0.27, 0.43)
            }),
            BackgroundColor(Color::srgb(0.018, 0.045, 0.095)),
        ))
        .with_children(|p| {
            let mut bundle = editor_text(
                fonts,
                "ui.editor.xdt.value",
                "{value}",
                15.,
                Color::WHITE,
                false,
            );
            bundle.localized = bundle.localized.with_arg("value", text);
            if height <= 40. {
                bundle.layout = TextLayout::default().with_linebreak(bevy::text::LineBreak::NoWrap);
            }
            if let Some((key, fallback)) = placeholder {
                bundle.localized = LocalizedText::new(format!("ui.editor.xdt.{key}"), fallback);
                bundle.color = TextColor(Color::srgb(0.51, 0.61, 0.68));
            }
            let mut entity = p.spawn((
                bundle,
                EditText,
                ffone_client::localization::UiTextNoAutoFit,
                RelativeCursorPosition::default(),
                Node {
                    width: if height > 40. { percent(100) } else { Val::Auto },
                    min_width: px(20),
                    min_height: px(20),
                    flex_shrink: 0.,
                    ..default()
                },
            ));
            if active {
                entity.insert(ActiveText).with_children(|p| {
                    strings::editing_tools::caret(p, editor.cursor);
                    if editor.cursor != editor.anchor {
                        strings::editing_tools::highlight(
                            p,
                            editor.cursor.min(editor.anchor),
                            editor.cursor.max(editor.anchor),
                        );
                    }
                });
            }
        });
}
pub(super) use super::presentation::draw;
