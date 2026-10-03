use super::*;

pub(super) fn spawn_skill_buff_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<SkillBuffUiCatalog>,
) {
    let mut icons = BTreeMap::new();
    for definition in catalog.definitions.values() {
        for path in [&definition.icon_path, &definition.cash_icon_path] {
            icons
                .entry(path.clone())
                .or_insert_with(|| asset_server.load(path.clone()));
        }
    }
    let assets = SkillBuffUiAssets {
        background: asset_server.load(SKILL_BUFF_BACK_PATH),
        font: asset_server.load(SKILL_BUFF_FONT_PATH),
        icons,
    };
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            SkillBuffUiRoot,
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
            ZIndex(SKILL_BUFF_UI_Z_INDEX),
        ))
        .with_children(|root| {
            for row in [
                SkillBuffRow::Local,
                SkillBuffRow::Cash,
                SkillBuffRow::Target,
            ] {
                for index in 0..SKILL_BUFF_MAX_ICONS {
                    spawn_skill_buff_icon_slot(root, row, index, &assets);
                }
            }
        });
}

pub(super) fn spawn_skill_buff_icon_slot(
    parent: &mut ChildSpawnerCommands,
    row: SkillBuffRow,
    index: usize,
    assets: &SkillBuffUiAssets,
) {
    parent
        .spawn((
            SkillBuffUiElement::Slot(row, index),
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|slot| {
            slot.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ImageNode {
                    image: assets.background.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Pickable::IGNORE,
            ));
            slot.spawn((
                SkillBuffUiElement::Icon(row, index),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ImageNode::default(),
                Pickable::IGNORE,
            ));
            if row == SkillBuffRow::Cash {
                let style = SkillBuffTextStyle::HudLabel;
                let spec = style.spec();
                slot.spawn((
                    SkillBuffUiElement::CashTime(index),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(2),
                        top: px(19.0 + spec.y_offset),
                        width: px(SKILL_BUFF_ICON_SIZE),
                        height: px(SKILL_BUFF_ICON_SIZE),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|label| {
                    label.spawn((
                        SkillBuffUiElement::CashText(index),
                        style,
                        Text::new(""),
                        skill_buff_cash_time_localized(""),
                        (
                            TextFont {
                                font: (assets.font.clone()).into(),
                                font_size: (spec.font_size).into(),
                                ..default()
                            },
                            LineHeight::Px(spec.line_height),
                        ),
                        TextColor(Color::srgba(
                            spec.normal_color[0],
                            spec.normal_color[1],
                            spec.normal_color[2],
                            spec.normal_color[3],
                        )),
                        TextLayout {
                            justify: spec.justify,
                            linebreak: spec.linebreak,
                        },
                        Pickable::IGNORE,
                    ));
                });
            }
        });
}
