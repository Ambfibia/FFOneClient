use super::*;

// Opt-in acceptance through real Bevy layout/focus, not injected Interaction values.
pub(super) fn exercise_pointer(
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    mut windows: Query<&mut Window>,
    slots: Query<(&BankUiElement, &UiGlobalTransform, &ComputedNode)>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut outbox: ResMut<ffone_client::bank_ui::BankUiOutbox0104>,
    mut state: ResMut<BankUiState>,
    help: Res<ffone_client::game_guide_ui::GameGuideUiModel>,
    mut input: ResMut<ffone_client::shared_input_ui::SharedInputDialog>,
    labels: Query<(&LocalizedText, &UiGlobalTransform)>,
    confirmation_labels: Query<
        (&LocalizedText, &UiGlobalTransform, &ComputedNode),
        With<ffone_client::system_message_ui::SystemMessageTextStyle>,
    >,
    popup: Res<ffone_client::bank_ui::BankItemPopupState>,
    trash: Query<&UiGlobalTransform, With<ffone_client::bank_ui::BankItemDeleteButton>>,
    mut deletion: ResMut<ffone_client::bank_ui::BankItemDeleteState>,
    messages: Res<ffone_client::system_message_ui::SystemMessageUiModel>,
) {
    if std::env::var_os("FFONE_BANK_POINTER").is_none() || preview.ready_frame.is_none() {
        return;
    }
    *frame += 1;
    let mode = std::env::var("FFONE_BANK_POINTER").unwrap_or_default();
    if mode.starts_with("popup-delete") && (24..=35).contains(&*frame) {
        if (24..=27).contains(&*frame) {
            let transform = trash.single().expect("Bank popup trash");
            let mut window = windows.single_mut().unwrap();
            let cursor = transform.translation / window.scale_factor();
            window.set_cursor_position(Some(cursor));
            if *frame == 25 {
                mouse.press(MouseButton::Left);
            }
            if *frame == 26 {
                mouse.release(MouseButton::Left);
            }
            if *frame == 27 {
                assert!(!popup.is_open());
                assert!(messages.is_popup());
                assert!(!deletion.has_ready());
                assert_eq!(messages.stack().last().unwrap().icon_quantity, 25);
                println!("Bank actual trash click opens stack confirmation; no deletion queued");
            }
        } else if mode != "popup-delete" {
            let key = if mode == "popup-delete-accept" {
                "ui.common.delete"
            } else {
                "ui.common.cancel"
            };
            if (29..=32).contains(&*frame) {
                let (_, transform, _) = confirmation_labels
                    .iter()
                    .find(|(label, _, node)| label.key == key && node.size().min_element() > 0.)
                    .expect("confirmation button label");
                let mut window = windows.single_mut().unwrap();
                let cursor = transform.translation / window.scale_factor();
                window.set_cursor_position(Some(cursor));
                if *frame == 31 {
                    mouse.press(MouseButton::Left);
                }
                if *frame == 32 {
                    mouse.release(MouseButton::Left);
                }
            }
            if *frame == 35 {
                assert!(!messages.is_popup());
                if mode == "popup-delete-accept" {
                    let intent = deletion.take_ready().expect("confirmed bank delete");
                    assert_eq!(intent.slot, 1);
                    assert_eq!(intent.item.option, 25);
                    assert!(deletion.take_ready().is_none());
                } else {
                    assert!(!deletion.has_ready());
                }
                assert!(outbox.is_empty());
                println!("Bank actual confirmation-label click passed: {mode}");
            }
        }
        return;
    }
    if mode == "popup-transfer" && (24..=27).contains(&*frame) {
        let (_, transform) = labels
            .iter()
            .find(|(label, _)| label.key == "ui.bank.popup.to_inventory")
            .expect("bank transfer label");
        let mut window = windows.single_mut().unwrap();
        let cursor = transform.translation / window.scale_factor();
        window.set_cursor_position(Some(cursor));
        if *frame == 25 {
            mouse.press(MouseButton::Left);
        }
        if *frame == 26 {
            mouse.release(MouseButton::Left);
        }
        if *frame == 27 {
            assert!(!popup.is_open());
            assert!(matches!(
                outbox.pop_front(),
                Some(ffone_client::bank_ui::BankUiCommand0104::ItemMove(_))
            ));
            assert!(outbox.is_empty());
            println!("bank popup actual transfer-label click passed");
        }
        return;
    }
    if mode == "redeem" && (24..=27).contains(&*frame) {
        if *frame == 24 {
            input.append("BeMore");
        }
        let (_, transform) = labels
            .iter()
            .find(|(label, _)| label.key == "ui.shared_input.redeem")
            .expect("redeem submit label");
        let mut window = windows.single_mut().unwrap();
        let cursor = transform.translation / window.scale_factor();
        window.set_cursor_position(Some(cursor));
        if *frame == 25 {
            mouse.press(MouseButton::Left);
        }
        if *frame == 26 {
            mouse.release(MouseButton::Left);
        }
        if *frame == 27 {
            use ffone_client::shared_input_ui::{REDEEM_INPUT_OWNER, SharedInputAction};
            assert_eq!(
                input.pop_for(REDEEM_INPUT_OWNER),
                Some(SharedInputAction::Submit {
                    owner: REDEEM_INPUT_OWNER,
                    value: "BeMore".into()
                })
            );
            println!("shared redeem actual submit-label click passed");
        }
        return;
    }
    if !(20..=23).contains(&*frame) {
        return;
    }
    let (element, button) = match mode.as_str() {
        "popup-delete" | "popup-delete-accept" | "popup-delete-cancel" => {
            (BankUiElement::InventorySlotFrame(1), MouseButton::Left)
        }
        "popup-general" | "popup-transfer" => (BankUiElement::BankSlotFrame(0), MouseButton::Left),
        "popup-long-name" | "popup-expires" | "popup-rental" | "popup-combined"
        | "popup-vehicle" | "popup-equip" => (BankUiElement::BankSlotFrame(1), MouseButton::Left),
        "popup-chest" => (BankUiElement::BankSlotFrame(2), MouseButton::Left),
        "popup-inventory" => (BankUiElement::InventorySlotFrame(0), MouseButton::Left),
        "help" => (BankUiElement::Help, MouseButton::Left),
        "redeem" => (BankUiElement::RedeemCode, MouseButton::Left),
        _ => (BankUiElement::BankSlotFrame(0), MouseButton::Right),
    };
    let (_, transform, computed) = slots
        .iter()
        .find(|(candidate, _, _)| **candidate == element)
        .expect("bank slot");
    assert!(computed.size().min_element() > 0.0);
    let mut window = windows.single_mut().unwrap();
    window.focused = true;
    let cursor = transform.translation / window.scale_factor();
    window.set_cursor_position(Some(cursor));
    if *frame == 21 {
        mouse.press(button);
    }
    if *frame == 22 {
        mouse.release(button);
    }
    if *frame == 23 {
        if mode.starts_with("popup-") {
            assert!(popup.is_open());
            assert!(outbox.is_empty());
            println!("bank actual item click opens {mode} without mutation");
            return;
        }
        if mode == "help" {
            assert!(help.modal_active());
            println!("bank real Help button passed");
            return;
        }
        if mode == "redeem" {
            assert_eq!(
                input.owner(),
                Some(ffone_client::shared_input_ui::REDEEM_INPUT_OWNER)
            );
            println!("bank real Redeem button passed without Enchant mode");
            return;
        }
        let command = outbox
            .pop_front()
            .expect("real bank pointer must emit move");
        assert!(matches!(
            command,
            ffone_client::bank_ui::BankUiCommand0104::ItemMove(_)
        ));
        assert!(outbox.is_empty());
        state.accept_item_move_success();
        println!("bank real UI focus/right-click acceptance passed: {command:?}");
    }
}
