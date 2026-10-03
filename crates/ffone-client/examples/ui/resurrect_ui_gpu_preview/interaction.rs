use super::*;

pub(super) const EXPECTED_INPUT_BOUNDARY: ResurrectInputBoundary = ResurrectInputBoundary {
    blocks_lower_ui: true,
    blocks_gameplay_input: true,
    requires_pointer: true,
    cursor_locked: false,
    mouse_controls_enabled: true,
};

pub(super) fn force_preview_hover(
    mode: Res<PreviewMode>,
    mut buttons: Query<(&Node, &ZIndex, &mut Interaction), With<Button>>,
) {
    for (node, z_index, mut interaction) in &mut buttons {
        let target = node.display == Display::Flex
            && node.left == px(RESURRECT_PHOENIX_BUTTON_RECT.x)
            && node.top == px(RESURRECT_PHOENIX_BUTTON_RECT.y)
            && z_index.0 == mode.hover_z_index();
        *interaction = if target {
            Interaction::Hovered
        } else {
            Interaction::None
        };
    }
}

pub(super) fn button_contract_is_exact(
    mode: PreviewMode,
    assets: &PreviewAssets,
    buttons: &Query<
        (
            &Node,
            &ComputedNode,
            &ZIndex,
            &Interaction,
            &ImageNode,
            &Children,
        ),
        With<Button>,
    >,
    texts: &Query<(&Text, &LocalizedText)>,
) -> bool {
    let normal_id = assets.image(ResurrectTextureRole::ButtonNormal).id();
    let hover_id = assets.image(ResurrectTextureRole::ButtonHover).id();
    let mut total = 0;
    let mut go = 0;
    let mut visible_revive = 0;
    let mut hidden_revive = 0;
    let mut visible_item = 0;
    let mut hidden_item = 0;
    let mut hovered = 0;
    let mut exact = true;

    for (node, computed, z_index, interaction, image, children) in buttons.iter() {
        total += 1;
        let label = children
            .iter()
            .find_map(|child| texts.get(child).ok())
            .map(|(_, localized)| localized.key.as_str());
        let (rect, expected_z, should_be_visible) = match label {
            Some(RESURRECT_GO_LOCALIZATION_KEY) => {
                go += 1;
                (RESURRECT_GO_BUTTON_RECT, 0, true)
            }
            Some(RESURRECT_REVIVE_LOCALIZATION_KEY) => {
                if node.display == Display::Flex {
                    visible_revive += 1;
                } else {
                    hidden_revive += 1;
                }
                (
                    RESURRECT_PHOENIX_BUTTON_RECT,
                    0,
                    node.display == Display::Flex,
                )
            }
            Some(RESURRECT_USE_ITEM_LOCALIZATION_KEY) => {
                let visible = mode == PreviewMode::GroupItem;
                if node.display == Display::Flex {
                    visible_item += 1;
                } else {
                    hidden_item += 1;
                }
                (RESURRECT_PHOENIX_BUTTON_RECT, 2, visible)
            }
            _ => {
                exact = false;
                continue;
            }
        };
        let is_hover_target = should_be_visible
            && rect == RESURRECT_PHOENIX_BUTTON_RECT
            && expected_z == mode.hover_z_index();
        let expected_interaction = if is_hover_target {
            hovered += 1;
            Interaction::Hovered
        } else {
            Interaction::None
        };
        exact &= z_index.0 == expected_z
            && node_matches_rect(node, rect)
            && node.display
                == if should_be_visible {
                    Display::Flex
                } else {
                    Display::None
                }
            && *interaction == expected_interaction
            && image.image.id() == if is_hover_target { hover_id } else { normal_id };
        if should_be_visible {
            exact &= computed.size() == Vec2::new(rect.width, rect.height);
        }
    }

    exact
        && total == 4
        && go == 1
        && visible_revive == 1
        && hidden_revive == 1
        && hovered == 1
        && match mode {
            PreviewMode::PhoenixSelf => visible_item == 0 && hidden_item == 1,
            PreviewMode::GroupItem => visible_item == 1 && hidden_item == 0,
        }
}
