use super::*;

pub const COMBI_REQUEST_PACKET_ID_0104: u32 = 0x1300_0098;

pub const COMBI_SUCCESS_PACKET_ID_0104: u32 = 0x3100_0116;

pub const COMBI_FAILURE_PACKET_ID_0104: u32 = 0x3100_0117;

pub const COMBI_REQUEST_PACKET_SIZE_0104: usize = 16;

pub const COMBI_SUCCESS_PACKET_SIZE_0104: usize = 36;

pub const COMBI_FAILURE_PACKET_SIZE_0104: usize = 20;

pub const COMBI_RESTRICTED_ITEM_FRAME_PATH: &str = "ui/en/combi/restricted-item-frame.png";

#[must_use]
pub fn localized_combi_wire_error(error_code: i32) -> LocalizedText {
    LocalizedText::new(
        "ui.combi.error.wire",
        "Item Combination error. ({error_code})",
    )
    .with_arg("error_code", error_code.to_string())
}

pub(super) const COMBI_HOVER_FRAME_WIDTH: f32 = 2.0;

pub(super) fn bind_selection_frame(
    image: Option<Mut<ImageNode>>,
    cannot_equip: Option<bool>,
    assets: &CombiUiAssets,
) {
    let Some(mut image) = image else {
        return;
    };
    image.image = assets.image(match cannot_equip {
        None => CombiStaticAssetRole::SlotEmpty,
        Some(false) => CombiStaticAssetRole::SlotOccupied,
        Some(true) => CombiStaticAssetRole::Restricted,
    });
}

/// Pointer feedback on cells and drop areas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CombiHoverFrame {
    Hidden,
    /// The pointer is over something a click or drag acts on.
    Hover,
    /// The carried item could land here.
    Available,
    /// Releasing the carried item now lands it here.
    Target,
    /// A click is refused (equipped items, message 260).
    Blocked,
}

pub(super) fn combi_drop_frame(carried: Option<usize>, targeted: bool) -> CombiHoverFrame {
    match (carried, targeted) {
        (None, _) => CombiHoverFrame::Hidden,
        (Some(_), false) => CombiHoverFrame::Available,
        (Some(_), true) => CombiHoverFrame::Target,
    }
}

pub(super) fn bind_hover_frame(
    node: &mut Node,
    background: Option<Mut<BackgroundColor>>,
    border: Option<Mut<BorderColor>>,
    frame: CombiHoverFrame,
) {
    let (color, fill_alpha, line_alpha) = match frame {
        CombiHoverFrame::Hidden => {
            node.display = Display::None;
            return;
        }
        CombiHoverFrame::Hover => (COMBI_COLOR_DEFAULT, 0.12, 0.85),
        CombiHoverFrame::Available => (COMBI_COLOR_DEFAULT, 0.04, 0.35),
        CombiHoverFrame::Target => (COMBI_COLOR_GREEN, 0.14, 0.9),
        CombiHoverFrame::Blocked => (COMBI_COLOR_RED, 0.1, 0.85),
    };
    node.display = Display::Flex;
    if let Some(mut background) = background {
        background.0 = color.bevy().with_alpha(fill_alpha);
    }
    if let Some(mut border) = border {
        *border = BorderColor::all(color.bevy().with_alpha(line_alpha));
    }
}
