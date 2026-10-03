//! Offline mission mail regression through the real production mode entry.
//! FFONE_PERF_EMAIL=1 with FFONE_PERF_OUTPUT enables it.
use super::*;
use ffone_client::email_ui::{
    EMAIL_UI_OPEN_SECONDS, EmailFolder, EmailUiButton, EmailUiButtonKind, open_email_ui,
    switch_email_folder,
};

#[derive(Default, Resource)]
struct EmailProbe {
    frame: u32,
    invitation: Option<i32>,
    reopen_count: u32,
    captured: [bool; 2],
    tab_captured: [bool; 6],
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<EmailProbe>()
        .add_systems(
            Update,
            drive
                .after(consume_world_gameplay_ui_outbox)
                .after(drive_email_production_post_interaction_0104)
                .before(EmailUiSet::Bind),
        )
        .add_systems(Last, verify.before(measure));
}

#[allow(clippy::too_many_arguments)]
fn drive(
    capture: Res<Capture>,
    mut probe: ResMut<EmailProbe>,
    content: Res<TutorialMissionContent>,
    mut guide: ResMut<GuideRuntime>,
    mut mission: ResMut<WorldMissionRuntime>,
    mut inventory: ResMut<LocalInventoryRuntime>,
    mut outbox: ResMut<GameplayUiOutbox>,
    mut model: ResMut<EmailUiModel>,
    status: Res<RuntimeStatus>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut audio: ResMut<EmailUiAudioOutbox>,
    mut buttons: Query<(&EmailUiButton, &mut Interaction)>,
    mut actions: ResMut<EmailUiOutbox>,
) {
    if capture.ready.is_none() || capture.samples.is_empty() {
        return;
    }
    if env::var_os("FFONE_PERF_EMAIL_TABS").is_some() {
        // Isolate presentation from the offline inventory/session gate. The
        // ordinary mission-mail fixture below still uses the NanoCom entry.
        if !model.visible {
            open_email_ui(&mut model, &mut actions, Vec::new(), 0);
        }
        model.opening_elapsed_seconds = EMAIL_UI_OPEN_SECONDS;
        model.right_opening_elapsed_seconds = EMAIL_UI_OPEN_SECONDS;
        probe.frame += 1;
        let phase = ((probe.frame - 1) / 40).min(5) as usize;
        let folder = if phase < 3 {
            EmailFolder::Guide
        } else {
            EmailFolder::Player
        };
        model.send_in_flight = false;
        if model.folder != folder {
            assert!(switch_email_folder(
                folder,
                &mut model,
                &mut transport,
                &mut audio
            ));
        }
        model.send_in_flight = false;
        for (button, mut interaction) in &mut buttons {
            *interaction = if (phase % 3 == 1 && button.kind == EmailUiButtonKind::PlayerTab)
                || (phase % 3 == 2 && button.kind == EmailUiButtonKind::GuideTab)
            {
                Interaction::Hovered
            } else {
                Interaction::None
            };
        }
        return;
    }
    // The offline bridge can reset modal owners after rejecting periodic
    // gameplay traffic. Replay only fixture authority and enter through the
    // same production action after each such reset.
    if !model.visible {
        probe.reopen_count += 1;
        if probe.reopen_count <= 3 {
            eprintln!(
                "EMAIL fixture entry {}: {}",
                probe.reopen_count, status.message
            );
        }
        let mut load = ffone_protocol::PcLoadData0104::zeroed();
        let mentor = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
        load.as_bytes_mut()[mentor..mentor + 2].copy_from_slice(&5_i16.to_le_bytes());
        let task = ffone_protocol::PcLoadData0104::RUNNING_QUESTS_OFFSET
            + ffone_protocol::RunningQuest0104::SIZE;
        load.as_bytes_mut()[task..task + 4].copy_from_slice(&198_i32.to_le_bytes());
        // The two reported character inventories previously blocked the whole
        // mailbox when their vehicle or membership-card artwork was absent.
        for (slot, (item_type, item_id)) in [
            (10_i16, 105_i16),
            (10, 42),
            (7, 5),
            (7, 29),
            (7, 32),
            (7, 30),
        ]
        .into_iter()
        .enumerate()
        {
            let offset =
                ffone_protocol::PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE;
            let bytes = &mut load.as_bytes_mut()[offset..offset + ItemBase0104::SIZE];
            bytes[..2].copy_from_slice(&item_type.to_le_bytes());
            bytes[2..4].copy_from_slice(&item_id.to_le_bytes());
            bytes[4..8].copy_from_slice(&1_i32.to_le_bytes());
        }
        guide.load_pc_state(&load);
        mission.seed(&load, &content).unwrap();
        inventory.seed(1, &load);
        outbox.push(GameplayUiAction::OpenEmailFromNanocom);
        return;
    }
    probe.frame += 1;
    match probe.frame {
        // Reapply the requested selection after offline session resets.
        // The transport unit test independently verifies unchanged refreshes
        // preserve selection without this presentation-fixture intervention.
        90.. => {
            let index = model
                .guide_messages
                .iter()
                .position(|message| message.mode == 2)
                .expect("the production mailbox must include an available invitation");
            probe.invitation = Some(model.guide_messages[index].mission_task_id);
            model.guide_page = index / 5;
            model.selected_row = Some(index % 5);
        }
        _ => {}
    }
}

