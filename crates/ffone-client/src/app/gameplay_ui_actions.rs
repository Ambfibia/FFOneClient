//! Ordered routing for actions emitted by the legacy gameplay UI.
//!
//! Shared actions retain their cross-domain order. In the tutorial, local
//! mission and NPC actions remain queued for the virtual-server consumer.
//! Each selected action is converted into a domain-owned type before its
//! handler mutates runtime state.

mod chat;
mod gm_commands;
mod gm_extended;
mod gm_state;
pub(super) mod gm_store;
pub(super) mod gm_runtime;
pub(super) mod gm_world_labels;
#[cfg(test)]
mod hotkey_tests;
mod mission;
mod modes;
mod npc;
mod warp;
#[cfg(test)]
mod warp_tests;

use crate::app::*;

#[derive(SystemParam)]
struct WorldGameplayModalOwners<'w, 's> {
    mission: MissionOwners<'w>,
    commerce: CommerceOwners<'w>,
    modes: WorldModeOwners<'w>,
    shell: GameplayShellOwners<'w, 's>,
    race: RaceOwners<'w>,
    email: EmailOwners<'w>,
    crafting: CraftingOwners<'w>,
    audio: GameplayAudioOwners<'w>,
}

#[derive(SystemParam)]
struct MissionOwners<'w> {
    world_mission: ResMut<'w, WorldMissionRuntime>,
    delete_confirmation: ResMut<'w, MissionDeleteConfirmationRuntime>,
    tutorial: Res<'w, TutorialMissionRuntime>,
}

#[derive(SystemParam)]
struct CommerceOwners<'w> {
    guide_outbox: Res<'w, GuideUiOutbox>,
    bank_state: ResMut<'w, BankUiState>,
    bank_outbox: ResMut<'w, BankUiOutbox0104>,
    bank_production: Res<'w, BankProductionRuntime0104>,
    vendor_state: ResMut<'w, VendorUiState>,
    vendor_outbox: ResMut<'w, VendorUiOutbox0104>,
    vendor_production: Res<'w, VendorProductionRuntime0104>,
    cashmall_state: ResMut<'w, CashmallUiState0104>,
    cashmall_outbox: ResMut<'w, CashmallUiOutbox0104>,
    user_store_state: Res<'w, UserStoreUiState0104>,
}

#[derive(SystemParam)]
struct WorldModeOwners<'w> {
    world_map: ResMut<'w, WorldMapPresentation>,
    world_map_asset_status: Res<'w, WorldMapPresentationAssetStatus>,
    option_model: ResMut<'w, OptionUiModel>,
    option_outbox: ResMut<'w, OptionUiOutbox>,
    option_asset_gate: Res<'w, OptionUiAssetGate>,
    option_runtime: Res<'w, OptionProductionRuntime>,
    rule_model: ResMut<'w, RuleUiModel>,
    rule_outbox: Res<'w, RuleUiOutbox>,
    rule_runtime: ResMut<'w, RuleRuntime>,
    nano_free_tuning_model: Res<'w, NanoFreeTuningModel>,
    nano_free_tuning_production: Res<'w, NanoFreeTuningProductionRuntime>,
    transportation_model: ResMut<'w, TransportationModel>,
    transportation_catalog: Res<'w, TransportationCatalog>,
    transportation_production: ResMut<'w, TransportationProductionRuntime>,
}

#[derive(SystemParam)]
struct GameplayShellOwners<'w, 's> {
    avatar_actions: Query<'w, 's, &'static LegacyAvatarActionState, With<LocalPlayer>>,
    system_messages: ResMut<'w, SystemMessageUiModel>,
    quit_model: ResMut<'w, QuitMenuUiModel>,
    quit_runtime: Res<'w, QuitMenuRuntime>,
    resurrect_ui: Res<'w, ResurrectUiModel>,
    localization: Res<'w, Localization>,
    language: Res<'w, Language>,
    cursors: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
    players: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static mut LegacyPlayerController,
            Option<&'static LegacyWorldColliderPending>,
        ),
        With<LocalPlayer>,
    >,
}

#[derive(SystemParam)]
struct RaceOwners<'w> {
    mode: ResMut<'w, RaceModeModel>,
    catalog: Res<'w, RaceRankCatalog>,
    production: ResMut<'w, RaceProductionRuntime>,
}

