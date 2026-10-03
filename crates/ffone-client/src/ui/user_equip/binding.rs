//! Per-frame binding of UserEquip state and projections onto spawned entities.

use super::UserEquipEditableLayout;
use super::nano_projection::UserEquipNanoGalleryEntryProjection;

use super::view_model::{UserEquipItemModeView, UserEquipNanoGallerySlotView};

use super::catalog::{UserEquipEquipmentSlotSpec, UserEquipSlotEndpoint};

use super::components::{
    UserEquipBoundTextTarget, UserEquipEditableLayoutHandle, UserEquipEquipmentSlotLabelText,
    UserEquipItemTabControl, UserEquipNanoTabControl, UserEquipUiElement, UserEquipUiRoot,
    UserEquipUiRuntimeAssets,
};

use super::geometry::UserEquipUiRect;

use super::images::bind_rect;

use super::item_popup::{user_equip_popup_content_y_offset, user_equip_popup_variant_layout_key};

use super::item_projection::UserEquipItemModeProjection;

use super::nano_projection::UserEquipNanoModeProjection;

use super::popup_state::{
    UserEquipDragState, UserEquipItemPopupState, UserEquipNanoViewerState, UserEquipPopupCommand,
    UserEquipPopupLayoutVariant, user_equip_popup_layout_variant,
};

use super::state::{
    UserEquipAvatarPreviewPresentation, UserEquipModalState, UserEquipMode,
    UserEquipPresentationContext, UserEquipUiState,
};

use super::view_model::{
    UserEquipPresentationIcon, equipment_slot_label, equipment_slot_label_color,
    user_equip_item_mode_view, user_equip_nano_gallery_view,
};

use crate::{
    gameplay_ui::GameplayNanoPortraitImages, localization::LocalizedText,
    player_preview::NativePlayerInventoryPreviewImage,
    tutorial_mission_content::TutorialMissionContent,
};

use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

mod editable_layout;
mod item_card_text;
mod item_popup_elements;
mod nano_viewer_elements;
mod slot_elements;
mod status_elements;
mod window_elements;
pub(super) use editable_layout::user_equip_editable_layout_key;
pub(super) use item_card_text::bind_item_card_text_case;
use item_popup_elements::{bind_item_popup_control_element, bind_item_popup_frame_element};
use nano_viewer_elements::{bind_nano_viewer_identity_element, bind_nano_viewer_skill_element};
use slot_elements::bind_slot_element;
use status_elements::bind_status_element;
use window_elements::bind_window_element;

#[derive(SystemParam)]
pub(super) struct UserEquipBindEnvironment<'w, 's> {
    pub(super) content: Option<Res<'w, TutorialMissionContent>>,
    pub(super) windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    pub(super) editable_layouts: Res<'w, Assets<UserEquipEditableLayout>>,
    pub(super) editable_layout_handle: Res<'w, UserEquipEditableLayoutHandle>,
}

#[derive(SystemParam)]
pub(super) struct UserEquipBindQueries<'w, 's> {
    pub(super) roots: Query<'w, 's, (&'static mut Node, &'static mut Visibility), With<UserEquipUiRoot>>,
    pub(super) elements: Query<
        'w,
        's,
        (
            &'static UserEquipUiElement,
            &'static mut Node,
            Option<&'static mut ImageNode>,
            Option<&'static mut LocalizedText>,
            Option<&'static mut TextColor>,
            Option<&'static mut TextLayout>,
            Option<&'static Interaction>,
            Option<&'static UserEquipBoundTextTarget>,
            Option<&'static mut BackgroundColor>,
        ),
        Without<UserEquipUiRoot>,
    >,
    pub(super) bound_texts: Query<'w, 's, &'static mut LocalizedText, Without<UserEquipUiElement>>,
    pub(super) equipment_label_colors: Query<
        'w,
        's,
        (
            &'static UserEquipEquipmentSlotLabelText,
            &'static mut TextColor,
        ),
        Without<UserEquipUiElement>,
    >,
    pub(super) item_tab_interactions: Query<'w, 's, &'static Interaction, With<UserEquipItemTabControl>>,
    pub(super) nano_tab_interactions: Query<'w, 's, &'static Interaction, With<UserEquipNanoTabControl>>,
}

