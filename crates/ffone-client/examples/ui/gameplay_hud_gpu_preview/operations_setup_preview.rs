use super::*;

pub(super) fn drive_reward_preview(
    localization: Res<Localization>,
    language: Res<Language>,
    mut notices: ResMut<ffone_client::gameplay_ui::rewards::RewardNotices>,
    content: Res<TutorialMissionContent>,
    mut overlay: ResMut<TutorialOverlayUiModel>,
    mut model: ResMut<GameplayUiModel>,
) {
    let Ok(mode) = env::var("FFONE_REWARD_PREVIEW") else {
        return;
    };
    if mode.starts_with("coco") {
        notices.clear();
        notices
            .receive_shiny(
                &ffone_protocol::wire_0104::ShinyPickupSuccess0104 {
                    skill_id: 183,
                    cstb: 0,
                },
                &content,
                &localization,
                &language,
            )
            .expect("Coco skill notice");
        notices.advance(if mode.ends_with("fade") { 3.5 } else { 2.0 });
        overlay.hide_and_clear();
        model.chat.active = false;
        return;
    }
    use ffone_client::world_mission_runtime::{WorldMissionRuntime, WorldMissionServerEvent0104};
    use ffone_protocol::{
        ItemBase0104, ItemReward0104, PcTaskStartSuccess0104, RewardItemReply0104,
    };
    let mut mission = WorldMissionRuntime::default();
    mission
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: 2248,
                remaining_time: 0,
            }),
            &content,
        )
        .unwrap();
    let mut packet = RewardItemReply0104 {
        candy: 100,
        fusion_matter: 200,
        nano_battery: 15,
        weapon_battery: 30,
        pack_padding: [0; 3],
        fatigue: 0,
        fatigue_level: 0,
        npc_type_id: 2676,
        task_id: 2248,
        items: vec![ItemReward0104 {
            inventory_location: 1,
            slot: 0,
            item: ItemBase0104 {
                item_type: 9,
                item_id: 1,
                option: 1,
                time_limit: 0,
            },
        }],
    };
    if mode.starts_with("found") {
        packet.items.push(ItemReward0104 {
            inventory_location: 2,
            slot: 0,
            item: ItemBase0104 {
                item_type: 8,
                item_id: 537,
                option: 1,
                time_limit: 0,
            },
        });
    }
    notices.clear();
    notices.receive(&packet, &content, &mission, 10, 20);
    if mode.starts_with("status") {
        notices.clear();
        notices.receive_currencies(12300, 4500, 12345, 4567);
        notices.set_inventory_full(Some(true));
        for _ in 0..25 {
            notices.advance(0.11);
        }
        if mode.ends_with("fade") {
            notices.advance(0.7);
        }
    }
    if !mode.starts_with("status") {
        notices.advance(if mode.ends_with("fade") { 2.5 } else { 1.0 });
    }
    overlay.hide_and_clear();
    model.chat.input_enabled = true;
    model.chat.active = mode.ends_with("chat");
}

