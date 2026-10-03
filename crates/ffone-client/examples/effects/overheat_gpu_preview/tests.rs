use ffone_client::overheat_ui::{
    LegacyClassWeaponOverheatTable, OverheatUiConfig, overheat_ui_view,
};

use super::*;

#[test]
fn measured_client_area_uses_integer_half_source_geometry() {
    let view = overheat_ui_view(
        CLIENT_AREA_WIDTH,
        CLIENT_AREA_HEIGHT,
        OverheatUiConfig::default(),
        OverheatParityPreview::enabled(),
        OverheatUiModel {
            ready_for_play: true,
            main: LegacyOverheatWeaponSlot::equipped(100, 4, 65.0),
            ..default()
        },
        &LegacyClassWeaponOverheatTable::default(),
    );
    assert_eq!(
        view.background.unwrap(),
        ffone_client::overheat_ui::OverheatUiRect::new(572.0, 340.0, 17.0, 86.0)
    );
    assert_eq!(
        view.normal_fill.unwrap(),
        ffone_client::overheat_ui::OverheatUiRect::new(579.0, 372.0, 6.0, 52.0)
    );
}
