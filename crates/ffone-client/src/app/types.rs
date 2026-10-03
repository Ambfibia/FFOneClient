use super::*;

/// Installs [`release_unextracted_view_targets`] alongside Bevy's own
/// pre-`create_surfaces` view-target cleanup.
pub struct ReleaseUnextractedViewTargetsPlugin;

impl Plugin for ReleaseUnextractedViewTargetsPlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_systems(
            Render,
            release_unextracted_view_targets
                .in_set(RenderSystems::PrepareViews)
                .before(create_surfaces),
        );
    }
}

#[derive(Component)]
pub(super) struct LocalPlayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LocalNetworkIdentity {
    pub(super) pc_uid: i64,
    pub(super) player_id: i32,
}

#[derive(Component)]
pub(super) struct LocalCharacterScene;

#[derive(Component)]
pub(super) struct WorldSliceEntity;

/// Records network NPC roots hidden while the clean tutorial script owns the
/// local NPC presentation. The marker makes restoration selective when the
/// client returns to the ordinary shared world.
#[derive(Component)]
pub(super) struct TutorialSuppressedNetworkNpc;