pub(super) fn inventory_static_localized(value: &'static str) -> LocalizedText {
    let key = match value {
        "EQUIPMENT" => "ui.inventory.tab.equipment",
        "NANOS" => "ui.inventory.tab.nanos",
        _ => "ui.content.passthrough",
    };
    if key == "ui.content.passthrough" {
        LocalizedText::new(key, "{text}").with_arg("text", value)
    } else {
        LocalizedText::new(key, value)
    }
}

pub(super) fn inventory_count_localized(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.inventory.item.count", "{count}").with_arg("count", value)
}

pub(super) fn equipment_slot_localized(spec: UserEquipEquipmentSlotSpec) -> LocalizedText {
    let fallback = equipment_slot_label(spec);
    match spec.label_ordinal {
        Some(ordinal) => LocalizedText::new("ui.inventory.slot.weapon", "WEAPON {ordinal}")
            .with_arg("ordinal", ordinal.to_string()),
        None => {
            let key = match spec.label_key {
                "HEAD" => "ui.inventory.slot.head",
                "FACE" => "ui.inventory.slot.face",
                "BACK" => "ui.inventory.slot.back",
                "CHEST" => "ui.inventory.slot.chest",
                "LEGS" => "ui.inventory.slot.legs",
                "FEET" => "ui.inventory.slot.feet",
                "VEHICLE" => "ui.inventory.slot.vehicle",
                _ => "ui.content.passthrough",
            };
            if key == "ui.content.passthrough" {
                LocalizedText::new(key, "{text}").with_arg("text", fallback)
            } else {
                LocalizedText::new(key, fallback)
            }
        }
    }
}

/// Values `bind_user_equip_ui` derives once per run and shares with every element helper.
#[derive(Clone, Copy)]
struct UserEquipBindContext<'a, 'w, 's> {
    asset_server: &'a Res<'w, AssetServer>,
    assets: &'a Res<'w, UserEquipUiRuntimeAssets>,
    state: &'a Res<'w, UserEquipUiState>,
    modal: &'a Res<'w, UserEquipModalState>,
    popup: &'a Res<'w, UserEquipItemPopupState>,
    drag: &'a Res<'w, UserEquipDragState>,
    presentation: &'a Res<'w, UserEquipPresentationContext>,
    avatar_presentation: &'a Res<'w, UserEquipAvatarPreviewPresentation>,
    inventory_preview: &'a Option<Res<'w, NativePlayerInventoryPreviewImage>>,
    nano_portraits: &'a Option<Res<'w, GameplayNanoPortraitImages>>,
    projection: &'a Option<Res<'w, UserEquipItemModeProjection>>,
    nano_projection: &'a UserEquipNanoModeProjection,
    environment: &'a UserEquipBindEnvironment<'w, 's>,
    window: &'a Window,
    view: &'a UserEquipItemModeView,
    nano_view: &'a Vec<UserEquipNanoGallerySlotView>,
    nano_mode: bool,
    popup_commands: &'a Vec<UserEquipPopupCommand>,
    popup_layout_variant: Option<UserEquipPopupLayoutVariant>,
    selected_nano: Option<&'a UserEquipNanoGalleryEntryProjection>,
    endpoint_icon: &'a dyn Fn(UserEquipSlotEndpoint) -> Option<&'a UserEquipPresentationIcon>,
    item_tab_hovered: bool,
    nano_tab_hovered: bool,
}

