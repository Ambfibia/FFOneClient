use super::*;

pub(super) fn sync_service_inventory_audio(
    state: Res<State<ClientState>>,
    bank: Res<BankUiState>,
    enchant: Res<enchant::EnchantProductionShell0104>,
    trade: Res<ffone_client::pc2pc_ui::Pc2pcUiModel0104>,
    store: Res<ffone_client::user_store_ui::UserStoreUiState0104>,
    vendor: Res<VendorUiState>,
    mut audio: ResMut<GameplayAudioRuntime>,
) {
    let active = *state.get() == ClientState::World
        && (bank.phase != ffone_client::bank_ui::BankLifecyclePhase::Hidden
            || enchant.ui_mode_audio_active
            || trade.state.phase.renders_shell()
            || store.active
            || vendor.phase != ffone_client::vendor_ui::VendorLifecyclePhase::Hidden);
    audio.set_service_inventory_audio_active(active);
}

pub(super) fn sync_item_popup_audio(
    bank: Res<ffone_client::bank_ui::BankItemPopupState>,
    vendor: Res<ffone_client::vendor_ui::VendorItemPopupState>,
    inventory: Res<ffone_client::user_equip_ui::UserEquipItemPopupState>,
    mut audio: ResMut<GameplayAudioRuntime>,
    mut previous: Local<[bool; 3]>,
) {
    let current = [
        bank.is_open(),
        vendor.is_open(),
        inventory.selected().is_some(),
    ];
    for (index, open) in current.into_iter().enumerate() {
        if open != previous[index] && (!open || index != 2) {
            // UserEquip already emits ShowPopup's Open_Screen at selection.
            audio.queue_gameplay_ui_sound(if open { "Open_Screen" } else { "Close_Screen" });
        }
    }
    *previous = current;
}

pub(super) fn sync_world_map_mode_audio(
    mut commands: Commands,
    state: Res<State<ClientState>>,
    map: Res<ffone_client::world_map::WorldMapPresentation>,
    production: Res<world_map::WorldMapProductionRuntime>,
    catalog: Res<ffone_client::semantic_audio::NativeAudioCatalog>,
    assets: Res<AssetServer>,
    mut audio: ResMut<GameplayAudioRuntime>,
    mut was_open: Local<bool>,
    mut pan_source: Local<Option<Entity>>,
) {
    let open = map.model.phase() != ffone_client::world_map::WorldMapPhase::Closed;
    if *state.get() == ClientState::World && open != *was_open {
        audio.queue_gameplay_ui_sound(if open { "Open_Screen" } else { "Close_Screen" });
    }
    *was_open = open;
    let pan = *state.get() == ClientState::World && open && production.pan_audio_active;
    if !pan {
        if let Some(entity) = pan_source.take() {
            commands.entity(entity).despawn();
        }
    } else if pan_source.is_none() {
        use ffone_client::{
            audio_channel::GameplayAudioChannel, semantic_audio::NativeAudioCategory,
        };
        let candidates = catalog
            .by_true_name("Character_Rotate_Left")
            .into_iter()
            .filter(|clip| clip.category == NativeAudioCategory::Sfx)
            .collect::<Vec<_>>();
        if let [clip] = candidates.as_slice() {
            // WorldMapMode.Update uses PlaySoundAtPoint with raw SFX gain,
            // unlike the 0.7 multiplier of ordinary SoundUtil UI calls.
            *pan_source = Some(
                commands
                    .spawn((
                        Name::new("WorldMap pan SFX"),
                        GameplayAudioChannel::new(NativeAudioCategory::Sfx, 1.0),
                        AudioPlayer::new(assets.load(clip.path.clone())),
                        PlaybackSettings::LOOP,
                    ))
                    .id(),
            );
        }
    }
}

pub(super) fn consume_gameplay_ui_audio_outbox(
    mut outbox: ResMut<GameplayUiAudioOutbox>,
    mut audio: ResMut<GameplayAudioRuntime>,
    mut runtime: ResMut<RuntimeStatus>,
    mission: Res<MissionUiModel>,
    mut previous_npc: Local<Option<i32>>,
) {
    let outgoing = runtime.chat.outgoing_audio_pending;
    if outgoing != 0 {
        runtime.chat.outgoing_audio_pending = 0;
    }
    let npc = mission
        .enabled
        .then(|| mission.npc_subtarget_actor_id())
        .flatten();
    for cue in gameplay_event_audio(outgoing, npc, &mut previous_npc) {
        audio.queue_gameplay_ui_sound(cue);
    }
    if !runtime.chat.pending_social_sfx.is_empty() {
        for cue in runtime.chat.pending_social_sfx.drain(..) {
            audio.queue_gameplay_ui_sound(cue);
        }
    }
    for cue in outbox.drain() {
        if let Some(true_name) = cue.true_name() {
            audio.queue_gameplay_ui_sound(true_name);
        } else {
            audio.queue_legacy_button_sound();
        }
    }
}

pub(super) fn consume_system_message_ui_audio_outbox(
    mut outbox: ResMut<SystemMessageUiAudioOutbox>,
    mut audio: ResMut<GameplayAudioRuntime>,
) {
    for cue in outbox.drain() {
        audio.queue_gameplay_ui_sound(cue.true_name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_sfx_map_pan_keeps_one_loop_and_stops_on_release_or_mode_exit() {
        use bevy::asset::AssetPlugin;
        use ffone_client::{audio_channel::GameplayAudioChannel, semantic_audio::NativeAudioCategory, world_map::*};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let catalog = NativeAudioCatalog::open(&root, false).unwrap();
        let mut map = WorldMapPresentation::default();
        super::super::world_map::open_world_map_for_ready_player(
            &mut map,
            &WorldMapPresentationAssetStatus::Ready,
            Some(WorldMapPlayer::new(
                WorldMapPoint::new(6320., 0., 1871.),
                0.,
            )),
            Some(0),
            &RaceRankCatalog::default(),
        )
        .unwrap();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: root.to_string_lossy().into(),
                ..default()
            },
        ))
        .init_asset::<AudioSource>()
        .insert_resource(State::new(ClientState::World))
        .insert_resource(catalog)
        .insert_resource(map)
        .init_resource::<world_map::WorldMapProductionRuntime>()
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(Update, sync_world_map_mode_audio);
        app.world_mut()
            .resource_mut::<world_map::WorldMapProductionRuntime>()
            .pan_audio_active = true;
        app.update();
        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<AudioPlayer>>()
            .single(app.world())
            .unwrap();
        let channel = app.world().get::<GameplayAudioChannel>(entity).unwrap();
        assert_eq!(channel.base_gain, 1.0);
        assert_eq!(channel.category, NativeAudioCategory::Sfx);
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<AudioPlayer>>()
                .single(app.world())
                .unwrap(),
            entity
        );
        app.world_mut()
            .resource_mut::<world_map::WorldMapProductionRuntime>()
            .pan_audio_active = false;
        app.update();
        assert!(app.world().get_entity(entity).is_err());
        app.world_mut()
            .resource_mut::<world_map::WorldMapProductionRuntime>()
            .pan_audio_active = true;
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            1
        );
        app.world_mut()
            .insert_resource(State::new(ClientState::Login));
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&AudioPlayer>()
                .iter(app.world())
                .count(),
            0
        );
    }
}
