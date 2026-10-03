use super::*;

pub(super) fn advance_upsell_page_fade(time: Res<Time>, mut model: ResMut<UpsellUiModel>) {
    model.advance_page_fade(time.delta_secs());
}
