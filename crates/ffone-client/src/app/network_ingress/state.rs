use super::*;

pub(in super::super) fn sync_reward_inventory(
    state: Res<State<ClientState>>,
    inventory: Res<LocalInventoryRuntime>,
    mut notices: ResMut<ffone_client::gameplay_ui::rewards::RewardNotices>,
) {
    if inventory.is_changed() || state.is_changed() {
        let full = (*state.get() == ClientState::World)
            .then(|| {
                inventory.snapshot().map(|snapshot| {
                    snapshot.inventory().iter().all(|item| !InventoryRuntime0104::item_is_empty(*item))
                })
            })
            .flatten();
        notices.set_inventory_full(full);
    }
}

#[derive(SystemParam)]
pub(super) struct InventoryIngress<'w> {
    pub(super) user_equip_state: ResMut<'w, UserEquipUiState>,
    pub(super) nano_viewer: ResMut<'w, ffone_client::user_equip_ui::UserEquipNanoViewerState>,
    pub(super) nano_free_tuning_model: ResMut<'w, NanoFreeTuningModel>,
    pub(super) nano_free_tuning_outbox: ResMut<'w, NanoFreeTuningUiCommandOutbox>,
    pub(super) nano_free_tuning_bank: ResMut<'w, NanoFreeTuningBank0104>,
    pub(super) nano_free_tuning_production: ResMut<'w, NanoFreeTuningProductionRuntime>,
    pub(super) quick_slot_ui: ResMut<'w, QuickSlotUiModel>,
    pub(super) inventory: ResMut<'w, LocalInventoryRuntime>,
    pub(super) user_equip_production: ResMut<'w, UserEquipProductionRuntime0104>,
    pub(super) local_vehicle: ResMut<'w, LocalVehiclePresentationRuntime>,
    pub(super) skill_buff_ui: ResMut<'w, SkillBuffUiModel>,
    pub(super) movement_buffs: ResMut<'w, movement_buffs::MovementBuffs>,
    pub(super) resurrect_ui: ResMut<'w, ResurrectUiModel>,
    pub(super) resurrect_context: ResMut<'w, ResurrectUiContext>,
    pub(super) resurrect_outbox: ResMut<'w, ResurrectUiOutbox>,
}

#[derive(SystemParam)]
pub(super) struct NpcModeIngress<'w> {
    pub(super) barber: ResMut<'w, ffone_client::barber::BarberInbox>,
    pub(super) bank_state: ResMut<'w, BankUiState>,
    pub(super) bank_modal: ResMut<'w, BankModalState>,
    pub(super) bank_projection: ResMut<'w, BankModeProjection0104>,
    pub(super) bank_outbox: ResMut<'w, BankUiOutbox0104>,
    pub(super) bank_production: ResMut<'w, BankProductionRuntime0104>,
    pub(super) bank_system_runtime: ResMut<'w, BankSystemMessageRuntime>,
    pub(super) guide_ui: ResMut<'w, GuideUiModel>,
    pub(super) guide_outbox: ResMut<'w, GuideUiOutbox>,
    pub(super) guide_audio: ResMut<'w, GuideUiAudioOutbox>,
    pub(super) guide_runtime: ResMut<'w, GuideRuntime>,
    pub(super) guide_production: ResMut<'w, GuideProductionRuntime>,
    pub(super) normal_npc_warp: ResMut<'w, NormalNpcWarpRuntime>,
    pub(super) vendor_state: ResMut<'w, VendorUiState>,
    pub(super) vendor_modal: ResMut<'w, VendorModalState>,
    pub(super) vendor_projection: ResMut<'w, VendorModeProjection0104>,
    pub(super) vendor_outbox: ResMut<'w, VendorUiOutbox0104>,
    pub(super) vendor_production: ResMut<'w, VendorProductionRuntime0104>,
    pub(super) vendor_system_runtime: ResMut<'w, VendorSystemMessageRuntime>,
    pub(super) upsell_ui: ResMut<'w, UpsellUiModel>,
    pub(super) upsell_outbox: ResMut<'w, UpsellUiOutbox>,
    pub(super) upsell_audio: ResMut<'w, UpsellUiAudioOutbox>,
    pub(super) quit_menu: ResMut<'w, QuitMenuUiModel>,
    pub(super) quit_outbox: ResMut<'w, QuitMenuUiOutbox>,
    pub(super) quit_runtime: ResMut<'w, QuitMenuRuntime>,
    pub(super) system_messages: ResMut<'w, SystemMessageUiModel>,
}

