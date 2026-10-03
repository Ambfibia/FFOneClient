use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpsellUiCommand {
    Close,
    Continue,
    OpenPayPage,
    NotRightNow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpsellUiAction {
    Exit,
    OpenPayPage,
}

pub fn apply_upsell_ui_command(
    model: &mut UpsellUiModel,
    outbox: &mut UpsellUiOutbox,
    audio: &mut UpsellUiAudioOutbox,
    command: UpsellUiCommand,
) -> bool {
    if !model.controls_enabled() {
        return false;
    }
    match command {
        UpsellUiCommand::Close => {
            model.close();
            outbox.push(UpsellUiAction::Exit);
            true
        }
        UpsellUiCommand::OpenPayPage => {
            if model.active_mode != Some(UpsellUiMode::Upgrade) {
                return false;
            }
            outbox.push(UpsellUiAction::OpenPayPage);
            true
        }
        UpsellUiCommand::NotRightNow => {
            if model.active_mode != Some(UpsellUiMode::Upgrade) {
                return false;
            }
            model.close();
            outbox.push(UpsellUiAction::Exit);
            true
        }
        UpsellUiCommand::Continue => match model.active_mode {
            Some(UpsellUiMode::Upgrade) => {
                model.close();
                outbox.push(UpsellUiAction::Exit);
                true
            }
            Some(UpsellUiMode::NewsPayZone | UpsellUiMode::NewsFreeZone) => {
                continue_news(model, outbox, audio)
            }
            None => false,
        },
    }
}
