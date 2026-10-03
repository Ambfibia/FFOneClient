use crate::app::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NetworkSessionBoundary {
    Connecting,
    Error,
    Disconnected,
}

#[derive(Clone, Copy, Debug, Eq, Message, PartialEq)]
pub(super) struct NetworkSessionReset {
    boundary: NetworkSessionBoundary,
}

impl NetworkSessionReset {
    pub(super) const fn new(boundary: NetworkSessionBoundary) -> Self {
        Self { boundary }
    }

    const fn resets_race(self) -> bool {
        matches!(self.boundary, NetworkSessionBoundary::Connecting)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(super) enum NetworkSessionLifecycleSet {
    Apply,
}

pub(super) struct NetworkSessionLifecyclePlugin;

impl Plugin for NetworkSessionLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NetworkSessionReset>().add_systems(
            Update,
            (
                reset_email_on_network_boundary,
                reset_world_modes_on_network_boundary,
                reset_social_on_network_boundary,
                reset_inventory_on_network_boundary,
                reset_guide_vendor_on_network_boundary,
                reset_commerce_on_network_boundary,
                reset_overlays_on_network_boundary,
                finish_network_boundary_reset,
            )
                .chain()
                .in_set(NetworkSessionLifecycleSet::Apply)
                .after(poll_network)
                .run_if(on_message::<NetworkSessionReset>),
        );
    }
}

#[derive(SystemParam)]
struct EmailSessionOwners<'w> {
    network: ResMut<'w, EmailNetworkRuntime0104>,
    inbox: ResMut<'w, EmailNetworkInbox0104>,
    transport: ResMut<'w, EmailTransportOutbox>,
    runtime: ResMut<'w, EmailProductionRuntime0104>,
    model: ResMut<'w, EmailUiModel>,
    actions: ResMut<'w, EmailUiOutbox>,
    audio: ResMut<'w, EmailUiAudioOutbox>,
    shell: ResMut<'w, EmailProductionShell0104>,
    system_messages: ResMut<'w, SystemMessageUiModel>,
}

fn reset_email_on_network_boundary(
    mut commands: Commands,
    mut resets: MessageReader<NetworkSessionReset>,
    mut owners: EmailSessionOwners,
) {
    if resets.read().count() == 0 {
        return;
    }
    reset_email_shell_0104(
        &mut commands,
        &mut owners.runtime,
        &mut owners.model,
        &mut owners.actions,
        &mut owners.audio,
        &mut owners.transport,
        &mut owners.network,
        &mut owners.inbox,
        &mut owners.shell,
        &mut owners.system_messages,
    );
}

#[derive(SystemParam)]
struct WorldModeSessionOwners<'w> {
    world_map: ResMut<'w, WorldMapPresentation>,
    world_map_production: ResMut<'w, WorldMapProductionRuntime>,
    world_map_clock: ResMut<'w, WorldMapServerClock>,
    transportation: ResMut<'w, TransportationModel>,
    transportation_outbox: ResMut<'w, TransportationUiCommandOutbox>,
    transportation_production: ResMut<'w, TransportationProductionRuntime>,
    race_mode: ResMut<'w, RaceModeModel>,
    race_mode_commands: ResMut<'w, RaceModeUiCommandOutbox>,
    race_reward: ResMut<'w, RaceRewardPresentation>,
    race_rank: ResMut<'w, RaceRankModel>,
    race_rank_commands: ResMut<'w, RaceRankUiCommandOutbox>,
    race_production: ResMut<'w, RaceProductionRuntime>,
    race_frames: ResMut<'w, RaceNetworkFrameInbox>,
}

fn reset_world_modes_on_network_boundary(
    mut resets: MessageReader<NetworkSessionReset>,
    mut owners: WorldModeSessionOwners,
) {
    let reset_race = resets.read().copied().any(NetworkSessionReset::resets_race);
    reset_world_map_shell(
        &mut owners.world_map,
        &mut owners.world_map_production,
        &mut owners.world_map_clock,
    );
    reset_transportation_shell(
        &mut owners.transportation,
        &mut owners.transportation_outbox,
        &mut owners.transportation_production,
    );
    if reset_race {
        reset_race_shell(
            &mut owners.race_mode,
            &mut owners.race_mode_commands,
            &mut owners.race_reward,
            &mut owners.race_rank,
            &mut owners.race_rank_commands,
            &mut owners.race_production,
            &mut owners.race_frames,
        );
    }
}

