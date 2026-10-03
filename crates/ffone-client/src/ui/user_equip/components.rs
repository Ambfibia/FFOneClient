//! Assets, UI element/control components, system set and editable layout asset.

use super::UserEquipEditableLayout;
use super::asset_contract::{UserEquipStaticAssetRole, UserEquipUiAssetContract};
use super::catalog::UserEquipSlotEndpoint;
use super::images::user_equip_missing_checker_image;
use super::source_evidence::USER_EQUIP_EDITABLE_LAYOUT_PATH;
use super::state::UserEquipAvatarTurnDirection;
use bevy::{asset::LoadState, prelude::*};
use std::array;

#[derive(Clone, Resource)]
pub(super) struct UserEquipUiAssets {
    pub(super) images: [Handle<Image>; 56],
    pub(super) font: Handle<Font>,
    pub(super) missing_checker: Handle<Image>,
}

impl UserEquipUiAssets {
    pub(super) fn load(
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        contract: &UserEquipUiAssetContract,
    ) -> Self {
        let paths = contract.paths();
        Self {
            images: array::from_fn(|index| asset_server.load(paths[index].to_owned())),
            font: asset_server.load(paths[UserEquipStaticAssetRole::Font.index()].to_owned()),
            missing_checker: images.add(user_equip_missing_checker_image()),
        }
    }

    pub(super) fn image(&self, role: UserEquipStaticAssetRole) -> Handle<Image> {
        assert!(!role.is_font(), "font role has no Image handle");
        self.images[role.index()].clone()
    }

    pub(super) fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.images
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(asset_server.load_state(self.font.id()), LoadState::Loaded)
    }
}

#[derive(Clone, Resource)]
pub(super) struct UserEquipUiRuntimeAssets(pub(super) UserEquipUiAssets);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct UserEquipUiRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum UserEquipUiElement {
    Backdrop,
    ClothesBackplate,
    RightBackplate,
    UserClothesPanel,
    AvatarPreview,
    TurnLeftPositioned,
    TurnRightPositioned,
    StatusPanel,
    StatusHpBack,
    StatusHpBar,
    StatusFusionBack,
    StatusFusionBar,
    StatusGuideBox,
    PcStuffPanel,
    PcStuffPanelBackground,
    EquipmentPanel,
    ItemTab,
    NanoTab,
    ItemTabLabel,
    NanoTabLabel,
    InventoryViewport,
    InventoryShadowA,
    InventoryShadowB,
    InventoryContent,
    InventorySlotFrame(usize),
    InventorySlotIcon(usize),
    InventorySlotBadge(usize),
    InventorySlotCount(usize),
    NanoViewport,
    NanoContent,
    NanoSlotFrame(usize),
    NanoSlotIcon(usize),
    ScrollTrack,
    ScrollUp,
    ScrollDown,
    ScrollThumb,
    Close,
    Trash,
    Help,
    EquipmentTitle,
    EquipmentTitleLabel,
    EquipmentSlotFrame(usize),
    EquipmentSlotIcon(usize),
    EquipmentSlotBadge(usize),
    EquipmentSlotLabel(usize),
    BoostSlot,
    BoostIcon,
    BoostLabel,
    BoostValue,
    PotionSlot,
    PotionIcon,
    PotionLabel,
    PotionValue,
    TarosBack,
    TarosDigit(usize),
    Dexlabs,
    StatusName,
    StatusLevel,
    StatusHp,
    StatusFusionMatter,
    StatusGuideLabel,
    StatusGuideName,
    StatusGuideIcon,
    StatusHpValue,
    StatusFusionMatterValue,
    NanoStatusPortrait(usize),
    NanoStatusPanel(usize),
    NanoStatusType(usize),
    NanoStatusSlotLabel(usize),
    NanoStatusName(usize),
    NanoStatusAttribute(usize),
    NanoStatusSkill(usize),
    NanoStatusStamina(usize),
    ItemPopup,
    ItemPopupBackdrop,
    ItemPopupEquipInfo,
    ItemPopupTitle,
    ItemPopupClose,
    ItemPopupTrash,
    ItemPopupButton(usize),
    ItemPopupButtonLabel(usize),
    ItemPopupIconFrame,
    ItemPopupIcon,
    ItemPopupIdentity,
    ItemPopupField(usize),
    ItemPopupCalculatorBack,
    ItemPopupAmountLabel,
    ItemPopupAmountValue,
    ItemPopupKeypadLabel(usize),
    ItemPopupGumNanoFrame(usize),
    ItemPopupGumNanoIcon(usize),
    ItemPopupGumNanoButton(usize),
    ItemPopupGumNanoButtonLabel(usize),
    DraggedItem,
    NanoViewer,
    NanoViewerInnerBackdrop,
    NanoViewerEquippedTitle,
    NanoViewerIcon,
    NanoViewerName,
    NanoViewerAttribute,
    NanoViewerDescription,
    NanoViewerCurrentPower,
    NanoViewerSkillIcon(usize),
    NanoViewerSkillName(usize),
    NanoViewerSkillType(usize),
    NanoViewerSkillDescription(usize),
    NanoViewerRequirementPrompt(usize),
    NanoViewerRequirementLabel(usize, usize),
    NanoViewerRequirementBar(usize, usize),
    NanoViewerRequirementValue(usize, usize),
    NanoViewerStationNotice,
    NanoViewerClose,
    HelpPanel,
    HelpTitle,
    HelpBody,
    HelpClose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
#[require(crate::ui::shared::controller::ControllerUiClose)]
pub(super) struct UserEquipCloseControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipSlotControl(pub(super) UserEquipSlotEndpoint);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipEquipmentSlotLabelText(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
#[require(crate::ui::shared::controller::ControllerUiTab(0))]
pub(super) struct UserEquipItemTabControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
#[require(crate::ui::shared::controller::ControllerUiTab(1))]
pub(super) struct UserEquipNanoTabControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipNanoSlotControl(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
#[require(crate::ui::shared::controller::ControllerUiClose)]
pub(super) struct UserEquipNanoViewerCloseControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipScrollUpControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipScrollDownControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipScrollTrackControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipScrollThumbControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipAvatarTurnControl(pub(super) UserEquipAvatarTurnDirection);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipTrashControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipHelpControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
#[require(crate::ui::shared::controller::ControllerUiClose)]
pub(super) struct UserEquipHelpCloseControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipPopupButtonControl(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipGumNanoButtonControl(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
#[require(crate::ui::shared::controller::ControllerUiClose)]
pub(super) struct UserEquipPopupCloseControl;

/// Text is kept on a child so the parent can reproduce Unity GUIStyle's
/// TextAnchor against the exact serialized Rect. Bevy text nodes otherwise
/// shrink to their glyph width and make `Justify::Right/Center` ineffective.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipBoundTextTarget(pub(super) Entity);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct UserEquipPopupTrashControl;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum UserEquipUiSet {
    Lifecycle,
    Interaction,
    Bind,
}

#[derive(Resource)]
pub(super) struct UserEquipEditableLayoutHandle(pub(super) Handle<UserEquipEditableLayout>);

impl FromWorld for UserEquipEditableLayoutHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load(USER_EQUIP_EDITABLE_LAYOUT_PATH))
    }
}
