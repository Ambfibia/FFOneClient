use super::*;

pub(in super::super) struct NetworkIngressPlugin;

impl Plugin for NetworkIngressPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkBridge::start())
            .init_resource::<RuntimeStatus>()
            .add_systems(Update, shiny_pickup::pickup
                .after(sync_tutorial_action_gate)
                .after(consume_network_entity_lifecycle_0104)
                .before(ffone_client::tutorial_effects_runtime::process_tutorial_effect_runtime))
            .init_resource::<LocalInfectionPresentationState>()
            .init_resource::<ActiveLoginCredentials>()
            .init_resource::<NetworkLifecycleSession>()
            .add_systems(
                Update,
                poll_network
                    .before(ffone_client::gameplay_ui::GameplayUiSet::Rewards)
                    .before(NetworkSessionLifecycleSet::Apply)
                    .before(consume_network_entity_lifecycle_0104)
                    .before(sync_quit_menu_context),
            )
            .add_systems(
                Update,
                sync_reward_inventory
                    .after(poll_network)
                    .after(NetworkSessionLifecycleSet::Apply)
                    .before(ffone_client::gameplay_ui::GameplayUiSet::Rewards),
            );
    }
}

#[derive(SystemParam)]
pub(in super::super) struct NetworkRosterIngress<'w> {
    pub(super) runtime: ResMut<'w, RuntimeStatus>,
    pub(super) selection_ui: ResMut<'w, CharacterSelectionUiModel>,
}

#[derive(SystemParam)]
pub(super) struct WorldIngressQueries<'w, 's> {
    pub(super) mission_npcs: Query<'w, 's, &'static NetworkNpcAppearance0104>,
    pub(super) remote_appearances: Query<'w, 's, &'static NetworkPcAppearance0104>,
    pub(super) remote_players: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkRemotePc0104,
            &'static GlobalTransform,
        ),
    >,
    pub(super) remote_animations: Query<'w, 's, (&'static NetworkRemotePc0104, &'static mut RemoteAnimation)>,
    pub(super) local_identities: Query<'w, 's, (Entity, &'static LocalNetworkIdentity)>,
    pub(super) parents: Query<'w, 's, &'static ChildOf>,
    pub(super) named_transforms: Query<'w, 's, (Entity, &'static Name, &'static GlobalTransform)>,
    pub(super) movement_bases: Query<'w, 's, &'static mut movement_buffs::MovementBuffBase>,
    pub(super) local_player: Query<
        'w,
        's,
        (
            Entity,
            &'static mut Transform,
            &'static mut LegacyPlayerController,
            &'static mut Visibility,
            &'static mut LegacyAvatarEnvironmentState,
        ),
        With<LocalPlayer>,
    >,
}
