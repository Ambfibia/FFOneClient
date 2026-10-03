use super::*;

pub(super) fn collect_bank_local_controls(
    buttons: Query<&Interaction, (With<BankHelpControl>, Changed<Interaction>)>,
    state: Res<BankUiState>,
    mut modal: ResMut<BankModalState>,
    mut requests: ResMut<BankLocalRequests>,
    help: Option<ResMut<crate::game_guide_ui::GameGuideUiModel>>,
    messages: Option<ResMut<crate::system_message_ui::SystemMessageUiModel>>,
    results: Option<ResMut<crate::system_message_ui::SystemMessageUiOutbox>>,
    audio: Option<ResMut<crate::gameplay_audio::GameplayAudioRuntime>>,
    redeem_buttons: Query<&Interaction, (With<BankRedeemControl>, Changed<Interaction>)>,
    projection: Res<BankModeProjection0104>,
    input: Option<ResMut<crate::shared_input_ui::SharedInputDialog>>,
    redeem: Option<ResMut<crate::shared_input_ui::SharedRedeemCode>>,
) {
    use crate::system_message_ui::{SystemMessageButtonType, SystemMessageRequest};
    if let Some(help) = help.as_ref() {
        if modal.help != help.modal_active() {
            modal.help = help.modal_active();
        }
    }
    if let Some(mut results) = results
        && !results.is_empty()
    {
        results.drain_matching(|action| matches!(action, crate::system_message_ui::SystemMessageUiAction::Chosen { request_id, .. } if *request_id == BANK_FULL_MESSAGE_OWNER));
    }
    if let Some(mut messages) = messages {
        if state.phase == BankLifecyclePhase::Hidden {
            if messages
                .stack()
                .iter()
                .any(|request| request.request_id == BANK_FULL_MESSAGE_OWNER)
            {
                messages.remove(BANK_FULL_MESSAGE_OWNER);
            }
            if requests.full.is_some() {
                requests.full = None;
            }
        } else if requests.full.is_some()
            && let Some(location) = requests.full.take()
        {
            messages.remove(BANK_FULL_MESSAGE_OWNER);
            messages.push(SystemMessageRequest::new_localized(
                BANK_FULL_MESSAGE_OWNER,
                bank_full_localized(location),
                SystemMessageButtonType::Ok,
            ));
            modal.system_popup = true;
        }
    }
    if state.input_capabilities(*modal).close
        && buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
        && let Some(mut help) = help
        && help.open_first_use(3)
    {
        modal.help = true;
        if let Some(mut audio) = audio {
            audio.queue_gameplay_ui_sound("Open_Screen");
        }
    }
    if state.input_capabilities(*modal).close
        && redeem_buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
        && let (Some(mut input), Some(mut redeem)) = (input, redeem)
    {
        modal.inventory_popup = redeem.open(
            crate::shared_input_ui::RedeemSource::Bank {
                pc: projection.owner_pc_id,
                npc: projection.npc_id,
            },
            &mut input,
        );
    }
}
