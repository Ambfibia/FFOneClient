use super::*;

impl EnchantUiTextStyle0104 {
    #[must_use]
    pub const fn source_skin_path_id(self) -> i64 {
        match self {
            Self::Jeff12LightBlueUpperLeft
            | Self::Cha12YellowMiddleLeft
            | Self::Jeff8LightBlueMiddleCenter
            | Self::Jeff8LightBlueUpperLeft
            | Self::Cha10SkyBlueMiddleLeft
            | Self::Cha10LightBlueMiddleCenter
            | Self::Jeff12SkyBlueMiddleLeft
            | Self::Cha10LightBlueMiddleLeft
            | Self::Jeff12LightBlueMiddleRight
            | Self::Jeff40LightBlueMiddleCenter
            | Self::Char12BlueMiddleCenter
            | Self::Cha10BlueUpperLeft
            | Self::Cha12YellowMiddleRight
            | Self::EnchantLevelMiddleCenter
            | Self::EnchantDefaultLabelUpperLeft
            | Self::EnchantDefaultButtonMiddleCenter => ENCHANT_SKIN_PATH_ID,
            Self::InventoryLabelUpperLeft
            | Self::InventoryBlankBoxUpperLeft
            | Self::InventoryBlankBoxMiddleRight
            | Self::InventoryButtonMiddleCenter
            | Self::InventoryEquipBarMiddleCenter
            | Self::InventoryEquipFontMiddleRight => ENCHANT_INVENTORY_SKIN_PATH_ID_0104,
        }
    }