#[derive(SystemParam)]
struct SocialSessionOwners<'w> {
    buddy_ui: ResMut<'w, BuddyUiModel>,
    buddy_runtime: ResMut<'w, BuddyRuntimeIntegration>,
    nanocom_messages: ResMut<'w, NanocomMessageUiModel>,
    nanocom_outbox: ResMut<'w, NanocomMessageUiOutbox>,
    group_runtime: ResMut<'w, GroupProductionRuntime0104>,
    group_integration: ResMut<'w, GroupRuntimeIntegration0104>,
    group_ui: ResMut<'w, GroupUiModel>,
    pc2pc_offers: ResMut<'w, Pc2pcOfferRuntime0104>,
    pc2pc_ui: ResMut<'w, Pc2pcUiModel0104>,
    pc2pc_prompt: ResMut<'w, WorldPc2pcOfferPrompt>,
    system_messages: ResMut<'w, SystemMessageUiModel>,
}

fn reset_social_on_network_boundary(
    mut resets: MessageReader<NetworkSessionReset>,
    mut owners: SocialSessionOwners,
) {
    if resets.read().count() == 0 {
        return;
    }
    *owners.buddy_ui = BuddyUiModel::default();
    owners.buddy_runtime.reset();
    owners.nanocom_messages.clear();
    owners.nanocom_outbox.clear();
    owners.group_runtime.reset();
    owners.group_integration.reset();
    *owners.group_ui = GroupUiModel::default();
    owners.pc2pc_offers.reset();
    owners.pc2pc_ui.reset();
    owners.pc2pc_prompt.reset(&mut owners.system_messages);
}

#[derive(SystemParam)]
struct InventorySessionOwners<'w> {
    nano_model: ResMut<'w, NanoFreeTuningModel>,
    nano_outbox: ResMut<'w, NanoFreeTuningUiCommandOutbox>,
    nano_bank: ResMut<'w, NanoFreeTuningBank0104>,
    nano_production: ResMut<'w, NanoFreeTuningProductionRuntime>,
    quick_slots: ResMut<'w, QuickSlotUiModel>,
    inventory: ResMut<'w, LocalInventoryRuntime>,
    user_equip_production: ResMut<'w, UserEquipProductionRuntime0104>,
    user_equip_system: ResMut<'w, UserEquipSystemMessageRuntime0104>,
    skill_buffs: ResMut<'w, SkillBuffUiModel>,
    movement_buffs: ResMut<'w, movement_buffs::MovementBuffs>,
    resurrect_ui: ResMut<'w, ResurrectUiModel>,
    resurrect_context: ResMut<'w, ResurrectUiContext>,
    resurrect_outbox: ResMut<'w, ResurrectUiOutbox>,
    system_messages: ResMut<'w, SystemMessageUiModel>,
}

fn reset_inventory_on_network_boundary(
    mut commands: Commands,
    mut resets: MessageReader<NetworkSessionReset>,
    mut owners: InventorySessionOwners,
) {
    if resets.read().count() == 0 {
        return;
    }
    reset_nano_free_tuning_shell(
        &mut commands,
        &mut owners.nano_model,
        &mut owners.nano_outbox,
        &mut owners.nano_bank,
        &mut owners.nano_production,
        true,
    );
    *owners.quick_slots = QuickSlotUiModel::default();
    owners.inventory.reset();
    owners.user_equip_production.reset();
    owners.user_equip_system.reset(&mut owners.system_messages);
    *owners.skill_buffs = SkillBuffUiModel::default();
    *owners.movement_buffs = movement_buffs::MovementBuffs::default();
    reset_resurrect_shell(
        &mut owners.resurrect_ui,
        &mut owners.resurrect_context,
        &mut owners.resurrect_outbox,
    );
}

#[derive(SystemParam)]
struct GuideVendorSessionOwners<'w> {
    guide_ui: ResMut<'w, GuideUiModel>,
    guide_outbox: ResMut<'w, GuideUiOutbox>,
    guide_audio: ResMut<'w, GuideUiAudioOutbox>,
    guide_runtime: ResMut<'w, GuideRuntime>,
    guide_production: ResMut<'w, GuideProductionRuntime>,
    vendor_state: ResMut<'w, VendorUiState>,
    vendor_modal: ResMut<'w, VendorModalState>,
    vendor_projection: ResMut<'w, VendorModeProjection0104>,
    vendor_outbox: ResMut<'w, VendorUiOutbox0104>,
    vendor_production: ResMut<'w, VendorProductionRuntime0104>,
    vendor_system: ResMut<'w, VendorSystemMessageRuntime>,
}

