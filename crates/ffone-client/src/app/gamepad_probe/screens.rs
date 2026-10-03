//! Production inventory controls driven through the same synthetic pad as the HUD.
use super::*;
use ffone_client::user_equip_ui::{UserEquipItemPopupState, UserEquipMode, UserEquipUiElement};

mod journal;

fn focus_element(world: &mut World, element: UserEquipUiElement) -> Entity {
    let entity = world
        .query::<(Entity, &UserEquipUiElement)>()
        .iter(world)
        .find(|(_, candidate)| **candidate == element)
        .expect("production inventory control")
        .0;
    assert!(world.get::<Button>(entity).is_some());
    world
        .resource_mut::<super::super::gamepad_ui::PadUiFocus>()
        .entity = Some(entity);
    entity
}

pub(super) fn screenshot(world: &mut World, name: &str) {
    let path = PathBuf::from(env::var_os("FFONE_PERF_OUTPUT").unwrap()).join(name);
    world
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(path));
}

pub(super) fn drive(world: &mut World, probe: &mut Probe) -> bool {
    if probe.phase >= 36 {
        return journal::drive(world, probe);
    }
    let delay = match probe.phase { 13 => 90, 23 => 60, _ => 6 };
    if probe.frames < delay {
        return false;
    }
    match probe.phase {
        12 => {
            world.resource_mut::<OptionUiModel>().visible = false;
            let mut load = ffone_protocol::PcLoadData0104::zeroed();
            let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET;
            load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&10i16.to_le_bytes());
            load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&1i32.to_le_bytes());
            world.resource_mut::<LocalInventoryRuntime>().seed(1, &load);
            world.resource_mut::<UserEquipUiState>().open_item_mode();
            probe.button = None;
        }
        13 => {
            assert!(
                world.resource::<UserEquipUiState>().is_active(),
                "inventory fixture closed: state={:?}, inventory={}, resurrect={}, bank={}, message={}",
                world.resource::<State<ClientState>>().get(),
                world
                    .resource::<LocalInventoryRuntime>()
                    .snapshot()
                    .is_some(),
                world.resource::<ResurrectUiModel>().visible,
                world.resource::<BankProductionRuntime0104>().modal_active(),
                world.resource::<RuntimeStatus>().message
            );
            if world.resource::<UserEquipUiState>().phase()
                != ffone_client::user_equip_ui::UserEquipLifecyclePhase::Visible
            {
                return false;
            }
            probe.first_focus = Some(focus_element(
                world,
                UserEquipUiElement::InventorySlotFrame(0),
            ));
        }
        14 => {
            let entity = probe.first_focus.unwrap();
            assert_eq!(
                world
                    .resource::<super::super::gamepad_ui::PadUiFocus>()
                    .entity,
                Some(entity)
            );
            assert!(
                world
                    .get::<super::super::gamepad_ui::PadUiHighlight>(entity)
                    .is_some(),
                "inventory cell has a visible pad highlight"
            );
            screenshot(world, "gamepad-inventory-focus.png");
            probe.button = Some(GamepadButton::South);
        }
        15 => {
            assert!(
                world
                    .resource::<UserEquipItemPopupState>()
                    .selected()
                    .is_some(),
                "A opens item details without a mouse drag"
            );
            probe.button = Some(GamepadButton::East);
        }
        16 => {
            assert!(
                world
                    .resource::<UserEquipItemPopupState>()
                    .selected()
                    .is_none(),
                "B closes item details"
            );
            assert!(
                world.resource::<UserEquipUiState>().is_active(),
                "B preserves inventory behind the item card"
            );
            probe.button = Some(GamepadButton::RightTrigger);
        }
        17 => {
            assert_eq!(
                world.resource::<UserEquipUiState>().mode(),
                UserEquipMode::Nano,
                "RB selects Nanos"
            );
            probe.button = Some(GamepadButton::LeftTrigger);
        }
        18 => {
            assert_eq!(
                world.resource::<UserEquipUiState>().mode(),
                UserEquipMode::Item,
                "LB returns to items"
            );
            probe.button = None;
            focus_element(world, UserEquipUiElement::InventorySlotFrame(0));
        }
        19 => {
            probe.button = Some(GamepadButton::South);
        }
        20 => {
            assert!(
                world
                    .resource::<UserEquipItemPopupState>()
                    .selected()
                    .is_some(),
                "the same item can be opened a second time"
            );
            focus_element(world, UserEquipUiElement::ItemPopupClose);
            probe.button = None;
        }
        21 => {
            let entity = world
                .resource::<super::super::gamepad_ui::PadUiFocus>()
                .entity
                .unwrap();
            assert_eq!(
                world.get::<UserEquipUiElement>(entity),
                Some(&UserEquipUiElement::ItemPopupClose)
            );
            assert!(
                world
                    .get::<super::super::gamepad_ui::PadUiHighlight>(entity)
                    .is_some(),
                "close cross has the same visible highlight"
            );
            screenshot(world, "gamepad-item-close-focus.png");
            probe.button = Some(GamepadButton::South);
        }
        22 => {
            assert!(
                world
                    .resource::<UserEquipItemPopupState>()
                    .selected()
                    .is_none()
            );
            println!(
                "BUG014 SCREENS PASS: cyan item and close focus, A opens/reopens item, B closes only details, LB/RB switch Item/Nano, A closes focused cross"
            );
            probe.button = None;
            world.resource_mut::<UserEquipUiState>().close();
            probe.phase = 36;
            return true;
        }
        23 => {
            assert_eq!(
                *world.resource::<State<ClientState>>().get(),
                ClientState::CharacterSelect
            );
            let focus = world
                .resource::<super::super::gamepad_ui::PadUiFocus>()
                .entity
                .unwrap();
            assert!(
                world
                    .get::<ffone_client::ui::shared::controller::ControllerUiDefault>(focus)
                    .is_some(),
                "character slot receives initial focus across rendering roots"
            );
            screenshot(world, "gamepad-character-selection.png");
            focus_rect(world, 244.0, 449.0);
        }
        24 => {
            probe.button = Some(GamepadButton::South);
        }
        25 => {
            assert_eq!(
                *world.resource::<State<ClientState>>().get(),
                ClientState::CharacterCreateIntro
            );
            probe.button = None;
        }
        26 => {
            if !world
                .resource::<DexterShipCutsceneRuntime>()
                .presentation_ready
            {
                return false;
            }
            probe.button = Some(GamepadButton::South);
        }
        27 => {
            assert_eq!(
                *world.resource::<State<ClientState>>().get(),
                ClientState::CharacterCreate,
                "A skips the Dexter creation cutscene"
            );
            probe.button = None;
            focus_rect(world, 120.0, 276.0);
            probe.value = world.resource::<CharacterCreationUiModel>().name_indices[0];
        }
        28 => {
            probe.button = Some(GamepadButton::South);
        }
        29 => {
            let value = world.resource::<CharacterCreationUiModel>().name_indices[0];
            assert_ne!(value, probe.value, "A advances the name roulette");
            probe.value = value;
            probe.button = None;
        }
        30 => {
            probe.button = Some(GamepadButton::South);
        }
        31 => {
            assert_ne!(
                world.resource::<CharacterCreationUiModel>().name_indices[0],
                probe.value,
                "the same roulette arrow accepts a second A"
            );
            probe.button = None;
            world.resource_mut::<CharacterCreationUiModel>().screen =
                ffone_client::character_creation_ui::CharacterCreationScreen::Appearance;
        }
        32 => {
            focus_rect(world, 680.0, 194.0);
            probe.value = usize::from(world.resource::<CharacterCreationUiModel>().appearance.hair);
        }
        33 => {
            probe.button = Some(GamepadButton::South);
            probe.held_at = world.resource::<Time>().elapsed_secs_f64();
        }
        34 => {
            if world.resource::<Time>().elapsed_secs_f64() - probe.held_at < 0.7 {
                return false;
            }
            let model = world.resource::<CharacterCreationUiModel>();
            assert_ne!(
                usize::from(model.appearance.hair),
                probe.value,
                "held A advances hairstyles"
            );
            screenshot(world, "gamepad-character-hair-hold.png");
            probe.button = None;
        }
        35 => {
            println!(
                "BUG014 FRONTEND PASS: initial character slot focus, A creates and skips intro, repeated A advances name roulette, held A advances hairstyles"
            );
            world.write_message(AppExit::Success);
        }
        _ => {}
    }
    probe.phase += 1;
    true
}

fn focus_rect(world: &mut World, x: f32, y: f32) {
    let entity = world
        .query_filtered::<(Entity, &Node), With<Button>>()
        .iter(world)
        .find(|(_, node)| node.left == px(x) && node.top == px(y))
        .expect("authored production control rectangle")
        .0;
    world
        .resource_mut::<super::super::gamepad_ui::PadUiFocus>()
        .entity = Some(entity);
}