    #[must_use]
    pub const fn source_style_name(self) -> &'static str {
        match self {
            Self::Jeff12LightBlueUpperLeft => "Jeff12LightBlue",
            Self::Cha12YellowMiddleLeft => "Cha12Yellow",
            Self::Jeff8LightBlueMiddleCenter => "Jeff8LightBlue",
            Self::Jeff8LightBlueUpperLeft => "Jeff8LightBlueLeft",
            Self::Cha10SkyBlueMiddleLeft => "Cha10SkyBlue",
            Self::Cha10LightBlueMiddleCenter => "Cha10LightBlue",
            Self::Jeff12SkyBlueMiddleLeft => "Jeff12SkyBlue",
            Self::Cha10LightBlueMiddleLeft => "Cha10LightBlueLeft",
            Self::Jeff12LightBlueMiddleRight => "Jeff12LightBlueRight",
            Self::Jeff40LightBlueMiddleCenter => "Jeff40LightBlue",
            Self::Char12BlueMiddleCenter => "Char12Blue",
            Self::Cha10BlueUpperLeft => "Cha10Blue",
            Self::Cha12YellowMiddleRight => "Cha12YellowRight",
            Self::EnchantLevelMiddleCenter => "EnchantLevel",
            Self::EnchantDefaultLabelUpperLeft | Self::InventoryLabelUpperLeft => "label",
            Self::EnchantDefaultButtonMiddleCenter | Self::InventoryButtonMiddleCenter => "button",
            Self::InventoryBlankBoxUpperLeft | Self::InventoryBlankBoxMiddleRight => "blankbox",
            Self::InventoryEquipBarMiddleCenter => "equipbar",
            Self::InventoryEquipFontMiddleRight => "equipfont",
        }
    }

    #[must_use]
    pub const fn source_font_path_id(self) -> i64 {
        match self {
            Self::Jeff12LightBlueUpperLeft
            | Self::Jeff12SkyBlueMiddleLeft
            | Self::Jeff12LightBlueMiddleRight => ENCHANT_JEFFE_13_FONT_PATH_ID_0104,
            Self::Cha12YellowMiddleLeft
            | Self::Char12BlueMiddleCenter
            | Self::Cha12YellowMiddleRight => ENCHANT_CHALET_14_FONT_PATH_ID_0104,
            Self::Jeff8LightBlueMiddleCenter
            | Self::Jeff8LightBlueUpperLeft
            | Self::EnchantDefaultButtonMiddleCenter
            | Self::InventoryLabelUpperLeft
            | Self::InventoryBlankBoxUpperLeft
            | Self::InventoryBlankBoxMiddleRight => ENCHANT_JEFFE_12_FONT_PATH_ID_0104,
            Self::Cha10SkyBlueMiddleLeft
            | Self::Cha10LightBlueMiddleCenter
            | Self::Cha10LightBlueMiddleLeft
            | Self::Cha10BlueUpperLeft
            | Self::EnchantLevelMiddleCenter => ENCHANT_CHALET_SMALL_FONT_PATH_ID_0104,
            Self::Jeff40LightBlueMiddleCenter => ENCHANT_JEFFE_40_FONT_PATH_ID_0104,
            Self::EnchantDefaultLabelUpperLeft => ENCHANT_JEFFE_08_FONT_PATH_ID_0104,
            Self::InventoryButtonMiddleCenter => ENCHANT_INVENTORY_BUTTON_FONT_PATH_ID_0104,
            Self::InventoryEquipBarMiddleCenter | Self::InventoryEquipFontMiddleRight => {
                ENCHANT_INVENTORY_SMALL_FONT_PATH_ID_0104
            }
        }
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        match self {
            Self::Jeff12LightBlueUpperLeft
            | Self::Jeff12SkyBlueMiddleLeft
            | Self::Jeff12LightBlueMiddleRight => 13.0,
            Self::Cha12YellowMiddleLeft
            | Self::Char12BlueMiddleCenter
            | Self::Cha12YellowMiddleRight => 14.0,
            Self::Jeff8LightBlueMiddleCenter
            | Self::Jeff8LightBlueUpperLeft
            | Self::Cha10SkyBlueMiddleLeft
            | Self::Cha10LightBlueMiddleCenter
            | Self::Cha10LightBlueMiddleLeft
            | Self::Cha10BlueUpperLeft
            | Self::EnchantLevelMiddleCenter
            | Self::EnchantDefaultButtonMiddleCenter
            | Self::InventoryLabelUpperLeft
            | Self::InventoryBlankBoxUpperLeft
            | Self::InventoryBlankBoxMiddleRight => 12.0,
            Self::Jeff40LightBlueMiddleCenter => 40.0,
            Self::EnchantDefaultLabelUpperLeft => 8.0,
            Self::InventoryButtonMiddleCenter => 11.0,
            Self::InventoryEquipBarMiddleCenter | Self::InventoryEquipFontMiddleRight => 7.0,
        }
    }

    #[must_use]
    pub const fn line_height(self) -> f32 {
        match self {
            Self::Jeff12LightBlueUpperLeft
            | Self::Jeff12SkyBlueMiddleLeft
            | Self::Jeff12LightBlueMiddleRight => 14.689_999_58,
            Self::Cha12YellowMiddleLeft
            | Self::Char12BlueMiddleCenter
            | Self::Cha12YellowMiddleRight => 15.819_999_69,
            Self::Jeff8LightBlueMiddleCenter
            | Self::Jeff8LightBlueUpperLeft
            | Self::EnchantDefaultButtonMiddleCenter
            | Self::InventoryLabelUpperLeft
            | Self::InventoryBlankBoxUpperLeft
            | Self::InventoryBlankBoxMiddleRight => 13.560_000_42,
            Self::Cha10SkyBlueMiddleLeft
            | Self::Cha10LightBlueMiddleCenter
            | Self::Cha10LightBlueMiddleLeft
            | Self::Cha10BlueUpperLeft
            | Self::EnchantLevelMiddleCenter => 13.560_000_42,
            Self::Jeff40LightBlueMiddleCenter => 45.200_000_76,
            Self::EnchantDefaultLabelUpperLeft => 9.039_999_96,
            Self::InventoryButtonMiddleCenter => 11.300_000_19,
            Self::InventoryEquipBarMiddleCenter | Self::InventoryEquipFontMiddleRight => {
                6.780_000_21
            }
        }
    }

    /// Unity `TextAnchor`: 0 UpperLeft, 3 MiddleLeft, 4 MiddleCenter,
    /// 5 MiddleRight.
    #[must_use]
    pub const fn legacy_alignment(self) -> i32 {
        match self {
            Self::Jeff12LightBlueUpperLeft
            | Self::Jeff8LightBlueUpperLeft
            | Self::Cha10BlueUpperLeft
            | Self::EnchantDefaultLabelUpperLeft
            | Self::InventoryLabelUpperLeft
            | Self::InventoryBlankBoxUpperLeft => 0,
            Self::Cha12YellowMiddleLeft
            | Self::Cha10SkyBlueMiddleLeft
            | Self::Jeff12SkyBlueMiddleLeft
            | Self::Cha10LightBlueMiddleLeft => 3,
            Self::Jeff8LightBlueMiddleCenter
            | Self::Cha10LightBlueMiddleCenter
            | Self::Jeff40LightBlueMiddleCenter
            | Self::Char12BlueMiddleCenter
            | Self::EnchantLevelMiddleCenter
            | Self::EnchantDefaultButtonMiddleCenter
            | Self::InventoryButtonMiddleCenter
            | Self::InventoryEquipBarMiddleCenter => 4,
            Self::Jeff12LightBlueMiddleRight
            | Self::Cha12YellowMiddleRight
            | Self::InventoryBlankBoxMiddleRight
            | Self::InventoryEquipFontMiddleRight => 5,
        }
    }

    #[must_use]
    pub const fn word_wrap(self) -> bool {
        matches!(
            self,
            Self::Cha12YellowMiddleLeft
                | Self::Jeff8LightBlueMiddleCenter
                | Self::Jeff8LightBlueUpperLeft
                | Self::Cha10SkyBlueMiddleLeft
                | Self::Cha10LightBlueMiddleCenter
                | Self::Cha10LightBlueMiddleLeft
                | Self::Jeff40LightBlueMiddleCenter
                | Self::Char12BlueMiddleCenter
                | Self::Cha10BlueUpperLeft
                | Self::EnchantDefaultLabelUpperLeft
                | Self::InventoryLabelUpperLeft
        )
    }

    #[must_use]
    pub const fn clipping(self) -> bool {
        matches!(
            self,
            Self::EnchantDefaultButtonMiddleCenter
                | Self::InventoryLabelUpperLeft
                | Self::InventoryButtonMiddleCenter
        )
    }

    /// Serialized `[left, right, top, bottom]` `RectOffset`.
    #[must_use]
    pub const fn padding(self) -> [f32; 4] {
        match self {
            Self::EnchantDefaultLabelUpperLeft | Self::InventoryLabelUpperLeft => {
                [0.0, 0.0, 3.0, 3.0]
            }
            Self::EnchantDefaultButtonMiddleCenter | Self::InventoryButtonMiddleCenter => {
                [6.0, 6.0, 3.0, 3.0]
            }
            _ => [0.0; 4],
        }
    }

    #[must_use]
    pub const fn content_offset(self) -> [f32; 2] {
        [0.0, 0.0]
    }

    /// Per-style replacement-font calibration. GPU bounds tests pin every
    /// reached role; none needs an additional vertical correction.
    #[must_use]
    pub const fn replacement_y_offset(self) -> f32 {
        0.0
    }

    pub(super) fn uses_chalet(self) -> bool {
        matches!(
            self,
            Self::Cha12YellowMiddleLeft
                | Self::Cha10SkyBlueMiddleLeft
                | Self::Cha10LightBlueMiddleCenter
                | Self::Cha10LightBlueMiddleLeft
                | Self::Char12BlueMiddleCenter
                | Self::Cha10BlueUpperLeft
                | Self::Cha12YellowMiddleRight
                | Self::EnchantLevelMiddleCenter
        )
    }

    pub(super) fn font(self, assets: &EnchantUiAssets0104) -> (TextFont, LineHeight) {
        (
            TextFont {
                font: (if self.uses_chalet() {
                    assets.chalet_font.clone()
                } else {
                    assets.jeffe_font.clone()
                })
                .into(),
                font_size: (self.font_size()).into(),
                ..default()
            },
            LineHeight::Px(self.line_height()),
        )
    }

    pub(super) fn apply_to_node(self, node: &mut Node) {
        let [left, right, top, bottom] = self.padding();
        node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
        match self.legacy_alignment() {
            0 => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::FlexStart;
            }
            3 => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::Center;
            }
            4 => {
                node.justify_content = JustifyContent::Center;
                node.align_items = AlignItems::Center;
            }
            5 => {
                node.justify_content = JustifyContent::FlexEnd;
                node.align_items = AlignItems::Center;
            }
            _ => unreachable!("reached Enchant style has an unsupported TextAnchor"),
        }
        node.overflow = if self.clipping() {
            Overflow::clip()
        } else {
            Overflow::visible()
        };
    }

    pub(super) fn node(self, rect: EnchantUiRect0104) -> Node {
        let mut node = rect.node();
        self.apply_to_node(&mut node);
        node
    }

    pub(super) fn layout(self) -> TextLayout {
        let justify = match self.legacy_alignment() {
            0 | 3 => Justify::Left,
            4 => Justify::Center,
            5 => Justify::Right,
            _ => unreachable!("reached Enchant style has an unsupported TextAnchor"),
        };
        TextLayout::new(
            justify,
            if self.word_wrap() {
                LineBreak::WordBoundary
            } else {
                LineBreak::NoWrap
            },
        )
    }
}