pub(super) fn setup_preview(
    mut commands: Commands,
    player_look: Res<PreviewPlayerLook>,
    mut model: ResMut<GameplayUiModel>,
    mut buddy_ui: ResMut<BuddyUiModel>,
    mut tutorial_overlay: ResMut<TutorialOverlayUiModel>,
    mut portraits: ResMut<CharacterSelectionPortraitsModel>,
    mut gameplay_portrait: ResMut<GameplayPlayerPortraitModel>,
    mut nanocom_messages: ResMut<TutorialNanocomMessageQueue>,
    mut production_nanocom_messages: ResMut<NanocomMessageUiModel>,
    mut mission_ui: ResMut<MissionUiModel>,
    mut nano_wheel: ResMut<NanoWheelTransientUi>,
    mut npc_bubbles: ResMut<NpcBarkerBubbleRuntime>,
    mut player_bubbles: ResMut<PlayerFreeChatBubbleRuntime>,
    (mission_content, nano_catalog): (
        Res<TutorialMissionContent>,
        Res<GameplayNanoPortraitCatalog>,
    ),
    localization: Res<Localization>,
    language: Res<Language>,
) {
    *model = GameplayUiModel::retrobution_reference_frame();
    let scene_line = |line| {
        let fallback = mission_content
            .scene_text(2, line)
            .expect("Retrobution tutorial scene line");
        localization.text(&language, &localized_tutorial_scene_text(2, line, fallback))
    };
    model.chat.lines = [2, 3, 4, 7, 8]
        .into_iter()
        .map(|line| ChatLineUi::normal(scene_line(line)))
        .chain(std::iter::once(ChatLineUi::tutorial(localization.text(
            &language,
            &localized_tutorial_instruction("Move your mouse to the right and find the marker."),
        ))))
        .collect();
    if let Ok(input) = env::var("FFONE_CHAT_INPUT_PREVIEW") {
        model.chat.input_enabled = true;
        model.chat.active = true;
        model.chat.input = input;
        if env::var_os("FFONE_TEXT_SELECTION_PREVIEW").is_some() {
            model.chat.edit.anchor = 0;
            model.chat.edit.cursor = model.chat.input.chars().count();
        }
    }
    let quick_chat_preview = env::var("FFONE_QUICK_CHAT_PREVIEW").ok();
    if let Some(preview) = quick_chat_preview.as_deref() {
        model.chat.input_enabled = true;
        model.chat.active = true;
        match preview {
            "menu" => model.chat.quick_menu.toggle(QuickChatMenuMode::MenuChat),
            "emotes" | "emotes-dance" => {
                model.chat.quick_menu.toggle(QuickChatMenuMode::Emotes);
                if preview == "emotes-dance" {
                    let dance = *model
                        .chat
                        .quick_menu
                        .visible_items(0)
                        .into_iter()
                        .find(|item| item.id == 16)
                        .expect("clean Dance emote container");
                    model.chat.quick_menu.select(dance);
                }
            }
            value => panic!(
                "FFONE_QUICK_CHAT_PREVIEW must be menu, emotes, or emotes-dance; got {value:?}"
            ),
        }
    }
    if quick_chat_preview.is_some() {
        *tutorial_overlay = TutorialOverlayUiModel::default();
    } else {
        *tutorial_overlay = TutorialOverlayUiModel::retrobution_reference_frame();
        tutorial_overlay.tutorial.instruction = Some(
            localization
                .text(
                    &language,
                    &localized_tutorial_instruction(
                        "Move your mouse to the right and find the marker.",
                    ),
                )
                .to_uppercase(),
        );
    }
    portraits
        .set_look(0, player_look.0.clone())
        .expect("set dedicated gameplay portrait look");
    gameplay_portrait.visible = true;
    gameplay_portrait.slot = Some(0);
    // Exercise the two status icons and, by default, a half-filled current-
    // level FM bar. Acceptance can override this with FFONE_FM_PERCENT.
    model.player.free_chat = true;
    model.player.allow_player_interaction = false;
    model.minimap.max_fusion_matter = 220;
    let fusion_percent = env::var("FFONE_FM_PERCENT")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(50.0)
        .clamp(0.0, 100.0);
    model.minimap.fusion_matter =
        (model.minimap.max_fusion_matter as f32 * fusion_percent / 100.0).round() as i32;
    model.minimap.markers = vec![
        MinimapMarkerSample {
            icon: MinimapMarkerIcon::New,
            left: 42.0,
            top: 53.0,
            width: 16.0,
            height: 16.0,
        },
        MinimapMarkerSample {
            icon: MinimapMarkerIcon::Advance,
            left: 92.0,
            top: 83.0,
            width: 16.0,
            height: 16.0,
        },
        MinimapMarkerSample {
            icon: MinimapMarkerIcon::ShowNpc,
            left: 69.0,
            top: 40.0,
            width: 16.0,
            height: 16.0,
        },
        MinimapMarkerSample {
            icon: MinimapMarkerIcon::Fusion,
            left: 55.0,
            top: 100.0,
            width: 16.0,
            height: 16.0,
        },
        MinimapMarkerSample {
            icon: MinimapMarkerIcon::Mob,
            left: 105.0,
            top: 55.0,
            width: 16.0,
            height: 16.0,
        },
    ];
    model.nanos = [
        NanoSlotUi {
            nano_id: Some(1),
            name: localization.text(
                &language,
                &LocalizedText::new("content.nano.1.name", "Buttercup"),
            ),
            skill_id: Some(1),
            style: Some(1),
            model_path: Some("characters/nanos/nano_buttercup/nano_buttercup.glb".to_owned()),
            nano_icon_number: Some(34),
            skill_icon_number: Some(10),
            active_skill: true,
            stamina_fraction: 1.0,
            active: true,
        },
        NanoSlotUi {
            nano_id: Some(2),
            name: localization.text(
                &language,
                &LocalizedText::new("content.nano.2.name", "Numbuh Two"),
            ),
            skill_id: Some(4),
            style: Some(0),
            model_path: Some("characters/nanos/nano_numbuhtwo/nano_numbuhtwo.glb".to_owned()),
            nano_icon_number: Some(28),
            skill_icon_number: Some(14),
            active_skill: false,
            stamina_fraction: 0.55,
            active: false,
        },
        NanoSlotUi {
            nano_id: Some(3),
            name: localization.text(
                &language,
                &LocalizedText::new("content.nano.3.name", "Eddy"),
            ),
            skill_id: Some(7),
            style: Some(2),
            model_path: Some("characters/nanos/nano_eddy/nano_eddy.glb".to_owned()),
            nano_icon_number: Some(20),
            skill_icon_number: Some(3),
            active_skill: true,
            stamina_fraction: 0.18,
            active: false,
        },
    ];
    if let Ok(id) = env::var("FFONE_HUD_NANO_ID") {
        let id: i16 = id.parse().expect("numeric Nano ID");
        let nano = mission_content.gameplay_nano(id).expect("published Nano");
        let skill_id =
            mission_content.journal_nano(i32::from(id)).unwrap().skills[0].skill_id as i16;
        let skill = mission_content.gameplay_skill(skill_id).unwrap();
        model.nanos[0].nano_id = Some(id);
        model.nanos[0].name = localization.text(
            &language,
            &LocalizedText::new(format!("content.nano.{id}.name"), &nano.name),
        );
        model.nanos[0].model_path = nano_catalog.model_path(id).map(str::to_owned);
        model.nanos[0].nano_icon_number = Some(nano.icon_number);
        model.nanos[0].skill_id = Some(skill_id);
        model.nanos[0].skill_icon_number = Some(skill.icon_number);
        model.nanos[0].active_skill = skill.active;
        model.nanos[0].style = Some(nano.style);
    }
    // Exercise both transient source-only Nano Wheel layers: the affinity-
    // colored gumball shell and a coarse active-skill cooldown wedge.
    assert!(nano_wheel.set_gumball_effect(1));
    nano_wheel.slots[2].active_skill_cooldown_remaining = Some(0.5);
    model.nano_battery = 75;
    model.weapon_battery = 600;
    let objective_preview = env::var_os("FFONE_CURRENT_OBJECTIVE_PREVIEW").is_some();
    if objective_preview {
        model.current_objective = CurrentObjectiveUi {
            visible: true,
            task_id: Some(2248),
            title: "Transmitter Critters".to_owned(),
            body: "Defeat the Oil Ogre.".to_owned(),
            remaining_time_seconds: None,
            enemies: Vec::new(),
            quest_items: vec![CurrentObjectiveProgressUi {
                content_id: 537,
                name: "Transmitter".to_owned(),
                complete: 0,
                needed: 1,
            }],
        };
        tutorial_overlay.hide_and_clear();
    }
    let mission_preview = env::var("FFONE_MISSION_UI_PREVIEW").ok();
    let barker_preview = env::var_os("FFONE_NPC_BARKER_PREVIEW").is_some();
    let freechat_preview = env::var("FFONE_FREECHAT_PREVIEW").ok();
    let chat_tab_preview = env::var("FFONE_CHAT_TAB_PREVIEW").ok();
    let computress_nanocom_preview = env::var_os("FFONE_COMPUTRESS_NANOCOM_PREVIEW").is_some();
    if computress_nanocom_preview {
        production_nanocom_messages.set_ui_scale(model.ui_scale);
        production_nanocom_messages.enqueue_type_9_localized(
            LocalizedText::new("content.npc.730.name", "Computress"),
            LocalizedText::new(
                "content.tabledata.guide.guide_string.19.sz_string",
                "Welcome back. Please check your email to learn about an important mission from me.",
            ),
            "ui/en/gameplay/guide/compu_icon.png",
            Some("Computress"),
        );
        assert!(production_nanocom_messages.tick(0.5).is_none());
        while production_nanocom_messages.pop_sound().is_some() {}
    }
    if mission_preview.is_none()
        && !objective_preview
        && !barker_preview
        && freechat_preview.is_none()
        && chat_tab_preview.is_none()
        && !computress_nanocom_preview
    {
        nanocom_messages.enqueue_type_9_numbuh_two(
            LocalizedText::new("content.npc.2671.name", "Numbuh Two"),
            localized_tutorial_literal(
                "I need some help with a sooper important mission! Report to me right away!",
            ),
        );
    }
    if barker_preview {
        tutorial_overlay.hide_and_clear();
        let player = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, 5.0),
                GlobalTransform::IDENTITY,
                LegacyAvatarActionState::default(),
            ))
            .id();
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 3.0, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            LegacyOrbitCamera::new(player),
        ));
        let npc = commands
            .spawn((
                Transform::IDENTITY,
                GlobalTransform::IDENTITY,
                NetworkNpcAppearance0104(NpcAppearance0104 {
                    npc_id: 643,
                    npc_type: 643,
                    hp: 439,
                    condition_bit_flag: 0,
                    position: [0; 3],
                    angle: 0,
                    barker_type: 0,
                }),
            ))
            .id();
        npc_bubbles.request_greeting(
            npc,
            mission_content
                .gameplay_npc(643)
                .expect("production Edd guide shop NPC"),
        );
    }
    if let Some(freechat_preview) = freechat_preview {
        tutorial_overlay.hide_and_clear();
        model.chat.input_enabled = true;
        model.chat.active = !freechat_preview.eq_ignore_ascii_case("inactive");
        model.chat.group_available = true;
        model.chat.input.clear();
        model.chat.lines = vec![ChatLineUi::npc(
            "Computress: Welcome back. Please check your email to learn about an important mission from me.",
        )];
        let player = commands
            .spawn((
                Transform::IDENTITY,
                GlobalTransform::IDENTITY,
                LegacyAvatarActionState::default(),
            ))
            .id();
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 2.2, 5.0).looking_at(Vec3::new(0.0, 1.1, 0.0), Vec3::Y),
            LegacyOrbitCamera::new(player),
        ));
        player_bubbles.request_message(player, "Player FreeChat bubble");
    }
    if let Some(chat_tab_preview) = chat_tab_preview {
        tutorial_overlay.hide_and_clear();
        let scroll_preview = chat_tab_preview.eq_ignore_ascii_case("all-scroll");
        model.chat.input_enabled = true;
        model.chat.active = !chat_tab_preview.ends_with("-inactive");
        model.chat.selected = match chat_tab_preview.trim_end_matches("-inactive") {
            "all" | "all-scroll" => ChatChannel::All,
            "buddy" => ChatChannel::Buddy,
            "group" => ChatChannel::Group,
            value => panic!(
                "FFONE_CHAT_TAB_PREVIEW must be all, all-scroll, buddy, group, or an -inactive variant; got {value:?}"
            ),
        };
        model.chat.group_available = false;
        model.chat.buddy_count = 0;
        model.chat.lines = if scroll_preview {
            (1..=18)
                .map(|index| ChatLineUi::normal(format!("Retrobution chat history line {index}.")))
                .collect()
        } else {
            Vec::new()
        };
        buddy_ui.set_chat_window_style(BuddyChatWindowStyle::Large);
        buddy_ui.set_large_chat_width(model.chat.window_size.x);
        buddy_ui.set_ui_scale(model.ui_scale);
        buddy_ui.set_visible(model.chat.selected == ChatChannel::Buddy && model.chat.active);
    }
    if let Some(mode) = mission_preview {
        mission_ui.enabled = true;
        tutorial_overlay.hide_and_clear();
        let mission = MissionUiEntry {
            task_id: 2248,
            npc_id: Some(100),
            mission_type: 3,
            task_type: 5,
            outgoing_task_id: 0,
            has_task_reward: true,
            is_first_mission_task: false,
            journal_npc_type: 2671,
            required_level: 1,
            difficulty_type: 0,
            nano: None,
            title: "A Welcome Distraction".to_owned(),
            npc_name: "Numbuh Two".to_owned(),
            npc_position: "Pokey Oaks North".to_owned(),
            objective: "Defeat the Oil Ogre.".to_owned(),
            offer_description:
                "Report to Numbuh Two and help prepare the Kids Next Door for the Fusion invasion."
                    .to_owned(),
            task_description: "Bring its transmitter back to Numbuh Two.".to_owned(),
            active_description: "Defeat the Oil Ogre.\n\nBring its transmitter back to Numbuh Two."
                .to_owned(),
            mission_summary: "Recover a transmitter from Fuse's monsters.".to_owned(),
            mission_complete_summary:
                "I defeated an Oil Ogre and recovered its transmitter for Numbuh Two.".to_owned(),
            completion_description: "Return to Numbuh Two for your reward.".to_owned(),
            rewards: MissionUiRewards {
                cash: 125,
                fusion_matter: 75,
            },
        };
        match mode.as_str() {
            "warp-departure" => {
                mission_ui.warp_transition_active = true;
                tutorial_overlay.visible = true;
                // An independent scene camera keeps a visible backdrop after
                // the HUD is hidden, so the letterbox bounds can be inspected.
                commands.spawn(Camera3d::default());
            }
            "offer" => mission_ui.journal = MissionJournalUi::Allow(mission),
            "nano" => {
                let task = env::var("FFONE_MISSION_NANO_TASK_ID")
                    .map(|value| value.parse().expect("numeric Nano mission task ID"))
                    .unwrap_or(2250);
                mission_ui.journal = MissionJournalUi::Allow(
                    mission_content
                        .journal_entry(task, "Pokey Oaks North")
                        .expect("published Nano mission"),
                );
            }
            "active-world" => {
                let world_mission = mission_content
                    .journal_entry(451, "Sector V")
                    .expect("Retrobution Spawn Spree task 451");
                mission_ui.viewed_journal_task_id = Some(world_mission.task_id);
                mission_ui.selected_journal_task_id = Some(world_mission.task_id);
                mission_ui.journal = MissionJournalUi::Other(JournalOtherUi {
                    title: "MISSION JOURNAL".to_owned(),
                    active_missions: vec![world_mission],
                    ..default()
                });
            }
            "active" => {
                mission_ui.viewed_journal_task_id = Some(mission.task_id);
                mission_ui.selected_journal_task_id = Some(mission.task_id);
                mission_ui.journal = MissionJournalUi::Other(JournalOtherUi {
                    title: "MISSION JOURNAL".to_owned(),
                    active_missions: vec![mission],
                    ..default()
                });
            }
            "active-untracked" => {
                let mut second = mission.clone();
                second.task_id = 2249;
                second.title = "A Stolen Transmitter".to_owned();
                second.objective = "Return to Numbuh Two.".to_owned();
                mission_ui.viewed_journal_task_id = Some(second.task_id);
                mission_ui.selected_journal_task_id = Some(mission.task_id);
                mission_ui.journal = MissionJournalUi::Other(JournalOtherUi {
                    title: "MISSION JOURNAL".to_owned(),
                    active_missions: vec![mission, second],
                    ..default()
                });
            }
            "active-row-click" => {
                let first = mission_content
                    .journal_entry(449, "Sector V")
                    .expect("Retrobution Spawn Spree task 449");
                let second = mission_content
                    .journal_entry(451, "Sector V")
                    .expect("Retrobution Spawn Spree task 451");
                let first_task_id = first.task_id;
                let second_task_id = second.task_id;
                mission_ui.viewed_journal_task_id = Some(first_task_id);
                mission_ui.selected_journal_task_id = Some(first_task_id);
                mission_ui.journal = MissionJournalUi::Other(JournalOtherUi {
                    title: "MISSION JOURNAL".to_owned(),
                    active_missions: vec![first, second],
                    ..default()
                });
                assert!(mission_ui.select_journal_mission(1));
                assert_eq!(mission_ui.viewed_journal_task_id, Some(second_task_id));
                assert_eq!(mission_ui.selected_journal_task_id, Some(second_task_id));
            }
            "completed" => {
                let completed_missions = mission_content
                    .completed_journal_entries(
                        &[2248, 2249, 2250, 2251, 2252, 2253, 2254],
                        "Pokey Oaks North",
                    )
                    .expect("Retrobution final-task completed tutorial history");
                mission_ui.journal = MissionJournalUi::Other(JournalOtherUi {
                    title: "MISSION JOURNAL".to_owned(),
                    active_missions: Vec::new(),
                    completed_missions,
                });
                assert!(mission_ui.select_journal_tab(JournalListTab::Completed));
            }
            "reward" => {
                mission_ui.journal = MissionJournalUi::Reward {
                    mission,
                    box1_choice: 0,
                    box2_choice: 0,
                };
            }
            "reward-world" => {
                let world_mission = mission_content
                    .journal_entry(451, "Sector V")
                    .expect("Retrobution Spawn Spree task 451");
                mission_ui.viewed_journal_task_id = Some(world_mission.task_id);
                mission_ui.selected_journal_task_id = Some(world_mission.task_id);
                mission_ui.nanocom_journal = JournalOtherUi {
                    active_missions: vec![world_mission.clone()],
                    ..default()
                };
                mission_ui.journal = MissionJournalUi::Reward {
                    mission: world_mission,
                    box1_choice: 0,
                    box2_choice: 0,
                };
            }
            "reward-nano" => {
                mission_ui.journal = MissionJournalUi::Reward {
                    mission: mission_content
                        .journal_entry(2250, "Pokey Oaks North")
                        .expect("Retrobution Nano tutorial mission 2250"),
                    box1_choice: 0,
                    box2_choice: 0,
                };
                tutorial_overlay.visible = true;
                tutorial_overlay.tutorial.secondary_instruction = Some(
                    localization
                        .text(
                            &language,
                            &localized_tutorial_instruction(
                                "Select \"A Fusion Matter\" from mission menu.",
                            ),
                        )
                        .to_uppercase(),
                );
                tutorial_overlay.tutorial.arrow = Some(TutorialArrowCue::at(
                    TutorialArrowDirection::Right,
                    702.0,
                    299.0,
                ));
            }
            "vendor" => mission_ui.show_npc_interaction(NpcInteractionUi {
                npc_id: 77,
                name: "Shopkeeper".to_owned(),
                services: vec![NpcServiceUiEntry::original(NpcServiceKind::Vendor)],
                ..default()
            }),
            "quest" => mission_ui.show_npc_interaction(NpcInteractionUi {
                npc_id: 100,
                name: "Numbuh Two".to_owned(),
                available_missions: vec![mission],
                ..default()
            }),
            "close" => {
                tutorial_overlay.visible = true;
                tutorial_overlay.tutorial.secondary_instruction =
                    Some("CLICK THE \"CLOSE\" BUTTON.".to_owned());
                tutorial_overlay.tutorial.arrow = Some(TutorialArrowCue::at(
                    TutorialArrowDirection::Left,
                    1094.0,
                    314.0,
                ));
                mission_ui.show_npc_interaction(NpcInteractionUi {
                    npc_id: 100,
                    name: "Numbuh Two".to_owned(),
                    ..default()
                });
            }
            "enter" | "enter-mid" => mission_ui.nanocom_main_menu_visible = true,
            "system" => {
                mission_ui.open_tutorial_exit_dialog();
            }
            "hostile" => {
                // Oil Ogre is exact NpcTable row 2676: hostile style 1, level
                // 1, 1300 HP and no active mob skill. Select the style-0 Nano
                // so the natural-size `nano_win` overlay is exercised too.
                model.nanos[0].active = false;
                model.nanos[1].active = true;
                let target = commands
                    .spawn(TutorialActor {
                        id: 1100,
                        npc_type: 2676,
                        team: 2,
                        hp: 650,
                        max_hp: 1300,
                        damaged: true,
                        interacting: false,
                        invulnerable: false,
                    })
                    .id();
                let mut action = LegacyAvatarActionState::default();
                action.target_selection.focused_npc = Some(LegacyFocusedTarget {
                    entity: target,
                    kind: LegacyTargetKind::Npc { team: 2 },
                    distance: 3.0,
                    talk_enabled: false,
                });
                commands.spawn((
                    action,
                    LegacyAvatarActionContext {
                        combat_condition: true,
                        ..default()
                    },
                ));
            }
            "nano-target" => {
                // Exercise the exact `PrintName.SkillIcon` state without also fabricating
                // a weapon-target bracket. Oil Ogre is exact NpcTable row 2676.
                let target = commands
                    .spawn((
                        Transform::IDENTITY,
                        GlobalTransform::IDENTITY,
                        TutorialActor {
                            id: 1100,
                            npc_type: 2676,
                            team: 2,
                            hp: 650,
                            max_hp: 1300,
                            damaged: true,
                            interacting: false,
                            invulnerable: false,
                        },
                    ))
                    .id();
                let focused = LegacyFocusedTarget {
                    entity: target,
                    kind: LegacyTargetKind::Npc { team: 2 },
                    distance: 3.0,
                    talk_enabled: false,
                };
                let mut action = LegacyAvatarActionState::default();
                action.target_selection.focused_npc = Some(focused);
                action.target_selection.nano_targets = vec![LegacyAttackTarget {
                    entity: target,
                    kind: focused.kind,
                    distance: focused.distance,
                }];
                let player = commands
                    .spawn((
                        Transform::from_xyz(0.0, 0.0, 5.0),
                        GlobalTransform::IDENTITY,
                        action,
                        LegacyAvatarActionContext::default(),
                    ))
                    .id();
                commands.spawn((
                    Camera3d::default(),
                    Transform::from_xyz(0.0, 3.0, 8.0)
                        .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
                    LegacyOrbitCamera::new(player),
                ));
            }
            "hostile-world" => {
                // Great Shellslug is exact normal-world NpcTable row 2071.
                // Its m_iIcon1 route resolves through m_pNpcIconData[184] to
                // installed mobicon_83, exercising the network target path.
                model.nanos[0].active = false;
                model.nanos[1].active = true;
                let target = commands
                    .spawn((
                        GlobalTransform::IDENTITY,
                        NetworkNpcAppearance0104(NpcAppearance0104 {
                            npc_id: 1100,
                            npc_type: 2071,
                            hp: 946,
                            condition_bit_flag: 0,
                            position: [0; 3],
                            angle: 0,
                            barker_type: 0,
                        }),
                    ))
                    .id();
                let mut action = LegacyAvatarActionState::default();
                action.target_selection.focused_npc = Some(LegacyFocusedTarget {
                    entity: target,
                    kind: LegacyTargetKind::Npc { team: 2 },
                    distance: 3.0,
                    talk_enabled: false,
                });
                commands.spawn((
                    action,
                    LegacyAvatarActionContext {
                        combat_condition: true,
                        ..default()
                    },
                ));
            }
            "trigger-use" | "trigger-hidden" => {
                tutorial_overlay.hide_and_clear();
                let target = commands.spawn_empty().id();
                let mut action = LegacyAvatarActionState::default();
                action.target_selection.trigger = Some(target);
                let player = commands
                    .spawn((
                        action,
                        LegacyAvatarActionContext {
                            move_mode: if mode == "trigger-hidden" {
                                ffone_client::avatar_action::LegacyMoveMode::Other
                            } else {
                                ffone_client::avatar_action::LegacyMoveMode::None
                            },
                            ..default()
                        },
                    ))
                    .id();
                commands.spawn(LegacyOrbitCamera::new(player));
            }
            "danger" => {
                commands.spawn((
                    LegacyAvatarActionState::default(),
                    LegacyAvatarActionContext {
                        combat_condition: true,
                        ..default()
                    },
                ));
            }
            _ => {}
        }
    }
}
