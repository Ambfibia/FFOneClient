//! Gameplay modal models and modal/chat input gates.

use super::guide::GuideProductionRuntime;
use super::nano_free_tuning::NanoFreeTuningProductionRuntime;
use super::race::RaceProductionRuntime;
use super::state::ClientState;
use bevy::{ecs::system::SystemParam, prelude::*};
use ffone_client::{
    bank_runtime::BankProductionRuntime0104,
    bank_ui::{BankLifecyclePhase, BankUiOutbox0104, BankUiState},
    buddy_ui::BuddyUiModel,
    cashmall_ui::{CashmallLifecyclePhase0104, CashmallUiState0104},
    combi_runtime::CombiProductionRuntime0104,
    email_runtime::EmailProductionRuntime0104,
    enchant_runtime::EnchantProductionRuntime0104,
    game_guide_ui::GameGuideUiModel,
    gameplay_ui::GameplayUiModel,
    guide_ui::GuideUiModel,
    mission_ui::MissionUiModel,
    nano_free_tuning_ui::NanoFreeTuningModel,
    nanocom_message_ui::NanocomMessageUiModel,
    option_ui::OptionUiModel,
    pc2pc_ui::Pc2pcUiModel0104,
    quit_menu_runtime::QuitMenuRuntime,
    quit_menu_ui::QuitMenuUiModel,
    resurrect_ui::ResurrectUiModel,
    rule_runtime::RuleRuntime,
    rule_ui::RuleUiModel,
    system_message_ui::SystemMessageUiModel,
    transportation_ui::{TransportationModel, TransportationPhase},
    upsell_ui::UpsellUiModel,
    user_equip_ui::UserEquipUiState,
    user_store_ui::UserStoreUiState0104,
    vendor_runtime::VendorProductionRuntime0104,
    vendor_ui::{VendorLifecyclePhase, VendorUiOutbox0104, VendorUiState},
    world_map::{WorldMapPhase, WorldMapPresentation},
};

#[derive(SystemParam)]
pub(super) struct GameplayModalModels<'w> {
    pub(super) gameplay_ui: Option<Res<'w, GameplayUiModel>>,
    pub(super) buddy_ui: Option<Res<'w, BuddyUiModel>>,
    pub(super) system_messages: Option<Res<'w, SystemMessageUiModel>>,
    pub(super) nanocom_messages: Option<Res<'w, NanocomMessageUiModel>>,
    pub(super) quit_menu: Option<Res<'w, QuitMenuUiModel>>,
    pub(super) quit_runtime: Option<Res<'w, QuitMenuRuntime>>,
    pub(super) option_ui: Option<Res<'w, OptionUiModel>>,
    pub(super) resurrect_ui: Option<Res<'w, ResurrectUiModel>>,
    pub(super) upsell_ui: Option<Res<'w, UpsellUiModel>>,
    pub(super) guide_ui: Option<Res<'w, GuideUiModel>>,
    pub(super) game_guide_ui: Option<Res<'w, GameGuideUiModel>>,
    pub(super) guide_production: Option<Res<'w, GuideProductionRuntime>>,
    pub(super) bank_ui: Option<Res<'w, BankUiState>>,
    pub(super) bank_production: Option<Res<'w, BankProductionRuntime0104>>,
    pub(super) vendor_ui: Option<Res<'w, VendorUiState>>,
    pub(super) vendor_production: Option<Res<'w, VendorProductionRuntime0104>>,
    pub(super) rule_ui: Option<Res<'w, RuleUiModel>>,
    pub(super) rule_runtime: Option<Res<'w, RuleRuntime>>,
    pub(super) nano_free_tuning: Option<Res<'w, NanoFreeTuningModel>>,
    pub(super) nano_free_tuning_production: Option<Res<'w, NanoFreeTuningProductionRuntime>>,
    pub(super) user_equip_ui: Option<Res<'w, UserEquipUiState>>,
    pub(super) pc2pc_ui: Option<Res<'w, Pc2pcUiModel0104>>,
    pub(super) player_menu: Option<Res<'w, super::group_pc2pc::WorldPc2pcOfferPrompt>>,
    pub(super) world_map: Option<Res<'w, WorldMapPresentation>>,
    pub(super) transportation: Option<Res<'w, TransportationModel>>,
    pub(super) race_production: Option<Res<'w, RaceProductionRuntime>>,
    pub(super) email_runtime: Option<Res<'w, EmailProductionRuntime0104>>,
    pub(super) combi_runtime: Option<Res<'w, CombiProductionRuntime0104>>,
    pub(super) barber: Option<Res<'w, ffone_client::barber::BarberModel>>,
    pub(super) enchant_runtime: Option<Res<'w, EnchantProductionRuntime0104>>,
    pub(super) cashmall_ui: Option<Res<'w, CashmallUiState0104>>,
    pub(super) user_store_ui: Option<Res<'w, UserStoreUiState0104>>,
}

