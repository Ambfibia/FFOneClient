use super::*;

pub const OPTION_UI_Z_INDEX: i32 = 8_200;

pub const OPTION_NORMAL_TAB_Z_INDEX: i32 = 0;

pub const OPTION_PAGE_Z_INDEX: i32 = 20;

pub const OPTION_SELECTED_TAB_Z_INDEX: i32 = 30;

pub const OPTION_CHROME_Z_INDEX: i32 = 40;

/// Clean `OnGraphics` paints an open `downbox` after every other control, so
/// its list covers the neighbouring pulldowns...
pub const OPTION_DROPDOWN_PANEL_Z_INDEX: i32 = 60;

pub const OPTION_JEFFE_FONT_PATH: &str = "fonts/jeffe.otf";

pub const OPTION_COMIC_FONT_PATH: &str = "fonts/comic-sans-ms-bold.ttf";

pub const OPTION_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct OptionUiAssetGate {
    pub ready: bool,
    pub failed: bool,
}

pub(super) fn update_option_asset_gate(
    asset_server: Res<AssetServer>,
    assets: Res<OptionUiAssets>,
    mut gate: ResMut<OptionUiAssetGate>,
) {
    // The gate describes an immutable authored asset closure. Once it has
    // settled, polling every image/font handle and dirtying the resource each
    // frame cannot produce a different valid result.
    if gate.failed || gate.ready {
        return;
    }
    let mut next = OptionUiAssetGate {
        ready: true,
        failed: false,
    };
    for state in assets
        .image_load_states(&asset_server)
        .chain(assets.font_load_states(&asset_server))
    {
        match state {
            LoadState::Failed(_) => {
                next.ready = false;
                next.failed = true;
                break;
            }
            LoadState::Loaded => {}
            _ => next.ready = false,
        }
    }
    if *gate != next {
        *gate = next;
    }
}
