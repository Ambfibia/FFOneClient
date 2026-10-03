use super::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldMapPresentationMap;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldMapPresentationMarker(pub usize);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldMapPresentationControlNode(pub WorldMapPresentationControl);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationBackdrop;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationNormalLayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationNoMap;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationBlack;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationLine;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationLineEffect(pub(super) u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationMarkerLayer;

#[derive(Component)]
pub(super) struct WorldMapDecoration;

#[derive(Component)]
pub(super) struct WorldMapSpin;

#[derive(Component)]
pub(super) struct WorldMapPulse(pub(super) bool);

#[derive(Component)]
pub(super) struct WorldMapFilterVisual {
    pub(super) index: usize,
    pub(super) part: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationControlLabel(pub(super) WorldMapPresentationControl);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationZoomBar;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationCurrentLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationCurrentValue;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationTooltip;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationTooltipText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum WorldMapPresentationSet {
    Preload,
    Bind,
}

pub struct WorldMapPresentationPlugin;

impl Plugin for WorldMapPresentationPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<WorldMapPresentation>()
            .init_resource::<WorldMapPresentationAssetStatus>()
            .init_resource::<WorldMapScanAnimation>()
            .init_resource::<crate::ui_icon_variants::UiIconVariants>()
            .configure_sets(
                Update,
                (
                    WorldMapPresentationSet::Preload,
                    WorldMapPresentationSet::Bind,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_world_map_presentation,
            )
            .add_systems(
                Update,
                (update_world_map_asset_status.in_set(WorldMapPresentationSet::Preload))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    advance_world_map_scan_animation,
                    sync_world_map_presentation,
                    sync_world_map_controls,
                    sync_world_map_text,
                    sync_world_map_markers,
                    sync_world_map_filter_visuals,
                    animate_world_map_decorations,
                )
                    .chain()
                    .in_set(WorldMapPresentationSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
