use super::*;

pub const NANO_FREE_TUNING_GAME_OBJECT_PATH_ID: i64 = 1_353;

pub const NANO_FREE_TUNING_TRANSFORM_PATH_ID: i64 = 1_266;

pub const NANO_FREE_TUNING_MODE_COMPONENT_PATH_ID: i64 = 1_486;

pub const NANO_FREE_TUNING_UI_COMPONENT_PATH_ID: i64 = 1_487;

pub const NANO_FREE_TUNING_MODE_SCRIPT_PATH_ID: i64 = 992;

pub const NANO_FREE_TUNING_UI_SCRIPT_PATH_ID: i64 = 1_102;

pub const NANO_FREE_TUNING_SKIN_PATH_ID: i64 = 1_379;

pub const NANO_FREE_TUNING_UI_Z_INDEX: i32 = 9_000;

pub const NANO_FREE_TUNING_PANEL_PATH: &str = "ui/en/nano-free-tuning/three_power_panel.png";

pub const NANO_FREE_TUNING_BLACK_PATH: &str = "ui/en/nano-free-tuning/black_bar.png";

pub const NANO_FREE_TUNING_SELECT_NORMAL_PATH: &str = "ui/en/nano-free-tuning/select_normal.png";

pub const NANO_FREE_TUNING_FONT_PATH: &str = "fonts/jeffe.otf";

/// Replacement-font calibration for clean fixed-raster `JEFFE___16`
/// (Font path ID 1012, `m_LineSpacing = 16.45199966430664`).
pub const NANO_FREE_TUNING_JEFFE_16_FONT_PATH_ID: i64 = 1_012;

/// Replacement-font calibration for clean fixed-raster `JEFFE___14`
/// (Font path ID 903, `m_LineSpacing = 13.710000038146973`).
pub const NANO_FREE_TUNING_JEFFE_14_FONT_PATH_ID: i64 = 903;

/// Replacement-font calibration for clean fixed-raster `JEFFE___08`
/// (Font path ID 948, `m_LineSpacing = 10.968000411987305`).
pub const NANO_FREE_TUNING_JEFFE_08_FONT_PATH_ID: i64 = 948;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningAssetContract {
    pub semantic_name: &'static str,
    pub path: &'static str,
    pub source_path_id: i64,
    pub source_name: &'static str,
    pub source_texture_format: &'static str,
    pub source_width: u32,
    pub source_height: u32,
    pub source_payload_sha256: &'static str,
    pub png_sha256: &'static str,
}

pub const NANO_FREE_TUNING_ASSET_CONTRACTS: [NanoFreeTuningAssetContract; 4] = [
    NanoFreeTuningAssetContract {
        semantic_name: "three_power_panel",
        path: NANO_FREE_TUNING_PANEL_PATH,
        source_path_id: 283,
        source_name: "NanoPopUpNextNano 1",
        source_texture_format: "DXT5",
        source_width: 357,
        source_height: 460,
        source_payload_sha256: "375F9B66E509E54F11F537453FE0CC4CD418FA4E95D065EF328007D7357FE65A",
        png_sha256: "FD0F4C64DBCA190B398DFA89CA955C4099A8A33612961925C840F1E8619AEA31",
    },
    NanoFreeTuningAssetContract {
        semantic_name: "black_bar",
        path: NANO_FREE_TUNING_BLACK_PATH,
        source_path_id: 270,
        source_name: "back",
        source_texture_format: "DXT1",
        source_width: 8,
        source_height: 8,
        source_payload_sha256: "66687AADF862BD776C8FC18B8E9F8E20089714856EE233B3902A591D0D5F2925",
        png_sha256: "02F754E0E1267317940B0703EA7BAB4C421CC13467D385D72136B80109099A81",
    },
    NanoFreeTuningAssetContract {
        semantic_name: "select_normal",
        path: NANO_FREE_TUNING_SELECT_NORMAL_PATH,
        source_path_id: 640,
        source_name: "blue_button_normal",
        source_texture_format: "RGBA32",
        source_width: 20,
        source_height: 25,
        source_payload_sha256: "01F0DC77EDE174C14572C7EFCC65DA28D0A2CBF3FB023E9EB510B3E5CF8BDC2E",
        png_sha256: "506EA52CF108EEB14A035FA52A4C650BA1A5255B6E4FB62A10ADC3AEDFC72C8E",
    },
    NanoFreeTuningAssetContract {
        semantic_name: "select_hover",
        path: NANO_FREE_TUNING_SELECT_HOVER_PATH,
        source_path_id: 309,
        source_name: "blue_button_over",
        source_texture_format: "RGBA32",
        source_width: 20,
        source_height: 25,
        source_payload_sha256: "98E68EDDE8DF2B55D3B46331441EDBB13DC4CDF384313F8A06432133C97E69E3",
        png_sha256: "1EFF6610DB23CFFDB5E454708141B56A10A4E60CDFB7D64357EFE0C6C30B1798",
    },
];

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub enum NanoFreeTuningPresentationAssetStatus {
    #[default]
    Loading,
    Ready,
    Failed {
        asset_path: &'static str,
    },
}

pub(super) fn update_nano_free_tuning_asset_status(
    asset_server: Res<AssetServer>,
    assets: Res<NanoFreeTuningPresentationAssets>,
    mut status: ResMut<NanoFreeTuningPresentationAssetStatus>,
) {
    let failed = assets
        .source_images()
        .into_iter()
        .find_map(|(path, handle)| {
            matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)).then_some(path)
        })
        .or_else(|| {
            matches!(
                asset_server.load_state(assets.font.id()),
                LoadState::Failed(_)
            )
            .then_some(NANO_FREE_TUNING_FONT_PATH)
        });
    *status = if let Some(asset_path) = failed {
        NanoFreeTuningPresentationAssetStatus::Failed { asset_path }
    } else if assets
        .source_images()
        .into_iter()
        .all(|(_, handle)| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(asset_server.load_state(assets.font.id()), LoadState::Loaded)
    {
        NanoFreeTuningPresentationAssetStatus::Ready
    } else {
        NanoFreeTuningPresentationAssetStatus::Loading
    };
}
