use super::*;

#[test]
fn bevy_tree_preserves_clean_immediate_mode_draw_order_and_group_clipping() {
    fn collect_roles(world: &World, entity: Entity, roles: &mut Vec<NanocomDrawRole>) {
        if let Some(role) = world.get::<NanocomDrawRole>(entity) {
            roles.push(*role);
        }
        if let Some(children) = world.get::<Children>(entity) {
            let children = children.iter().collect::<Vec<_>>();
            for child in children {
                collect_roles(world, child, roles);
            }
        }
    }

    let asset_root = tempdir().unwrap().keep();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(NanocomMessageUiPlugin);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.update();

    let world = app.world_mut();
    let compact_root = world
        .query_filtered::<Entity, With<NanocomMessageCompactPanel>>()
        .single(world)
        .unwrap();
    let modal_root = world
        .query_filtered::<Entity, With<NanocomMessageExpandedRoot>>()
        .single(world)
        .unwrap();
    let mut compact = Vec::new();
    collect_roles(world, compact_root, &mut compact);
    assert_eq!(
        compact,
        [
            NanocomDrawRole::CompactFrame,
            NanocomDrawRole::CompactTitle,
            NanocomDrawRole::CompactBody,
            NanocomDrawRole::CompactIcon,
        ]
    );
    let mut modal = Vec::new();
    collect_roles(world, modal_root, &mut modal);
    assert_eq!(
        modal,
        [
            NanocomDrawRole::ModalOverlay,
            NanocomDrawRole::ModalDialogBox,
            NanocomDrawRole::ModalMessageArea,
            NanocomDrawRole::ModalIcon,
            NanocomDrawRole::ModalTitle,
            NanocomDrawRole::ModalTextArea,
            NanocomDrawRole::ModalAccept,
            NanocomDrawRole::ModalDecline,
        ]
    );
    assert!(
        modal
            .windows(2)
            .all(|pair| pair[0].ordinal() < pair[1].ordinal())
    );

    let dialog = world
        .query_filtered::<&Node, With<NanocomMessageExpandedDialog>>()
        .single(world)
        .unwrap();
    assert_eq!(dialog.overflow, Overflow::clip());
}
