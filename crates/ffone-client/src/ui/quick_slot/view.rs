use super::*;

pub(super) fn spawn_quick_slot_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    contract: Res<QuickSlotUiAssetContract>,
) {
    let assets = contract
        .resolved()
        .map(|paths| QuickSlotUiAssets::load(&asset_server, paths));
    commands.insert_resource(QuickSlotUiRuntimeAssets(assets.clone()));
    commands
        .spawn((
            QuickSlotUiRoot,
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
            ZIndex(QUICK_SLOT_UI_Z_INDEX),
        ))
        .with_children(|root| {
            let Some(assets) = assets else {
                return;
            };
            root.spawn((
                QuickSlotUiElement::Background,
                QuickSlotUiRect::default().node(),
                ImageNode::new(assets.background.clone()),
                Pickable::IGNORE,
            ));
            for slot in 0..QUICK_SLOT_COUNT {
                root.spawn((
                    Button,
                    QuickSlotButton(slot),
                    QuickSlotUiElement::SlotFrame(slot),
                    QuickSlotUiRect::default().node(),
                    ImageNode::new(assets.empty_style.clone()),
                ));
                root.spawn((
                    QuickSlotUiElement::SlotIcon(slot),
                    QuickSlotUiRect::default().node(),
                    ImageNode::default(),
                    Pickable::IGNORE,
                ));
                root.spawn((
                    QuickSlotUiElement::Cooldown(slot),
                    QuickSlotUiRect::default().node(),
                    ImageNode::new(assets.cooldown.clone()),
                    Pickable::IGNORE,
                ));
            }
        });
}
