//! Opt-in full-client UI interaction; uses replayed authority, not a live shard.
use super::*;
use ffone_client::pc2pc_ui::*;
use ffone_client::inventory_runtime::InventoryRuntime0104;

#[derive(Default, Resource)]
struct Probe { opened: bool, registered: bool, removed: bool, cash: bool, submitted: bool }

pub(super) fn install(app: &mut App) {
    app.init_resource::<Probe>()
        .add_systems(Update, discard_offline_events.before(poll_network))
        .add_systems(Last, drive.before(measure));
}
fn discard_offline_events(bridge: Res<NetworkBridge>) { let _ = bridge.drain(); }
fn drive(world: &mut World) {
    if *world.resource::<State<ClientState>>().get() != ClientState::World || world.resource::<GameplayLoadingState>().visible { return; }
    world.resource_mut::<Capture>().samples.clear();
    if world.resource::<ButtonInput<MouseButton>>().just_pressed(MouseButton::Left) {
        let controls = world.query::<(&Pc2pcUiElement, &Interaction)>().iter(world)
            .filter(|(_, interaction)| **interaction != Interaction::None)
            .map(|(element, interaction)| format!("{element:?}={interaction:?}")).collect::<Vec<_>>();
        println!("bug039-ui: click controls={controls:?}, phase={:?}, modal={:?}, status={}",
            world.resource::<Pc2pcUiModel0104>().state.phase,
            world.resource::<Pc2pcModalState>(), world.resource::<RuntimeStatus>().message);
    }
    if !world.resource::<Probe>().opened {
        let mut load = ffone_protocol::PcLoadData0104::zeroed();
        let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET;
        load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&7i16.to_le_bytes());
        load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&90i16.to_le_bytes());
        load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&12i32.to_le_bytes());
        let inventory = InventoryRuntime0104::from_pc_load(1, &load);
        let identity = Pc2pcSessionIdentity0104::new(Pc2pcPair0104::new(1, 2).unwrap(), 1, Pc2pcOfferDirection0104::Outgoing).unwrap();
        let snapshot = Pc2pcAuthoritativeSnapshot0104::from_accepted_trade(identity,
            Pc2pcParticipantNames0104::new("Trade From", "Trade To").unwrap(), 500, &inventory).unwrap();
        world.resource_scope(|world, mut model: Mut<Pc2pcUiModel0104>| {
            model.begin_session(snapshot, world.resource::<TutorialMissionContent>(), &Pc2pcFailClosedEquipEligibility);
        });
        world.resource_mut::<Probe>().opened = true;
        println!("bug039-ui: opened; click inventory item, then remove it, add Taros, Submit, and Close");
        return;
    }
    let pending = world.resource::<Pc2pcUiModel0104>().state.pending.clone();
    let outcome = match pending {
        Some(Pc2pcPendingRequest0104::RegisterItem(intent)) => {
            assert_eq!(world.resource::<Pc2pcUiModel0104>().snapshot.as_ref().unwrap().base_inventory()[0].option, 12);
            world.resource_mut::<Probe>().registered = true;
            println!("bug039-ui: inventory click queued register; owned inventory unchanged");
            Some(Pc2pcServerOutcome0104::RegisterItemSuccess { envelope: intent.envelope, trade_item: intent.item,
                inventory_item: Pc2pcTradeItem0104 { option: 0, ..intent.item } })
        }
        Some(Pc2pcPendingRequest0104::UnregisterItem(intent)) => {
            world.resource_mut::<Probe>().removed = true;
            println!("bug039-ui: local offer click queued unregister");
            Some(Pc2pcServerOutcome0104::UnregisterItemSuccess { envelope: intent.envelope, trade_item: intent.item,
                inventory_item: Pc2pcTradeItem0104 { option: 12, ..intent.item } })
        }
        Some(Pc2pcPendingRequest0104::RegisterCash(intent)) => {
            assert_eq!(intent.taros, 25);
            world.resource_mut::<Probe>().cash = true;
            println!("bug039-ui: typed 25 Taros and submitted");
            Some(Pc2pcServerOutcome0104::RegisterCashSuccess { envelope: intent.envelope, taros: intent.taros })
        }
        Some(Pc2pcPendingRequest0104::Confirm(intent)) => {
            world.resource_mut::<Probe>().submitted = true;
            println!("bug039-ui: Submit click queued confirmation");
            Some(Pc2pcServerOutcome0104::Confirmed { envelope: intent.envelope })
        }
        _ => None,
    };
    if let Some(outcome) = outcome {
        world.resource_scope(|world, mut model: Mut<Pc2pcUiModel0104>| {
            model.apply_server_outcome(outcome, Pc2pcBackendCapabilities { numeric_popup_backend: true, ..default() },
                world.resource::<TutorialMissionContent>(), &Pc2pcFailClosedEquipEligibility).unwrap();
        });
    }
    if world.resource::<Pc2pcUiModel0104>().state.phase == Pc2pcLifecyclePhase::Cancelled {
        let p = world.resource::<Probe>();
        assert!(p.registered && p.removed && p.cash && p.submitted, "exercise every UI step before closing");
        assert_eq!(world.resource::<Pc2pcUiModel0104>().snapshot.as_ref().unwrap().base_inventory()[0].option, 12);
        let output = world.resource::<Capture>().output.join("pc2pc-ui-pass.txt");
        fs::write(output, "PASS full offline client; real pointer register/unregister, keyboard Taros=25, Submit, Close; owned inventory unchanged. Live two-client exchange not established.").unwrap();
        world.resource_mut::<Messages<AppExit>>().write(AppExit::Success);
    }
}
