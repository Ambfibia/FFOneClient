use super::*;

impl Pc2pcTextStyle0104 {
    pub(super) fn font(self, assets: &Pc2pcUiAssets) -> (TextFont, LineHeight) {
        let (font, font_size, line_height) = match self {
            Self::Button | Self::ChatSendButton => (
                assets.jeffe_font.clone(),
                PC2PC_JEFFE_14_FONT_SIZE,
                PC2PC_JEFFE_14_LINE_HEIGHT,
            ),
            Self::ReadyLeftLabel => (
                assets.jeffe_font.clone(),
                PC2PC_JEFFE_16_FONT_SIZE,
                PC2PC_JEFFE_16_LINE_HEIGHT,
            ),
            Self::ChatLine => (
                assets.chalet_font.clone(),
                PC2PC_CHALET_SMALL_FONT_SIZE,
                PC2PC_CHAT_LINE_HEIGHT,
            ),
            Self::ChatInput => (
                assets.chalet_font.clone(),
                PC2PC_CHALET_SMALL_FONT_SIZE,
                PC2PC_CHALET_SMALL_LINE_HEIGHT,
            ),
            Self::EquipmentTitle | Self::EquipmentSlot => (
                assets.jeffe_font.clone(),
                USER_EQUIP_SMALL_FONT_SIZE,
                USER_EQUIP_SMALL_FONT_LINE_HEIGHT,
            ),
            Self::InventoryTab => (
                assets.jeffe_font.clone(),
                USER_EQUIP_TAB_FONT_SIZE,
                USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
            ),
            Self::InventoryCount => (
                assets.jeffe_font.clone(),
                USER_EQUIP_COUNT_FONT_SIZE,
                USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
            ),
            Self::LabelUpperLeft | Self::LabelMiddleRight | Self::RightLabel => (
                assets.jeffe_font.clone(),
                PC2PC_JEFFE_12_FONT_SIZE,
                PC2PC_JEFFE_12_LINE_HEIGHT,
            ),
        };
        (
            TextFont {
                font: (font).into(),
                font_size: (font_size).into(),
                ..default()
            },
            LineHeight::Px(line_height),
        )
    }

    pub(super) fn node(self, rect: Pc2pcUiRect) -> Node {
        let mut node = rect.node();
        self.apply_to_node(&mut node);
        node
    }

    pub(super) fn apply_to_node(self, node: &mut Node) {
        match self {
            Self::LabelUpperLeft => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::FlexStart;
                node.padding = UiRect::new(
                    px(0),
                    px(0),
                    px(PC2PC_LABEL_PADDING_TOP),
                    px(PC2PC_LABEL_PADDING_BOTTOM),
                );
                node.overflow = Overflow::clip();
            }
            Self::LabelMiddleRight => {
                node.justify_content = JustifyContent::FlexEnd;
                node.align_items = AlignItems::Center;
                node.padding = UiRect::new(
                    px(0),
                    px(0),
                    px(PC2PC_LABEL_PADDING_TOP),
                    px(PC2PC_LABEL_PADDING_BOTTOM),
                );
                node.overflow = Overflow::clip();
            }
            Self::RightLabel => {
                node.justify_content = JustifyContent::FlexEnd;
                node.align_items = AlignItems::Center;
            }
            Self::Button => {
                node.justify_content = JustifyContent::Center;
                node.align_items = AlignItems::Center;
                node.padding = UiRect::new(
                    px(PC2PC_BUTTON_PADDING_LEFT),
                    px(PC2PC_BUTTON_PADDING_RIGHT),
                    px(PC2PC_BUTTON_PADDING_TOP),
                    px(PC2PC_BUTTON_PADDING_BOTTOM),
                );
                node.overflow = Overflow::clip();
            }
            Self::ChatLine | Self::ReadyLeftLabel | Self::InventoryTab => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::FlexStart;
            }
            Self::ChatInput => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::Center;
            }
            Self::ChatSendButton | Self::EquipmentTitle => {
                node.justify_content = JustifyContent::Center;
                node.align_items = AlignItems::Center;
            }
            Self::InventoryCount => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::FlexStart;
            }
            Self::EquipmentSlot => {
                node.justify_content = JustifyContent::Center;
                node.align_items = AlignItems::FlexEnd;
            }
        }
    }

    pub(super) fn layout(self) -> TextLayout {
        let (justify, linebreak) = match self {
            Self::LabelMiddleRight | Self::RightLabel => (Justify::Right, LineBreak::WordBoundary),
            Self::Button | Self::ChatSendButton | Self::EquipmentTitle | Self::EquipmentSlot => {
                (Justify::Center, LineBreak::NoWrap)
            }
            Self::ChatInput | Self::InventoryTab | Self::InventoryCount => {
                (Justify::Left, LineBreak::NoWrap)
            }
            Self::LabelUpperLeft | Self::ChatLine | Self::ReadyLeftLabel => {
                (Justify::Left, LineBreak::WordBoundary)
            }
        };
        TextLayout::new(justify, linebreak)
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum Pc2pcDisabledControl {
    AddTarosNeedsNumericPopup,
    ChatNeedsBackend,
    MenuChatNeedsBackend,
    PortraitNeedsBackend,
    NanoTabUnavailableInTrade,
    CloseOwnedBySessionController,
    TrashUnavailableInTrade,
    HelpOwnedByShell,
    ItemMutationOwnedByIntentBoundary,
}

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pc2pcUiSet {
    Animate,
    Interaction,
    Bind,
}

pub struct Pc2pcUiPlugin;

impl Plugin for Pc2pcUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.add_message::<bevy::input::keyboard::KeyboardInput>();
        app.init_resource::<Pc2pcUiAssetContract>()
            .init_resource::<Pc2pcUiModel0104>()
            .init_resource::<gestures::TarosEditor>()
            .init_resource::<crate::gameplay_ui::GameplayUiAudioOutbox>()
            .init_resource::<Pc2pcOfferRuntime0104>()
            .init_resource::<Pc2pcModalState>()
            .insert_resource(Pc2pcBackendCapabilities { numeric_popup_backend: true, ..default() })
            .init_resource::<Pc2pcPortraitBindings>()
            .init_resource::<Pc2pcStaticAssetReadiness>()
            .configure_sets(
                Update,
                (Pc2pcUiSet::Animate, Pc2pcUiSet::Interaction, Pc2pcUiSet::Bind)
                    .chain()
                    .before(LocalizationSet::Apply),
            )
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_pc2pc_ui)
            .add_systems(
                Update,
                ((
                    (gestures::enable_controls, gestures::spawn_editor).in_set(Pc2pcUiSet::Animate),
                    gestures::interact.in_set(Pc2pcUiSet::Interaction),
                    gestures::bind_editor.in_set(Pc2pcUiSet::Bind),
                    tick_pc2pc_state.in_set(Pc2pcUiSet::Animate),
                    bind_pc2pc_ui.in_set(Pc2pcUiSet::Bind),
                ))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