#[derive(Clone, Resource)]
pub(super) struct EnchantUiAssets0104 {
    pub(super) images: [Handle<Image>; EnchantStaticAssetRole0104::COUNT],
    pub(super) jeffe_font: Handle<Font>,
    pub(super) chalet_font: Handle<Font>,
}

impl EnchantUiAssets0104 {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            images: array::from_fn(|index| {
                asset_server.load(ENCHANT_UI_DEFAULT_IMAGE_PATHS_0104[index])
            }),
            jeffe_font: asset_server.load(ENCHANT_JEFFE_FONT_PATH),
            chalet_font: asset_server.load(ENCHANT_CHALET_FONT_PATH),
        }
    }

    pub(super) fn image(&self, role: EnchantStaticAssetRole0104) -> Handle<Image> {
        self.images[role.index()].clone()
    }

    pub(super) fn readiness(&self, asset_server: &AssetServer) -> EnchantStaticAssetReadiness0104 {
        let failed = self
            .images
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
            || matches!(
                asset_server.load_state(self.jeffe_font.id()),
                LoadState::Failed(_)
            )
            || matches!(
                asset_server.load_state(self.chalet_font.id()),
                LoadState::Failed(_)
            );
        if failed {
            return EnchantStaticAssetReadiness0104::Failed;
        }
        let loaded = self
            .images
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(
                asset_server.load_state(self.jeffe_font.id()),
                LoadState::Loaded
            )
            && matches!(
                asset_server.load_state(self.chalet_font.id()),
                LoadState::Loaded
            );
        if loaded {
            EnchantStaticAssetReadiness0104::Ready
        } else {
            EnchantStaticAssetReadiness0104::Loading
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnchantSuccessPresentation0104 {
    pub item: ItemBase0104,
    pub presentation: EnchantItemPresentation0104,
    pub displayed_enchant_level: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantUiCommand0104 {
    BeginInventoryDrag(usize),
    DropOnAttachment(EnchantAttachmentSlot0104),
    DetachAttachment(EnchantAttachmentSlot0104),
    Preview,
    Clear,
    Enchant,
    Close,
    DropOnTrash,
    Help,
    OpenRedeemCode,
    EnchantMoreItems,
    GoToMyStuff,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct EnchantUiOutbox0104 {
    pub(super) commands: VecDeque<EnchantUiCommand0104>,
}

impl EnchantUiOutbox0104 {
    pub fn push(&mut self, command: EnchantUiCommand0104) {
        self.commands.push_back(command);
    }

    pub fn pop_front(&mut self) -> Option<EnchantUiCommand0104> {
        self.commands.pop_front()
    }

    #[must_use]
    pub fn as_slices(&self) -> (&[EnchantUiCommand0104], &[EnchantUiCommand0104]) {
        self.commands.as_slices()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum EnchantInteractiveControl0104 {
    InventorySlot(usize),
    Attachment(EnchantAttachmentSlot0104),
    Preview,
    Clear,
    Enchant,
    Close,
    Trash,
    Help,
    RedeemCode,
    EnchantMoreItems,
    GoToMyStuff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum EnchantUiElement0104 {
    Backdrop,
    RightBackplate,
    MainGroup,
    Panel,
    PrimaryNpcBoundary,
    Title,
    Intro,
    TargetTitle,
    NeededTitle,
    HelpTitle,
    ChanceTitle,
    Chance,
    TarosTitle,
    Taros,
    AttachmentFrame(EnchantAttachmentSlot0104),
    AttachmentIcon(EnchantAttachmentSlot0104),
    AttachmentCount(EnchantAttachmentSlot0104),
    MaterialRequiredCount(EnchantAttachmentSlot0104),
    TargetCombinedBadge,
    TargetLevelBadge,
    TargetLevelText,
    TargetName,
    TargetDescription,
    MaterialXMark(EnchantAttachmentSlot0104),
    MaterialQuantityCover(EnchantAttachmentSlot0104),
    MaterialQuantityWarning(EnchantAttachmentSlot0104),
    SupportName(EnchantAttachmentSlot0104),
    SupportDescription(EnchantAttachmentSlot0104),
    RawCover,
    EmptyPrompt,
    Preview,
    Clear,
    Enchant,
    PcStuffPanel,
    ItemTabLabel,
    InventoryViewport,
    InventoryContent,
    InventorySlotFrame(usize),
    InventorySlotIcon(usize),
    InventorySlotBadge(usize),
    InventorySlotCount(usize),
    EquipmentPanel,
    EquipmentTitle,
    EquipmentTitleLabel,
    EquipmentSlotFrame(usize),
    EquipmentSlotIcon(usize),
    EquipmentSlotBadge(usize),
    EquipmentSlotLabel(usize),
    Close,
    Trash,
    Help,
    DexlabsBanner,
    TarosCounter,
    TarosDigit(usize),
    RedeemCode,
    BatterySlotFrame(usize),
    BatteryIcon(usize),
    BatteryLabel(usize),
    BatteryCount(usize),
    Shade,
    WaitingGroup,
    WaitingLabel,
    WaitingNpcBoundary,
    WaitingProgress,
    SuccessGroup,
    SuccessNpc,
    SuccessHooray,
    SuccessMessage,
    SuccessIcon,
    SuccessLevelBadge,
    SuccessLevelText,
    SuccessName,
    SuccessLevel,
    SuccessDescription,
    SuccessPoint,
    SuccessGroupRating,
    SuccessDefense,
    SuccessTypeLabel,
    SuccessRangeLabel,
    SuccessRarityLabel,
    SuccessTradeLabel,
    SuccessTypeValue,
    SuccessRangeValue,
    SuccessRarityValue,
    SuccessTradeValue,
    EnchantMoreItems,
    GoToMyStuff,
}

#[derive(Component)]
pub struct EnchantUiRoot0104;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum EnchantUiSet0104 {
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct EnchantUiPlugin0104;

impl Plugin for EnchantUiPlugin0104 {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<EnchantModeProjection0104>()
            .init_resource::<EnchantUiOutbox0104>()
            .init_resource::<EnchantInventoryUiState0104>()
            .init_resource::<EnchantUiAssetStatus0104>()
            .configure_sets(
                Update,
                (EnchantUiSet0104::Interaction, EnchantUiSet0104::Bind).chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_enchant_ui_0104,
            )
            .add_systems(
                Update,
                (advance_enchant_inventory_ui_0104
                    .before(collect_enchant_ui_input_0104)
                    .in_set(EnchantUiSet0104::Interaction))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (collect_enchant_ui_input_0104.in_set(EnchantUiSet0104::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((bind_enchant_ui_0104, bind_enchant_button_labels_0104)
                    .chain()
                    .in_set(EnchantUiSet0104::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