impl GameplayModalModels<'_> {
    pub(super) fn chat_active(&self, state: ClientState) -> bool {
        state == ClientState::World
            && self.gameplay_ui.as_ref().is_some_and(|gameplay_ui| {
                gameplay_ui.visible && gameplay_ui.chat.input_enabled && gameplay_ui.chat.active
            })
    }

    pub(super) fn buddy_modal(&self) -> bool {
        self.buddy_ui
            .as_ref()
            .is_some_and(|buddy_ui| buddy_ui.add_dialog_open())
    }

    pub(super) fn system_popup(&self) -> bool {
        self.system_messages
            .as_ref()
            .is_some_and(|messages| messages.is_popup())
    }

    pub(super) fn nanocom_popup(&self) -> bool {
        self.nanocom_messages
            .as_ref()
            .is_some_and(|messages| messages.expanded_visible())
    }

    pub(super) fn quit_modal(&self) -> bool {
        self.quit_menu.as_ref().is_some_and(|menu| menu.visible)
            || self
                .quit_runtime
                .as_ref()
                .is_some_and(|runtime| runtime.is_waiting_for_server())
    }

    pub(super) fn option_modal(&self) -> bool {
        self.option_ui.as_ref().is_some_and(|model| model.visible)
    }

    pub(super) fn resurrect_modal(&self) -> bool {
        self.resurrect_ui.as_ref().is_some_and(|menu| menu.visible)
    }

    pub(super) fn upsell_modal(&self) -> bool {
        self.upsell_ui.as_ref().is_some_and(|menu| menu.visible())
    }

    pub(super) fn guide_modal(&self) -> bool {
        self.guide_ui.as_ref().is_some_and(|model| {
            self.guide_production
                .as_ref()
                .is_some_and(|production| production.modal_active(model))
        })
    }

    pub(super) fn game_guide_modal(&self) -> bool {
        self.game_guide_ui
            .as_ref()
            .is_some_and(|model| model.modal_active())
    }

    pub(super) fn bank_modal(&self) -> bool {
        self.bank_ui.as_ref().is_some_and(|state| {
            state.phase != BankLifecyclePhase::Hidden
                || self
                    .bank_production
                    .as_ref()
                    .is_some_and(|production| production.modal_active())
        })
    }

    pub(super) fn vendor_modal(&self) -> bool {
        self.vendor_ui.as_ref().is_some_and(|state| {
            state.phase != VendorLifecyclePhase::Hidden
                || self
                    .vendor_production
                    .as_ref()
                    .is_some_and(|production| production.modal_active())
        })
    }

    pub(super) fn rule_modal(&self) -> bool {
        self.rule_ui.as_ref().is_some_and(|model| {
            self.rule_runtime
                .as_ref()
                .is_some_and(|runtime| runtime.modal_active(model))
        })
    }

    pub(super) fn nano_free_tuning_modal(&self) -> bool {
        self.nano_free_tuning.as_ref().is_some_and(|model| {
            self.nano_free_tuning_production
                .as_ref()
                .is_some_and(|production| production.modal_active(model))
        })
    }

    pub(super) fn user_equip_modal(&self) -> bool {
        self.user_equip_ui
            .as_ref()
            .is_some_and(|state| state.is_active())
    }

    pub(super) fn pc2pc_modal(&self) -> bool {
        self.player_menu.as_ref().is_some_and(|menu| menu.selected.is_some()) || self.pc2pc_ui
            .as_ref()
            .is_some_and(|model| model.state.phase.renders_shell())
    }

    pub(super) fn world_map_modal(&self) -> bool {
        self.world_map
            .as_ref()
            .is_some_and(|presentation| presentation.model.phase() != WorldMapPhase::Closed)
    }

    pub(super) fn transportation_modal(&self) -> bool {
        self.transportation
            .as_ref()
            .is_some_and(|model| model.phase() != TransportationPhase::Hidden)
    }

    pub(super) fn transportation_world_owned(&self) -> bool {
        // Game mode 19 owns only the route-selection modal. Once the modal
        // closes, a skyway is clean `currentBus`/moving-platform gameplay in
        // MainGame, so NanoCom and the computer modes remain available.
        self.transportation_modal()
    }

    pub(super) fn race_modal(&self) -> bool {
        self.race_production
            .as_ref()
            .is_some_and(|production| production.modal_active())
    }

    pub(super) fn email_modal(&self) -> bool {
        self.email_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.modal_active())
    }

    pub(super) fn barber_modal(&self) -> bool {
        self.barber.as_ref().is_some_and(|model| model.active())
    }

    pub(super) fn combi_modal(&self) -> bool {
        self.combi_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.modal_active())
    }

    pub(super) fn enchant_modal(&self) -> bool {
        self.enchant_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.is_active())
    }

    pub(super) fn cashmall_modal(&self) -> bool {
        self.cashmall_ui
            .as_ref()
            .is_some_and(|state| state.phase() != CashmallLifecyclePhase0104::Hidden)
    }

    pub(super) fn user_store_modal(&self) -> bool {
        self.user_store_ui
            .as_ref()
            .is_some_and(|state| state.active)
    }
}