fn verify(
    mut commands: Commands,
    capture: Res<Capture>,
    mut probe: ResMut<EmailProbe>,
    model: Res<EmailUiModel>,
    localization: Res<Localization>,
    language: Res<Language>,
    nodes: Query<(
        &bevy::ui::ComputedStackIndex,
        &ImageNode,
        Option<&EmailUiButton>,
    )>,
    asset_server: Res<AssetServer>,
) {
    if env::var_os("FFONE_PERF_EMAIL_TABS").is_some() {
        if model.visible && probe.frame >= 20 && (probe.frame - 20) % 40 == 0 {
            let phase = ((probe.frame - 20) / 40) as usize;
            if phase < 6 && !probe.tab_captured[phase] {
                let stack = |kind| {
                    nodes
                        .iter()
                        .find(|(_, _, button)| button.is_some_and(|button| button.kind == kind))
                        .unwrap()
                        .0
                        .0
                };
                let rim = nodes
                    .iter()
                    .find(|(_, image, _)| {
                        asset_server
                            .get_path(image.image.id())
                            .is_some_and(|path| path.path().ends_with("email/list.png"))
                    })
                    .unwrap()
                    .0
                    .0;
                assert!(stack(EmailUiButtonKind::PlayerTab) < rim);
                let guide = stack(EmailUiButtonKind::GuideTab);
                if phase == 3 || phase == 4 {
                    assert!(
                        guide < rim,
                        "inactive guide tab must stay under the list rim"
                    );
                } else {
                    assert!(rim < guide, "selected or hovered guide tab must be visible");
                }
                assert_eq!(
                    model.folder,
                    if phase < 3 {
                        EmailFolder::Guide
                    } else {
                        EmailFolder::Player
                    }
                );
                probe.tab_captured[phase] = true;
                commands.spawn(Screenshot::primary_window()).observe(
                    bevy::render::view::screenshot::save_to_disk(
                        capture.output.join(format!("tabs-{phase}.png")),
                    ),
                );
            }
        }
        return;
    }
    if !model.visible || !matches!(probe.frame, 60 | 150) {
        return;
    }
    assert!(model.visible && model.folder == EmailFolder::Guide);
    let selected = model.selected_guide().expect("selected production letter");
    let name = if probe.frame == 60 {
        assert_eq!(
            (
                selected.mode,
                selected.mission_task_id,
                selected.sender_npc_id
            ),
            (1, 198, 728)
        );
        "npc-letter"
    } else {
        assert_eq!(selected.mode, 2);
        assert_eq!(Some(selected.mission_task_id), probe.invitation);
        "mentor-invitation"
    };
    let capture_index = usize::from(probe.frame == 150);
    probe.captured[capture_index] = true;
    commands.spawn(Screenshot::primary_window()).observe(
        bevy::render::view::screenshot::save_to_disk(capture.output.join(format!("{name}.png"))),
    );
    fs::write(
        capture.output.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "task": selected.mission_task_id,
            "mode": selected.mode,
            "count": model.guide_messages.len(),
            "offlineReentries": probe.reopen_count,
            "sender": localization.text(&language, &selected.localized_sender()),
            "subject": localization.text(&language, &selected.localized_subject()),
            "body": localization.text(&language, &selected.localized_content()),
        }))
        .unwrap(),
    )
    .unwrap();
}

pub(super) fn assert_complete(world: &World) {
    if let Some(probe) = world.get_resource::<EmailProbe>() {
        if env::var_os("FFONE_PERF_EMAIL_TABS").is_some() {
            assert!(
                probe.tab_captured.iter().all(|captured| *captured),
                "email tab fixture incomplete"
            );
            return;
        }
        assert!(
            probe.captured.iter().all(|captured| *captured),
            "email fixture incomplete after {} visible frames and {} entry attempts: {}",
            probe.frame,
            probe.reopen_count,
            world.resource::<RuntimeStatus>().message
        );
    }
}
