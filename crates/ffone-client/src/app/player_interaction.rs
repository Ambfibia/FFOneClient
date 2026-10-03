//! Actions for a selected live player. Selection itself never sends a request.
use super::group_pc2pc::WorldPc2pcOfferPrompt;
use super::runtime_status::RuntimeStatus;
use bevy::prelude::*;
use ffone_client::{
    avatar_action::LegacyAvatarTargetFeed,
    entity_lifecycle::{NetworkPcAppearance0104, NetworkRemotePc0104},
    group_runtime::{GroupInviteOwner0104, GroupOwnerReachability0104, GroupProductionRuntime0104, GroupRemotePlayer0104},
    localization::LocalizedText,
    network::{NetworkBridge, NetworkCommand},
    pc2pc_ui::Pc2pcOfferRuntime0104,
};

#[derive(Component)]
pub(super) struct PlayerMenuRoot;
#[derive(Component)]
pub(super) struct PlayerMenuName;
#[derive(Component, Clone, Copy)]
pub(super) enum Action { Trade, Buddy, Group, Close }

pub(super) fn spawn(mut commands: Commands, assets: Res<AssetServer>) {
    let font = assets.load(ffone_client::pc2pc_ui::PC2PC_JEFFE_FONT_PATH);
    commands.spawn((
        PlayerMenuRoot,
        Node { position_type: PositionType::Absolute, left: percent(50), top: percent(35),
            width: px(250), margin: UiRect::left(px(-125)), padding: UiRect::all(px(12)),
            flex_direction: FlexDirection::Column, row_gap: px(6), display: Display::None, ..default() },
        GlobalZIndex(1_500), BackgroundColor(Color::srgb(0.02, 0.12, 0.20)),
        ffone_client::ui::shared::controller::ControllerUiBoundary,
    )).with_children(|panel| {
        panel.spawn((PlayerMenuName, Text::new(""), TextFont { font: font.clone().into(), font_size: 16.0.into(), ..default() },
            TextColor(Color::WHITE), LocalizedText::new("ui.player_menu.name", "{name}").with_arg("name", ""), Pickable::IGNORE));
        for (action, key, label) in [
            (Action::Trade, "trade", "TRADE"), (Action::Buddy, "buddy", "ADD BUDDY"),
            (Action::Group, "group", "INVITE TO GROUP"), (Action::Close, "close", "CLOSE"),
        ] {
            panel.spawn((Button, action, Node { width: percent(100), height: px(34), align_items: AlignItems::Center,
                justify_content: JustifyContent::Center, ..default() }, BackgroundColor(Color::srgb(0.06, 0.32, 0.48))))
                .with_child((Text::new(label), TextFont { font: font.clone().into(), font_size: 14.0.into(), ..default() },
                    LocalizedText::new(format!("ui.player_menu.{key}"), label), TextColor(Color::WHITE), Pickable::IGNORE));
        }
    });
}

pub(super) fn bind(
    menu: Res<WorldPc2pcOfferPrompt>,
    mut root: Query<&mut Node, With<PlayerMenuRoot>>,
    mut name: Query<&mut LocalizedText, With<PlayerMenuName>>,
) {
    for mut root in &mut root { root.display = if menu.selected.is_some() { Display::Flex } else { Display::None }; }
    for mut name in &mut name {
        name.set_if_neq(LocalizedText::new("ui.player_menu.name", "{name}").with_arg("name", &menu.selected_name));
    }
}

pub(super) fn interact(
    mut menu: ResMut<WorldPc2pcOfferPrompt>,
    mut offers: ResMut<Pc2pcOfferRuntime0104>,
    mut group: ResMut<GroupProductionRuntime0104>,
    mut status: ResMut<RuntimeStatus>,
    bridge: Res<NetworkBridge>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    feeds: Query<&LegacyAvatarTargetFeed>,
    peers: Query<(&NetworkRemotePc0104, &NetworkPcAppearance0104)>,
    buttons: Query<(&Action, &Interaction), Changed<Interaction>>,
) {
    let Some((_, local, remote, actor, entity)) = menu.selected else { return; };
    let peer = peers.get(entity).ok().filter(|(pc, _)| pc.pc_id == remote);
    let valid = status.player_id == Some(local) && status.allow_player_interaction && peer.is_some()
        && feeds.get(actor).is_ok_and(|feed| feed.source_connected && feed.samples.iter().any(|sample|
            sample.entity == entity && sample.talk_enabled && ffone_client::world_targeting::world_pc_in_interaction_range(sample.distance)));
    if !valid || keys.just_pressed(KeyCode::Escape) {
        menu.selected = None;
        keys.clear_just_pressed(KeyCode::Escape);
        return;
    }
    let Some(action) = buttons.iter().find_map(|(action, interaction)| (*interaction == Interaction::Pressed).then_some(*action)) else { return; };
    let (_, appearance) = peer.unwrap();
    let result: Result<(), String> = match action {
        Action::Close => Ok(()),
        Action::Trade => offers.request_outgoing(local, remote).map(|_| ()).map_err(|e| e.to_string()),
        Action::Buddy => bridge.send(NetworkCommand::RequestBuddy(ffone_protocol::BuddyMakeRequest0104 {
            buddy_id: remote, buddy_pc_uid: appearance.0.style.pc_uid,
        })).map_err(|e| e.to_string()),
        Action::Group => group.request_invite(
            GroupRemotePlayer0104::new(remote, appearance.0.style.pc_uid, &menu.selected_name),
            GroupInviteOwner0104::PlayerMenu, GroupOwnerReachability0104::Proven,
        ).map_err(|e| format!("{e:?}")).and_then(|request| bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)).map_err(|e| e.to_string())),
    };
    menu.selected = None;
    if let Err(error) = result { status.message = format!("Player interaction failed: {error}"); }
}