pub(super) fn bind_user_equip_ui(
    asset_server: Res<AssetServer>,
    assets: Res<UserEquipUiRuntimeAssets>,
    state: Res<UserEquipUiState>,
    modal: Res<UserEquipModalState>,
    popup: Res<UserEquipItemPopupState>,
    drag: Res<UserEquipDragState>,
    nano_viewer: Res<UserEquipNanoViewerState>,
    presentation: Res<UserEquipPresentationContext>,
    avatar_presentation: Res<UserEquipAvatarPreviewPresentation>,
    inventory_preview: Option<Res<NativePlayerInventoryPreviewImage>>,
    nano_portraits: Option<Res<GameplayNanoPortraitImages>>,
    projection: Option<Res<UserEquipItemModeProjection>>,
    nano_projection: Option<Res<UserEquipNanoModeProjection>>,
    environment: UserEquipBindEnvironment,
    mut queries: UserEquipBindQueries,
) {
    let Ok(window) = environment.windows.single() else {
        for (_, mut visibility) in &mut queries.roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let width = window.width().max(0.0) as u32;
    let height = window.height().max(0.0) as u32;
    let view = projection.as_deref().and_then(|projection| {
        user_equip_item_mode_view(
            width,
            height,
            *state,
            *modal,
            projection,
            assets.0.all_loaded(&asset_server),
        )
    });
    for (mut node, mut visibility) in &mut queries.roots {
        node.width = px(width);
        node.height = px(height);
        *visibility = if view.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let Some(view) = view else {
        return;
    };
    for (label, mut text_color) in &mut queries.equipment_label_colors {
        text_color.0 = equipment_slot_label_color(view.equipment[label.0].frame_visual);
    }
    let nano_projection = nano_projection
        .as_deref()
        .expect("UserEquip plugin initializes its read-only Nano projection");
    let nano_view = user_equip_nano_gallery_view(view.layout, nano_projection);
    let nano_mode = state.mode() == UserEquipMode::Nano;
    let popup_commands = popup.commands(
        projection
            .as_deref()
            .expect("visible UserEquip view requires its authoritative projection"),
        environment.content.as_deref(),
    );
    let popup_layout_variant = user_equip_popup_layout_variant(
        &popup,
        projection
            .as_deref()
            .expect("visible UserEquip view requires its authoritative projection"),
        environment.content.as_deref(),
    );
    let selected_nano = nano_viewer
        .selected_visual_index
        .and_then(|index| nano_projection.gallery.get(index));
    let endpoint_icon = |endpoint: UserEquipSlotEndpoint| match endpoint {
        UserEquipSlotEndpoint::Inventory { slot_index } => {
            view.inventory.get(slot_index).map(|slot| &slot.icon)
        }
        UserEquipSlotEndpoint::Equipment { visual_index, .. } => {
            view.equipment.get(visual_index).map(|slot| &slot.icon)
        }
    };
    let item_tab_hovered = queries
        .item_tab_interactions
        .iter()
        .any(|interaction| matches!(interaction, Interaction::Hovered | Interaction::Pressed));
    let nano_tab_hovered = queries
        .nano_tab_interactions
        .iter()
        .any(|interaction| matches!(interaction, Interaction::Hovered | Interaction::Pressed));
    let editable_layout = environment
        .editable_layouts
        .get(&environment.editable_layout_handle.0)
        .map(|layout| &layout.0);

    let context = UserEquipBindContext {
        asset_server: &asset_server,
        assets: &assets,
        state: &state,
        modal: &modal,
        popup: &popup,
        drag: &drag,
        presentation: &presentation,
        avatar_presentation: &avatar_presentation,
        inventory_preview: &inventory_preview,
        nano_portraits: &nano_portraits,
        projection: &projection,
        nano_projection,
        environment: &environment,
        window,
        view: &view,
        nano_view: &nano_view,
        nano_mode,
        popup_commands: &popup_commands,
        popup_layout_variant,
        selected_nano,
        endpoint_icon: &endpoint_icon,
        item_tab_hovered,
        nano_tab_hovered,
    };

    for (element, mut node, image, localized, text_color, text_layout, interaction, bound_text, background) in
        &mut queries.elements
    {
        let localized = localized
            .or_else(|| bound_text.and_then(|target| queries.bound_texts.get_mut(target.0).ok()));
        match *element {
            UserEquipUiElement::Backdrop
            | UserEquipUiElement::ClothesBackplate
            | UserEquipUiElement::RightBackplate
            | UserEquipUiElement::UserClothesPanel
            | UserEquipUiElement::AvatarPreview
            | UserEquipUiElement::TurnLeftPositioned
            | UserEquipUiElement::TurnRightPositioned
            | UserEquipUiElement::PcStuffPanel
            | UserEquipUiElement::PcStuffPanelBackground
            | UserEquipUiElement::EquipmentPanel
            | UserEquipUiElement::InventoryContent
            | UserEquipUiElement::InventoryViewport
            | UserEquipUiElement::NanoViewport
            | UserEquipUiElement::NanoContent
            | UserEquipUiElement::ScrollTrack
            | UserEquipUiElement::ScrollUp
            | UserEquipUiElement::ScrollDown
            | UserEquipUiElement::ScrollThumb
            | UserEquipUiElement::InventoryShadowA
            | UserEquipUiElement::InventoryShadowB
            | UserEquipUiElement::Close
            | UserEquipUiElement::ItemPopupClose
            | UserEquipUiElement::Trash
            | UserEquipUiElement::Help
            | UserEquipUiElement::HelpPanel
            | UserEquipUiElement::DraggedItem => {
                bind_window_element(&context, element, node.reborrow(), image, localized, text_color, text_layout, interaction)
            }
            UserEquipUiElement::NanoSlotFrame (_)
            | UserEquipUiElement::NanoSlotIcon (_)
            | UserEquipUiElement::InventorySlotFrame (_)
            | UserEquipUiElement::InventorySlotIcon (_)
            | UserEquipUiElement::InventorySlotBadge (_)
            | UserEquipUiElement::InventorySlotCount (_)
            | UserEquipUiElement::EquipmentSlotFrame (_)
            | UserEquipUiElement::EquipmentSlotIcon (_)
            | UserEquipUiElement::EquipmentSlotBadge (_) => {
                bind_slot_element(&context, element, node.reborrow(), image, localized, text_color, text_layout, interaction, background)
            }
            UserEquipUiElement::BoostValue
            | UserEquipUiElement::PotionValue
            | UserEquipUiElement::TarosDigit (_)
            | UserEquipUiElement::StatusName
            | UserEquipUiElement::StatusLevel
            | UserEquipUiElement::StatusHpValue
            | UserEquipUiElement::StatusFusionMatterValue
            | UserEquipUiElement::StatusGuideName
            | UserEquipUiElement::StatusGuideIcon
            | UserEquipUiElement::StatusHpBar
            | UserEquipUiElement::StatusFusionBar
            | UserEquipUiElement::ItemTab
            | UserEquipUiElement::NanoTab
            | UserEquipUiElement::NanoStatusPanel (_)
            | UserEquipUiElement::NanoStatusPortrait (_)
            | UserEquipUiElement::NanoStatusType (_)
            | UserEquipUiElement::NanoStatusSlotLabel (_)
            | UserEquipUiElement::NanoStatusName (_)
            | UserEquipUiElement::NanoStatusAttribute (_)
            | UserEquipUiElement::NanoStatusSkill (_)
            | UserEquipUiElement::NanoStatusStamina (_) => {
                bind_status_element(&context, element, node.reborrow(), image, localized, text_color, text_layout, interaction)
            }
            UserEquipUiElement::ItemPopup
            | UserEquipUiElement::ItemPopupBackdrop
            | UserEquipUiElement::ItemPopupTitle
            | UserEquipUiElement::ItemPopupIconFrame
            | UserEquipUiElement::ItemPopupIcon
            | UserEquipUiElement::ItemPopupIdentity
            | UserEquipUiElement::ItemPopupEquipInfo => {
                bind_item_popup_frame_element(&context, element, node.reborrow(), image, localized, text_color, text_layout, interaction)
            }
            UserEquipUiElement::ItemPopupField (_)
            | UserEquipUiElement::ItemPopupGumNanoFrame (_)
            | UserEquipUiElement::ItemPopupGumNanoIcon (_)
            | UserEquipUiElement::ItemPopupGumNanoButton (_)
            | UserEquipUiElement::ItemPopupGumNanoButtonLabel (_)
            | UserEquipUiElement::ItemPopupButton (_)
            | UserEquipUiElement::ItemPopupTrash
            | UserEquipUiElement::ItemPopupButtonLabel (_)
            | UserEquipUiElement::ItemPopupCalculatorBack
            | UserEquipUiElement::ItemPopupAmountLabel
            | UserEquipUiElement::ItemPopupAmountValue
            | UserEquipUiElement::ItemPopupKeypadLabel (_) => {
                bind_item_popup_control_element(&context, element, node.reborrow(), image, localized, text_color, text_layout, interaction)
            }
            UserEquipUiElement::NanoViewer
            | UserEquipUiElement::NanoViewerInnerBackdrop
            | UserEquipUiElement::NanoViewerEquippedTitle
            | UserEquipUiElement::NanoViewerIcon
            | UserEquipUiElement::NanoViewerName
            | UserEquipUiElement::NanoViewerAttribute
            | UserEquipUiElement::NanoViewerDescription
            | UserEquipUiElement::NanoViewerCurrentPower => {
                bind_nano_viewer_identity_element(&context, element, node.reborrow(), image, localized, text_color, text_layout, interaction)
            }
            UserEquipUiElement::NanoViewerSkillIcon (_)
            | UserEquipUiElement::NanoViewerSkillName (_)
            | UserEquipUiElement::NanoViewerSkillType (_)
            | UserEquipUiElement::NanoViewerSkillDescription (_)
            | UserEquipUiElement::NanoViewerRequirementPrompt (_)
            | UserEquipUiElement::NanoViewerRequirementLabel (_, _)
            | UserEquipUiElement::NanoViewerRequirementBar (_, _)
            | UserEquipUiElement::NanoViewerRequirementValue (_, _)
            | UserEquipUiElement::NanoViewerStationNotice
            | UserEquipUiElement::NanoViewerClose => {
                bind_nano_viewer_skill_element(&context, element, node.reborrow(), image, localized, text_color, text_layout, interaction)
            }
            UserEquipUiElement::ItemTabLabel
            | UserEquipUiElement::NanoTabLabel
            | UserEquipUiElement::EquipmentTitle
            | UserEquipUiElement::EquipmentTitleLabel
            | UserEquipUiElement::EquipmentSlotLabel(_)
            | UserEquipUiElement::BoostSlot
            | UserEquipUiElement::BoostIcon
            | UserEquipUiElement::BoostLabel
            | UserEquipUiElement::PotionSlot
            | UserEquipUiElement::PotionIcon
            | UserEquipUiElement::PotionLabel
            | UserEquipUiElement::TarosBack
            | UserEquipUiElement::Dexlabs
            | UserEquipUiElement::StatusPanel
            | UserEquipUiElement::StatusHpBack
            | UserEquipUiElement::StatusFusionBack
            | UserEquipUiElement::StatusGuideBox
            | UserEquipUiElement::StatusGuideLabel
            | UserEquipUiElement::StatusHp
            | UserEquipUiElement::StatusFusionMatter
            | UserEquipUiElement::HelpTitle
            | UserEquipUiElement::HelpBody
            | UserEquipUiElement::HelpClose => {}
        }
        let editable_key = user_equip_editable_layout_key(*element);
        let variant_key = popup_layout_variant
            .and_then(|variant| user_equip_popup_variant_layout_key(*element, variant));
        let editable_rect = editable_layout.and_then(|layout| match variant_key.as_deref() {
            // Popup variants have different source owners and dimensions.
            // Never let an absent variant override silently fall back to the
            // equipment card's geometry.
            Some(key) => layout.rect(key),
            None => editable_key.as_deref().and_then(|key| layout.rect(key)),
        });
        if let Some(rect) = editable_rect {
            // The editor owns geometry only. Runtime subtype/item semantics
            // still own visibility (for example the clean weapon popup has
            // no STATUS label, and vehicle cards hide the rating row).
            let display = node.display;
            let content_y = if matches!(
                *element,
                UserEquipUiElement::ItemPopupEquipInfo
                    | UserEquipUiElement::ItemPopupTitle
                    | UserEquipUiElement::ItemPopupClose
                    | UserEquipUiElement::ItemPopupTrash
                    | UserEquipUiElement::ItemPopupButton(_)
                    | UserEquipUiElement::ItemPopupIcon
                    | UserEquipUiElement::ItemPopupIdentity
                    | UserEquipUiElement::ItemPopupField(_)
            ) {
                user_equip_popup_content_y_offset(popup_layout_variant)
            } else {
                0.0
            };
            bind_rect(
                &mut node,
                UserEquipUiRect::new(rect.x, rect.y + content_y, rect.width, rect.height),
            );
            node.display = display;
        }
    }
}

pub(super) fn bounded_status_fraction(current: i32, maximum: i32) -> f32 {
    if maximum <= 0 {
        0.0
    } else {
        ((current.max(0) as f32) / maximum as f32).clamp(0.0, 1.0)
    }
}