#[derive(SystemParam)]
struct EmailOwners<'w> {
    catalog: Res<'w, EmailProductionCatalog0104>,
    nano_bank: Res<'w, NanoFreeTuningBank0104>,
    runtime: ResMut<'w, EmailProductionRuntime0104>,
    model: ResMut<'w, EmailUiModel>,
    actions: ResMut<'w, EmailUiOutbox>,
    audio: ResMut<'w, EmailUiAudioOutbox>,
    transport: Res<'w, EmailTransportOutbox>,
    network: Res<'w, EmailNetworkRuntime0104>,
    inbox: Res<'w, EmailNetworkInbox0104>,
    shell: ResMut<'w, EmailProductionShell0104>,
}

#[derive(SystemParam)]
struct CraftingOwners<'w> {
    barber: ResMut<'w, ffone_client::barber::BarberModel>,
    combi_catalog: Res<'w, CombiProductionCatalog0104>,
    combi_runtime: ResMut<'w, CombiProductionRuntime0104>,
    combi_shell: ResMut<'w, CombiProductionShell0104>,
    enchant_runtime: ResMut<'w, EnchantProductionRuntime0104>,
    enchant_shell: ResMut<'w, EnchantProductionShell0104>,
}

#[derive(SystemParam)]
struct GameplayAudioOwners<'w> {
    catalog: Res<'w, NativeAudioCatalog>,
    runtime: ResMut<'w, GameplayAudioRuntime>,
}

#[derive(SystemParam)]
struct NormalNpcWarpInputs<'w, 's> {
    group_ui: Res<'w, GroupUiModel>,
    inventory: Res<'w, LocalInventoryRuntime>,
    npc_sources: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkNpcAppearance0104,
            &'static GlobalTransform,
        ),
    >,
    effects: ResMut<'w, TutorialEffectRuntime>,
    production: ResMut<'w, NormalNpcWarpRuntime>,
}