#[derive(SystemParam)]
pub(super) struct GameplayUiModalInputs<'w> {
    pub(super) guide_ui: Res<'w, GuideUiModel>,
    pub(super) guide_production: Res<'w, GuideProductionRuntime>,
    pub(super) bank_state: Res<'w, BankUiState>,
    pub(super) bank_production: Res<'w, BankProductionRuntime0104>,
    pub(super) bank_outbox: Res<'w, BankUiOutbox0104>,
    pub(super) vendor_state: Res<'w, VendorUiState>,
    pub(super) vendor_production: Res<'w, VendorProductionRuntime0104>,
    pub(super) vendor_outbox: Res<'w, VendorUiOutbox0104>,
    pub(super) rule_ui: Res<'w, RuleUiModel>,
    pub(super) rule_runtime: Res<'w, RuleRuntime>,
    pub(super) nano_free_tuning: Res<'w, NanoFreeTuningModel>,
    pub(super) nano_free_tuning_production: Res<'w, NanoFreeTuningProductionRuntime>,
    pub(super) upsell_ui: Res<'w, UpsellUiModel>,
    pub(super) user_equip_ui: Res<'w, UserEquipUiState>,
    pub(super) pc2pc_ui: Res<'w, Pc2pcUiModel0104>,
    pub(super) world_map: Res<'w, WorldMapPresentation>,
    pub(super) transportation: Option<Res<'w, TransportationModel>>,
    pub(super) race_production: Res<'w, RaceProductionRuntime>,
    pub(super) email_runtime: Res<'w, EmailProductionRuntime0104>,
    pub(super) combi_runtime: Res<'w, CombiProductionRuntime0104>,
    pub(super) enchant_runtime: Res<'w, EnchantProductionRuntime0104>,
    pub(super) cashmall_ui: Option<Res<'w, CashmallUiState0104>>,
    pub(super) user_store_ui: Option<Res<'w, UserStoreUiState0104>>,
}

impl GameplayUiModalInputs<'_> {
    pub(super) fn transportation_modal(&self) -> bool {
        self.transportation
            .as_ref()
            .is_some_and(|model| model.phase() != TransportationPhase::Hidden)
    }

    pub(super) fn cashmall_modal(&self) -> bool {
        self.cashmall_ui
            .as_ref()
            .is_some_and(|state| state.phase() != CashmallLifecyclePhase0104::Hidden)
    }

    pub(super) fn user_store_modal(&self) -> bool {
        self.user_store_ui
            .as_ref()
            .is_some_and(|state| state.active)
    }
}

pub(super) fn sync_nanocom_foreign_modal_suppression(
    modals: GameplayModalModels,
    mut mission_ui: ResMut<MissionUiModel>,
) {
    let suppressed = mission_ui.system_popup_active()
        || modals.system_popup()
        || modals.quit_modal()
        || modals.resurrect_modal()
        || modals.pc2pc_modal()
        || modals.game_guide_modal()
        || modals.guide_modal()
        || modals.transportation_world_owned()
        || modals.race_modal()
        || modals.email_modal()
        || (modals.combi_modal() || modals.barber_modal())
        || modals.enchant_modal()
        || modals.cashmall_modal()
        || modals.user_store_modal();
    mission_ui.set_nanocom_foreign_modal_suppressed(suppressed);
}

pub(super) fn sync_guide_chat_input_gate(
    mission_ui: Res<MissionUiModel>,
    modals: GameplayUiModalInputs,
    option_ui: Res<OptionUiModel>,
    system_messages: Res<SystemMessageUiModel>,
    quit_menu: Res<QuitMenuUiModel>,
    quit_runtime: Res<QuitMenuRuntime>,
    resurrect_ui: Res<ResurrectUiModel>,
    mut gameplay_ui: ResMut<GameplayUiModel>,
) {
    if mission_ui.chat_input_blocked()
        || modals.guide_production.modal_active(&modals.guide_ui)
        || modals.bank_state.phase != BankLifecyclePhase::Hidden
        || modals.bank_production.modal_active()
        || !modals.bank_outbox.is_empty()
        || modals.vendor_state.phase != VendorLifecyclePhase::Hidden
        || modals.vendor_production.modal_active()
        || !modals.vendor_outbox.is_empty()
        || modals.rule_runtime.modal_active(&modals.rule_ui)
        || modals
            .nano_free_tuning_production
            .modal_active(&modals.nano_free_tuning)
        || modals.upsell_ui.visible()
        || modals.user_equip_ui.is_active()
        || modals.pc2pc_ui.state.phase.renders_shell()
        || modals.world_map.model.phase() != WorldMapPhase::Closed
        || modals.transportation_modal()
        || modals.race_production.modal_active()
        || modals.email_runtime.modal_active()
        || modals.combi_runtime.modal_active()
        || modals.enchant_runtime.is_active()
        || modals.cashmall_modal()
        || modals.user_store_modal()
        || option_ui.visible
        || system_messages.is_popup()
        || quit_menu.visible
        || quit_runtime.is_waiting_for_server()
        || resurrect_ui.visible
    {
        gameplay_ui.chat.input_enabled = false;
        gameplay_ui.chat.active = false;
        gameplay_ui.chat.input.clear();
    }
}
