//! Optional live equipment/location regression for newly created characters.
use super::*;
use ffone_client::{
    character_selection_portraits::{
        CharacterSelectionPortraitStatus, CharacterSelectionPortraitsModel,
    },
    user_equip_ui::{USER_EQUIP_EQUIPMENT_STRIP_ORDER, UserEquipSlotEndpoint},
};
use std::collections::BTreeMap;

#[derive(Default, Resource)]
pub(super) struct EquipmentCheck {
    requested_uid: Option<i64>,
    frames: usize,
    expected: BTreeMap<i64, CharacterSummary>,
}

pub(super) fn install(app: &mut App) {
    if env::var_os("FFONE_CHARACTER_EQUIPMENT_CHECK").is_some() {
        app.init_resource::<EquipmentCheck>();
    }
}

pub(super) fn prepare_return(world: &mut World) -> bool {
    if !world.contains_resource::<EquipmentCheck>() {
        return true;
    }
    let uid = world
        .resource::<RuntimeStatus>()
        .roster
        .selected_uid
        .unwrap();
    if world.resource::<EquipmentCheck>().requested_uid != Some(uid) {
        let inventory = world
            .resource::<LocalInventoryRuntime>()
            .snapshot()
            .unwrap();
        assert!(
            inventory.equipment()[1].item_id > 0,
            "new character has a starter shirt"
        );
        let slot_index = inventory
            .inventory()
            .iter()
            .position(|item| item.item_id == 0)
            .unwrap();
        let visual_index = USER_EQUIP_EQUIPMENT_STRIP_ORDER
            .iter()
            .position(|slot| slot.wire_slot_index == 1)
            .unwrap();
        world.resource_mut::<UserEquipUiState>().open_item_mode();
        world
            .resource_mut::<UserEquipUiOutbox>()
            .push(UserEquipUiAction::MoveItem {
                from: UserEquipSlotEndpoint::Equipment {
                    visual_index,
                    wire_slot_index: 1,
                },
                to: UserEquipSlotEndpoint::Inventory { slot_index },
            });
        // A real server GOTO reply updates both world admission and controller.
        world
            .resource::<NetworkBridge>()
            .send(NetworkCommand::SendRegisteredGameplay0104(
                ffone_protocol::RegisteredGameplayRequest0104::new(
                    packet::P_CL2FE_REQ_PC_GOTO,
                    ffone_protocol::wire_0104::PcGotoRequest0104 {
                        // Land beside the authored Candy Cove NPCs, above
                        // their collision floor rather than over the ocean.
                        to_x: 685700,
                        to_y: 73100,
                        to_z: -5000,
                    }
                    .encode(),
                )
                .unwrap(),
            ))
            .unwrap();
        let mut check = world.resource_mut::<EquipmentCheck>();
        check.requested_uid = Some(uid);
        check.frames = 0;
        return false;
    }
    let inventory = world
        .resource::<LocalInventoryRuntime>()
        .snapshot()
        .unwrap();
    if inventory.equipment()[1].item_id != 0 {
        return false;
    }
    let runtime = world.resource::<RuntimeStatus>();
    let character =
        character_flow::authoritative_user_equip_preview_character(runtime, inventory).unwrap();
    let expected = world
        .resource::<LoadedCharacterCreationData>()
        .0
        .resolve_character_summary(&character)
        .unwrap();
    let slot = (character.slot - 1) as usize;
    if world
        .resource::<WorldPlayerEquipmentProjection>()
        .candidate
        .is_some()
        || !matches!(
            world.resource::<NativePlayerPreviewModel>().status,
            NativePlayerPreviewStatus::ReadyAnimated { .. }
        )
        || !matches!(
            world.resource::<CharacterSelectionPortraitsModel>().slots[slot].status,
            CharacterSelectionPortraitStatus::ReadyAnimated { .. }
        )
    {
        return false;
    }
    let position = {
        let mut players =
            world.query_filtered::<(&Transform, &LegacyPlayerController), With<LocalPlayer>>();
        let (transform, controller) = players.single(world).unwrap();
        if !controller.grounded {
            return false;
        }
        ProtocolPosition::from_native(transform.translation).raw()
    };
    let location =
        ffone_client::character_selection_ui::resolve_character_selection_location(position)
            .unwrap();
    if location.district != "Candy Cove" {
        return false;
    }
    assert_eq!(
        world.resource::<NativePlayerPreviewModel>().look(),
        Some(&expected)
    );
    assert_eq!(
        world.resource::<CharacterSelectionPortraitsModel>().slots[slot].look(),
        Some(&expected)
    );
    let mut rigs =
        world.query_filtered::<&TutorialSelectedPlayerRig, With<TutorialSelectedPlayerRigActive>>();
    let rig = rigs.single(world).unwrap();
    // The world uses its own identity; the assembled parts must match every preview.
    assert_eq!(rig.look().parts, expected.parts);
    let mut check = world.resource_mut::<EquipmentCheck>();
    check.frames += 1;
    if check.frames < 60 {
        return false;
    }
    let mut character = character;
    character.position = position;
    check.expected.insert(uid, character);
    let output = world
        .resource::<Probe>()
        .output
        .join(format!("equipment-world-{uid}.png"));
    world
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(output));
    world.resource_mut::<UserEquipUiState>().close();
    true
}

pub(super) fn verify_selection(world: &World) {
    let Some(check) = world.get_resource::<EquipmentCheck>() else {
        return;
    };
    let runtime = world.resource::<RuntimeStatus>();
    let data = &world.resource::<LoadedCharacterCreationData>().0;
    let portraits = world.resource::<CharacterSelectionPortraitsModel>();
    for expected in check.expected.values() {
        let character = runtime
            .roster
            .characters
            .iter()
            .find(|c| c.pc_uid == expected.pc_uid)
            .unwrap();
        assert_eq!(character.equipment, expected.equipment);
        assert_eq!(character.position[..2], expected.position[..2]);
        let look = data.resolve_character_summary(character).unwrap();
        assert_eq!(
            portraits.slots[(character.slot - 1) as usize].look(),
            Some(&look)
        );
        if runtime.roster.selected_uid == Some(character.pc_uid) {
            assert_eq!(
                world.resource::<NativePlayerPreviewModel>().look(),
                Some(&look)
            );
        }
    }
    println!(
        "equipment-session PASS: {} updated characters, all player renders and locations",
        check.expected.len()
    );
}