#[derive(SystemParam)]
pub(super) struct WorldGameplayActionContext<'w, 's> {
    gameplay_ui: ResMut<'w, GameplayUiModel>,
    buddy_ui: Res<'w, BuddyUiModel>,
    normal_warp: NormalNpcWarpInputs<'w, 's>,
    mission_ui: ResMut<'w, MissionUiModel>,
    nanocom_messages: ResMut<'w, NanocomMessageUiModel>,
    content: Res<'w, TutorialMissionContent>,
    npc_appearances: Query<'w, 's, (Entity, &'static NetworkNpcAppearance0104)>,
    guide_ui: ResMut<'w, GuideUiModel>,
    game_guide_ui: ResMut<'w, GameGuideUiModel>,
    modal_owners: WorldGameplayModalOwners<'w, 's>,
    guide_runtime: Res<'w, GuideRuntime>,
    guide_production: ResMut<'w, GuideProductionRuntime>,
    upsell_ui: ResMut<'w, UpsellUiModel>,
    user_equip_ui: ResMut<'w, UserEquipUiState>,
    nano_viewer: ResMut<'w, ffone_client::user_equip_ui::UserEquipNanoViewerState>,
    bridge: Res<'w, NetworkBridge>,
    vehicle: Res<'w, LocalVehiclePresentationRuntime>,
    runtime: ResMut<'w, RuntimeStatus>,
}

impl WorldGameplayActionContext<'_, '_> {
    fn gameplay_modal_active(&self) -> bool {
        self.game_guide_ui.modal_active()
            || self.guide_production.modal_active(&self.guide_ui)
            || self.normal_warp.production.pending.is_some()
            || !self.modal_owners.commerce.guide_outbox.is_empty()
            || self.modal_owners.commerce.bank_state.phase != BankLifecyclePhase::Hidden
            || self.modal_owners.commerce.bank_production.modal_active()
            || !self.modal_owners.commerce.bank_outbox.is_empty()
            || self.modal_owners.commerce.vendor_state.phase != VendorLifecyclePhase::Hidden
            || self.modal_owners.commerce.vendor_production.modal_active()
            || !self.modal_owners.commerce.vendor_outbox.is_empty()
            || self.upsell_ui.visible()
            || self.modal_owners.modes.world_map.model.phase() != WorldMapPhase::Closed
            || self.modal_owners.modes.option_model.visible
            || self
                .modal_owners
                .modes
                .rule_runtime
                .modal_active(&self.modal_owners.modes.rule_model)
            || !self.modal_owners.modes.rule_outbox.is_empty()
            || self
                .modal_owners
                .modes
                .nano_free_tuning_production
                .modal_active(&self.modal_owners.modes.nano_free_tuning_model)
            || self.modal_owners.modes.transportation_model.phase() != TransportationPhase::Hidden
            || self.modal_owners.race.production.modal_active()
            || self.modal_owners.email.runtime.modal_active()
            || self.modal_owners.crafting.combi_runtime.modal_active()
            || self.modal_owners.crafting.enchant_runtime.is_active()
            || self.modal_owners.commerce.cashmall_state.phase()
                != CashmallLifecyclePhase0104::Hidden
            || self.modal_owners.commerce.user_store_state.active
    }
}

enum RoutedGameplayUiAction {
    Warp(WarpAction),
    Chat(ChatAction),
    Mode(ModeAction),
    Mission(MissionAction),
    Npc(NpcAction),
}

enum WarpAction {
    WarpAwayStarted,
    RequestWarpAway,
    NpcWarp {
        npc_id: i32,
        npc_type: i32,
        warp_id: i32,
        required_task_id: Option<i32>,
        target: TutorialWarpTarget,
    },
}

enum ChatAction {
    SelectChatChannel(ChatChannel),
    ToggleMenuChat,
    ToggleEmotes,
    SelectQuickChatItem(QuickChatItem),
    SendChat(String),
}

enum ModeAction {
    OpenNanocomMenu,
    CloseNanocomMenu,
    OpenUserEquipItemMode {
        source: ffone_client::user_equip_ui::UserEquipOpenSource,
    },
    OpenOptionFromNanocomSettings,
    OpenEmailFromNanocom,
    OpenWorldMapFromNanocom,
    OpenGameGuideFromNanocom,
    OpenQuitFromNanocom,
}

enum MissionAction {
    OpenMissionJournal,
    CloseMissionJournal,
    RequestTaskStopConfirmation {
        task_id: i32,
    },
    ConfirmTutorialExit {
        message_id: i32,
        button_type: i32,
    },
    OpenMissionAllow {
        task_id: i32,
        npc_id: i32,
    },
    OpenMissionReward {
        task_id: i32,
        npc_id: i32,
    },
    TaskStart {
        task_id: i32,
        npc_id: i32,
    },
    QuestEnd {
        task_id: i32,
        npc_id: i32,
        box1_choice: i32,
        box2_choice: i32,
    },
}

enum NpcAction {
    NpcService {
        npc_id: i32,
        service: NpcServiceKind,
    },
    NpcIconClose {
        npc_id: i32,
    },
}

impl From<GameplayUiAction> for RoutedGameplayUiAction {
    fn from(action: GameplayUiAction) -> Self {
        match action {
            GameplayUiAction::WarpAwayStarted => Self::Warp(WarpAction::WarpAwayStarted),
            GameplayUiAction::RequestWarpAway => Self::Warp(WarpAction::RequestWarpAway),
            GameplayUiAction::NpcWarp {
                npc_id,
                npc_type,
                warp_id,
                required_task_id,
                target,
            } => Self::Warp(WarpAction::NpcWarp {
                npc_id,
                npc_type,
                warp_id,
                required_task_id,
                target,
            }),
            GameplayUiAction::SelectChatChannel(channel) => {
                Self::Chat(ChatAction::SelectChatChannel(channel))
            }
            GameplayUiAction::ToggleMenuChat => Self::Chat(ChatAction::ToggleMenuChat),
            GameplayUiAction::ToggleEmotes => Self::Chat(ChatAction::ToggleEmotes),
            GameplayUiAction::SelectQuickChatItem(item) => {
                Self::Chat(ChatAction::SelectQuickChatItem(item))
            }

            GameplayUiAction::SendChat(message) => Self::Chat(ChatAction::SendChat(message)),
            GameplayUiAction::OpenNanocomMenu => Self::Mode(ModeAction::OpenNanocomMenu),
            GameplayUiAction::CloseNanocomMenu => Self::Mode(ModeAction::CloseNanocomMenu),
            GameplayUiAction::OpenUserEquipItemMode { source } => {
                Self::Mode(ModeAction::OpenUserEquipItemMode { source })
            }
            GameplayUiAction::OpenOptionFromNanocomSettings => {
                Self::Mode(ModeAction::OpenOptionFromNanocomSettings)
            }
            GameplayUiAction::OpenEmailFromNanocom => Self::Mode(ModeAction::OpenEmailFromNanocom),
            GameplayUiAction::OpenWorldMapFromNanocom => {
                Self::Mode(ModeAction::OpenWorldMapFromNanocom)
            }
            GameplayUiAction::OpenGameGuideFromNanocom => {
                Self::Mode(ModeAction::OpenGameGuideFromNanocom)
            }
            GameplayUiAction::OpenQuitFromNanocom => Self::Mode(ModeAction::OpenQuitFromNanocom),
            GameplayUiAction::OpenMissionJournal => {
                Self::Mission(MissionAction::OpenMissionJournal)
            }
            GameplayUiAction::CloseMissionJournal => {
                Self::Mission(MissionAction::CloseMissionJournal)
            }
            GameplayUiAction::RequestTaskStopConfirmation { task_id } => {
                Self::Mission(MissionAction::RequestTaskStopConfirmation { task_id })
            }
            GameplayUiAction::ConfirmTutorialExit {
                message_id,
                button_type,
            } => Self::Mission(MissionAction::ConfirmTutorialExit {
                message_id,
                button_type,
            }),
            GameplayUiAction::OpenMissionAllow { task_id, npc_id } => {
                Self::Mission(MissionAction::OpenMissionAllow { task_id, npc_id })
            }
            GameplayUiAction::OpenMissionReward { task_id, npc_id } => {
                Self::Mission(MissionAction::OpenMissionReward { task_id, npc_id })
            }
            GameplayUiAction::TaskStart { task_id, npc_id } => {
                Self::Mission(MissionAction::TaskStart { task_id, npc_id })
            }
            GameplayUiAction::QuestEnd {
                task_id,
                npc_id,
                box1_choice,
                box2_choice,
            } => Self::Mission(MissionAction::QuestEnd {
                task_id,
                npc_id,
                box1_choice,
                box2_choice,
            }),
            GameplayUiAction::NpcService { npc_id, service } => {
                Self::Npc(NpcAction::NpcService { npc_id, service })
            }
            GameplayUiAction::NpcIconClose { npc_id } => {
                Self::Npc(NpcAction::NpcIconClose { npc_id })
            }
        }
    }
}

pub(super) fn consume_world_gameplay_ui_outbox(
    state: Res<State<ClientState>>,
    mut outbox: ResMut<GameplayUiOutbox>,
    mut context: WorldGameplayActionContext,
) {
    let tutorial = *state.get() == ClientState::Tutorial;
    for action in outbox
        .drain_matching(|action| !tutorial || !tutorial_owns_gameplay_action(action))
        .into_iter()
        .map(RoutedGameplayUiAction::from)
    {
        // Recompute for every action: an earlier action in this same drained
        // batch may have opened or closed a modal.
        let gameplay_modal = context.gameplay_modal_active();
        match action {
            RoutedGameplayUiAction::Warp(action) => {
                warp::handle_warp(action, gameplay_modal, &mut context)
            }
            RoutedGameplayUiAction::Chat(action) => {
                chat::handle_chat(action, gameplay_modal, &mut context)
            }
            RoutedGameplayUiAction::Mode(action) => {
                modes::handle_modes(action, gameplay_modal, &mut context)
            }
            RoutedGameplayUiAction::Mission(action) => {
                mission::handle_mission(action, gameplay_modal, &mut context)
            }
            RoutedGameplayUiAction::Npc(action) => {
                npc::handle_npc(action, gameplay_modal, &mut context)
            }
        }
    }
}

pub(super) fn tutorial_owns_gameplay_action(action: &GameplayUiAction) -> bool {
    matches!(
        action,
        GameplayUiAction::ConfirmTutorialExit { .. }
            // These pages refer to local tutorial actors, which are absent
            // from the world's authoritative NPC/range validation query.
            | GameplayUiAction::OpenMissionAllow { .. }
            | GameplayUiAction::OpenMissionReward { .. }
            | GameplayUiAction::TaskStart { .. }
            | GameplayUiAction::QuestEnd { .. }
            | GameplayUiAction::NpcWarp { .. }
            | GameplayUiAction::NpcIconClose { .. }
    )
}

#[cfg(test)]
mod tests;