#[derive(SystemParam)]
pub(super) struct WorldModeIngress<'w> {
    pub(super) reward_notices: ResMut<'w, ffone_client::gameplay_ui::rewards::RewardNotices>,
    pub(super) world_map: ResMut<'w, WorldMapPresentation>,
    pub(super) world_map_production: ResMut<'w, WorldMapProductionRuntime>,
    pub(super) world_map_clock: ResMut<'w, WorldMapServerClock>,
    pub(super) transportation_model: ResMut<'w, TransportationModel>,
    pub(super) transportation_outbox: ResMut<'w, TransportationUiCommandOutbox>,
    pub(super) transportation_production: ResMut<'w, TransportationProductionRuntime>,
    pub(super) race_mode: ResMut<'w, RaceModeModel>,
    pub(super) race_mode_commands: ResMut<'w, RaceModeUiCommandOutbox>,
    pub(super) race_reward: ResMut<'w, RaceRewardPresentation>,
    pub(super) race_rank: ResMut<'w, RaceRankModel>,
    pub(super) race_rank_commands: ResMut<'w, RaceRankUiCommandOutbox>,
    pub(super) race_production: ResMut<'w, RaceProductionRuntime>,
    pub(super) race_frames: ResMut<'w, RaceNetworkFrameInbox>,
    pub(super) combi_runtime: ResMut<'w, CombiProductionRuntime0104>,
    pub(super) combi_state: ResMut<'w, CombiUiState0104>,
    pub(super) combi_projection: ResMut<'w, CombiModeProjection0104>,
    pub(super) combi_outbox: ResMut<'w, CombiUiOutbox0104>,
    pub(super) combi_shell: ResMut<'w, CombiProductionShell0104>,
    pub(super) combi_frames: ResMut<'w, CombiNetworkFrameInbox0104>,
    pub(super) enchant_runtime: ResMut<'w, EnchantProductionRuntime0104>,
    pub(super) enchant_projection: ResMut<'w, EnchantModeProjection0104>,
    pub(super) enchant_outbox: ResMut<'w, EnchantUiOutbox0104>,
    pub(super) enchant_shell: ResMut<'w, EnchantProductionShell0104>,
    pub(super) enchant_frames: ResMut<'w, EnchantNetworkFrameInbox0104>,
    pub(super) option_runtime: Res<'w, OptionProductionRuntime>,
    pub(super) gameplay_ui: Res<'w, GameplayUiModel>,
    pub(super) localization: Res<'w, Localization>,
    pub(super) language: Res<'w, Language>,
    pub(super) mission_ui: ResMut<'w, MissionUiModel>,
    pub(super) world_mission: ResMut<'w, WorldMissionRuntime>,
    pub(super) world_nano_authority: ResMut<'w, WorldNanoAuthorityInbox0104>,
    pub(super) tutorial_content: Res<'w, TutorialMissionContent>,
}

#[derive(SystemParam)]
pub(in super::super) struct WorldSpawnRuntime<'w, 's> {
    pub(super) session: NetworkSessionIngress<'w>,
    pub(super) social: SocialIngress<'w>,
    pub(super) inventory: InventoryIngress<'w>,
    pub(super) npc_modes: NpcModeIngress<'w>,
    pub(super) world_modes: WorldModeIngress<'w>,
    pub(super) queries: WorldIngressQueries<'w, 's>,
}