fn reset_guide_vendor_on_network_boundary(
    mut resets: MessageReader<NetworkSessionReset>,
    mut owners: GuideVendorSessionOwners,
) {
    if resets.read().count() == 0 {
        return;
    }
    reset_guide_shell(
        &mut owners.guide_ui,
        &mut owners.guide_outbox,
        &mut owners.guide_audio,
        &mut owners.guide_runtime,
        &mut owners.guide_production,
        // Ingress already cleared old metadata in event order. A fresh login
        // may have arrived in this same frame; retain its entitlement.
        false,
    );
    reset_vendor_shell(
        &mut owners.vendor_state,
        &mut owners.vendor_modal,
        &mut owners.vendor_projection,
        &mut owners.vendor_outbox,
        &mut owners.vendor_production,
        &mut owners.vendor_system,
        true,
    );
}

#[derive(SystemParam)]
struct CommerceSessionOwners<'w> {
    bank_state: ResMut<'w, BankUiState>,
    bank_modal: ResMut<'w, BankModalState>,
    bank_projection: ResMut<'w, BankModeProjection0104>,
    bank_outbox: ResMut<'w, BankUiOutbox0104>,
    bank_production: ResMut<'w, BankProductionRuntime0104>,
    bank_system: ResMut<'w, BankSystemMessageRuntime>,
    combi_runtime: ResMut<'w, CombiProductionRuntime0104>,
    combi_state: ResMut<'w, CombiUiState0104>,
    combi_projection: ResMut<'w, CombiModeProjection0104>,
    combi_outbox: ResMut<'w, CombiUiOutbox0104>,
    combi_shell: ResMut<'w, CombiProductionShell0104>,
    combi_frames: ResMut<'w, CombiNetworkFrameInbox0104>,
    enchant_runtime: ResMut<'w, EnchantProductionRuntime0104>,
    enchant_projection: ResMut<'w, EnchantModeProjection0104>,
    enchant_outbox: ResMut<'w, EnchantUiOutbox0104>,
    enchant_shell: ResMut<'w, EnchantProductionShell0104>,
    enchant_frames: ResMut<'w, EnchantNetworkFrameInbox0104>,
    system_messages: ResMut<'w, SystemMessageUiModel>,
}

fn reset_commerce_on_network_boundary(
    mut commands: Commands,
    mut resets: MessageReader<NetworkSessionReset>,
    mut owners: CommerceSessionOwners,
) {
    if resets.read().count() == 0 {
        return;
    }
    reset_bank_shell(
        &mut owners.bank_state,
        &mut owners.bank_modal,
        &mut owners.bank_projection,
        &mut owners.bank_outbox,
        &mut owners.bank_production,
        &mut owners.bank_system,
    );
    reset_combi_shell_0104(
        &mut commands,
        &mut owners.combi_runtime,
        &mut owners.combi_state,
        &mut owners.combi_projection,
        &mut owners.combi_outbox,
        &mut owners.combi_shell,
        &mut owners.combi_frames,
        &mut owners.system_messages,
    );
    reset_enchant_shell_0104(
        &mut owners.enchant_runtime,
        &mut owners.enchant_projection,
        &mut owners.enchant_outbox,
        &mut owners.enchant_shell,
        &mut owners.enchant_frames,
        &mut owners.system_messages,
    );
}

#[derive(SystemParam)]
struct OverlaySessionOwners<'w> {
    reward_notices: ResMut<'w, ffone_client::gameplay_ui::rewards::RewardNotices>,
    upsell_ui: ResMut<'w, UpsellUiModel>,
    upsell_outbox: ResMut<'w, UpsellUiOutbox>,
    upsell_audio: ResMut<'w, UpsellUiAudioOutbox>,
    quit_menu: ResMut<'w, QuitMenuUiModel>,
    quit_outbox: ResMut<'w, QuitMenuUiOutbox>,
    quit_runtime: ResMut<'w, QuitMenuRuntime>,
}

fn reset_overlays_on_network_boundary(
    mut resets: MessageReader<NetworkSessionReset>,
    mut owners: OverlaySessionOwners,
) {
    if resets.read().count() == 0 {
        return;
    }
    owners.reward_notices.clear();
    reset_upsell_shell(
        &mut owners.upsell_ui,
        &mut owners.upsell_outbox,
        &mut owners.upsell_audio,
    );
    reset_quit_menu_shell(
        &mut owners.quit_menu,
        &mut owners.quit_outbox,
        &mut owners.quit_runtime,
    );
}

fn finish_network_boundary_reset(
    mut resets: MessageReader<NetworkSessionReset>,
    mut mission_ui: ResMut<MissionUiModel>,
    mut world_mission: ResMut<WorldMissionRuntime>,
    mut system_messages: ResMut<SystemMessageUiModel>,
) {
    if resets.read().count() == 0 {
        return;
    }
    if mission_ui.cancel_warp_away() {
        system_messages.set_focus_out(false);
    }
    system_messages.clear();
    *mission_ui = MissionUiModel::default();
    world_mission.clear();
}

#[cfg(test)]
mod tests;
