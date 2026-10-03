use super::*;

pub(in super::super) fn tutorial_ui_for_stage(
    stage: TutorialStage,
    elapsed_secs: f32,
    viewport_width: i32,
    viewport_height: i32,
    localization: &Localization,
    language: &Language,
) -> TutorialUi {
    let screen_width = viewport_width as f32;
    let journal_left = ((viewport_width - 1036) / 2) as f32;
    let journal_top = ((viewport_height - 654) / 2) as f32;
    let half_width = (viewport_width / 2) as f32;
    let quarter_height = (viewport_height / 4) as f32;
    let half_height = (viewport_height / 2) as f32;
    let npc_menu_pivot_x = (viewport_width * 3 / 4) as f32;
    let mission_dialog_pivot =
        TutorialCueScalePivot::Point(Vec2::new(npc_menu_pivot_x, half_height));
    let (illustration, arrow) = match stage {
        TutorialStage::Movement(MovementStage::LookRight) => (
            Some(TutorialIllustrationCue::mouse()),
            Some(TutorialArrowCue::mouse(TutorialArrowDirection::Right)),
        ),
        TutorialStage::Movement(MovementStage::LookLeft) => (
            Some(TutorialIllustrationCue::mouse()),
            Some(TutorialArrowCue::mouse(TutorialArrowDirection::Left)),
        ),
        TutorialStage::Movement(MovementStage::LookUp) => (
            Some(TutorialIllustrationCue::mouse()),
            Some(TutorialArrowCue::mouse(TutorialArrowDirection::Up)),
        ),
        TutorialStage::Movement(MovementStage::LookDown) => (
            Some(TutorialIllustrationCue::mouse()),
            Some(TutorialArrowCue::mouse(TutorialArrowDirection::Down)),
        ),
        TutorialStage::Movement(MovementStage::MoveForward) if elapsed_secs < 6.0 => {
            (Some(TutorialIllustrationCue::move_keys()), None)
        }
        TutorialStage::Movement(
            MovementStage::MoveForward | MovementStage::MoveAndSteer | MovementStage::ReachLedge,
        ) => (Some(TutorialIllustrationCue::move_forward()), None),
        TutorialStage::Movement(MovementStage::MoveBackward) => {
            (Some(TutorialIllustrationCue::move_backward()), None)
        }
        TutorialStage::Movement(MovementStage::JumpAndLand) => {
            (Some(TutorialIllustrationCue::jump()), None)
        }
        TutorialStage::Combat(CombatStage::TargetHostile) => {
            (Some(TutorialIllustrationCue::mouse()), None)
        }
        TutorialStage::Combat(CombatStage::FirstAttack)
        | TutorialStage::Mission(MissionStage::TargetAndTalk) => {
            (Some(TutorialIllustrationCue::left_mouse()), None)
        }
        TutorialStage::Combat(CombatStage::KillThree) => (
            None,
            Some(
                TutorialArrowCue::at(TutorialArrowDirection::Up, half_width - 35.0, 40.0)
                    .with_scale_pivot(TutorialCueScalePivot::CenterTop),
            ),
        ),
        TutorialStage::Combat(CombatStage::PlayerWasHit) => (
            None,
            Some(
                TutorialArrowCue::at(TutorialArrowDirection::Up, 150.0, 40.0)
                    .with_scale_pivot(TutorialCueScalePivot::TopLeft),
            ),
        ),
        TutorialStage::Minimap(MinimapStage::MinimapSequence) => (
            None,
            Some(
                TutorialArrowCue::at(TutorialArrowDirection::Right, screen_width - 280.0, 50.0)
                    .with_scale_pivot(TutorialCueScalePivot::TopRight),
            ),
        ),
        TutorialStage::Mission(MissionStage::MissionMenu | MissionStage::RewardMenu) => {
            let (left, top) = if elapsed_secs < 4.5 {
                (half_width - 100.0, quarter_height)
            } else {
                (npc_menu_pivot_x - 246.0, half_height - 41.0)
            };
            (
                None,
                Some(
                    TutorialArrowCue::at(TutorialArrowDirection::Right, left, top)
                        .with_scale_pivot(if elapsed_secs < 4.5 {
                            TutorialCueScalePivot::Point(Vec2::new(half_width, quarter_height))
                        } else {
                            mission_dialog_pivot
                        }),
                ),
            )
        }
        TutorialStage::Mission(MissionStage::ObjectiveCombat) => (
            None,
            Some(
                TutorialArrowCue::at(TutorialArrowDirection::Right, screen_width - 259.0, 200.0)
                    .with_scale_pivot(TutorialCueScalePivot::TopRight),
            ),
        ),
        TutorialStage::Mission(MissionStage::SelectJournal) => (
            None,
            Some(
                TutorialArrowCue::at(TutorialArrowDirection::Right, screen_width - 259.0, 190.0)
                    .with_scale_pivot(TutorialCueScalePivot::TopRight),
            ),
        ),
        TutorialStage::Mission(MissionStage::JournalIntro) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Up,
                    journal_left + 740.0,
                    journal_top + 280.0,
                )
                .with_scale_pivot(TutorialCueScalePivot::Center),
            ),
        ),
        TutorialStage::Mission(MissionStage::JournalMissionTab) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Left,
                    journal_left + 300.0,
                    journal_top + 100.0,
                )
                .with_scale_pivot(TutorialCueScalePivot::Center),
            ),
        ),
        TutorialStage::Mission(MissionStage::JournalObjective) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Left,
                    journal_left + 545.0,
                    journal_top + 200.0,
                )
                .with_scale_pivot(TutorialCueScalePivot::Center),
            ),
        ),
        TutorialStage::Mission(MissionStage::JournalDetail) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Left,
                    journal_left + 295.0,
                    journal_top + 380.0,
                )
                .with_scale_pivot(TutorialCueScalePivot::Center),
            ),
        ),
        TutorialStage::Mission(MissionStage::CloseJournal) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Right,
                    journal_left + 896.0,
                    journal_top,
                )
                .with_scale_pivot(TutorialCueScalePivot::Center),
            ),
        ),
        TutorialStage::Mission(MissionStage::ReturnToNumbuhTwo) => (
            None,
            Some(
                TutorialArrowCue::at(TutorialArrowDirection::Right, screen_width - 259.0, 200.0)
                    .with_scale_pivot(TutorialCueScalePivot::TopRight),
            ),
        ),
        TutorialStage::Infection(
            InfectionStage::ApproachButtercup
            | InfectionStage::TalkButtercup
            | InfectionStage::TravelToGate
            | InfectionStage::SelectAttendant,
        ) => (
            None,
            Some(
                TutorialArrowCue::at(TutorialArrowDirection::Right, screen_width - 259.0, 200.0)
                    .with_scale_pivot(TutorialCueScalePivot::TopRight),
            ),
        ),
        TutorialStage::Mission(MissionStage::AcceptMission) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Down,
                    journal_left + 420.0,
                    journal_top + 490.0,
                )
                .with_scale_pivot(TutorialCueScalePivot::Center),
            ),
        ),
        TutorialStage::Mission(MissionStage::ClaimReward) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Down,
                    journal_left + 400.0,
                    journal_top + 490.0,
                )
                .with_scale_pivot(TutorialCueScalePivot::Center),
            ),
        ),
        TutorialStage::Mission(MissionStage::CloseAccept | MissionStage::CloseReward)
        | TutorialStage::Infection(InfectionStage::CloseMission) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Left,
                    npc_menu_pivot_x + 146.0,
                    half_height - 26.0,
                )
                .with_scale_pivot(mission_dialog_pivot),
            ),
        ),
        TutorialStage::Infection(
            InfectionStage::SelectMission | InfectionStage::SelectDexterMission,
        ) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Right,
                    npc_menu_pivot_x - 246.0,
                    half_height - 41.0,
                )
                .with_scale_pivot(mission_dialog_pivot),
            ),
        ),
        TutorialStage::Infection(
            InfectionStage::WarpFromTechSquare
            | InfectionStage::WarpIntoLair
            | InfectionStage::WarpOut,
        ) => (
            None,
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Right,
                    npc_menu_pivot_x - 201.0,
                    half_height - 37.0,
                )
                .with_scale_pivot(mission_dialog_pivot),
            ),
        ),
        TutorialStage::NanoPower(NanoPowerStage::SummonNano) => (
            Some(TutorialIllustrationCue::nano_one()),
            Some(
                TutorialArrowCue::at(
                    TutorialArrowDirection::Down,
                    screen_width - 240.0,
                    viewport_height as f32 - 180.0,
                )
                .with_scale_pivot(TutorialCueScalePivot::BottomRight),
            ),
        ),
        TutorialStage::NanoPower(NanoPowerStage::UseNanoPower) => {
            (Some(TutorialIllustrationCue::right_mouse()), None)
        }
        _ => (None, None),
    };
    let instruction = stage.metadata().instruction.map(|instruction| {
        localization
            .text(language, &localized_tutorial_instruction(instruction))
            .to_uppercase()
    });
    // The two Chapter-05 mission-menu prompts are coroutine-owned: the source
    // shows only the pointer/voice for 4.5 seconds, then calls `SubText2`.
    // `TutorialAuxiliaryPresentation` supplies that delayed secondary text;
    // emitting stage metadata here would create an early quarter-screen copy.
    let instruction_owned_by_auxiliary = matches!(
        stage,
        TutorialStage::Mission(MissionStage::MissionMenu | MissionStage::RewardMenu)
    );
    // `SubText2` renders at y=0. These are every stage-backed Chapter-05/06
    // call to that layer: close prompts, both Fusion Matter selections, and
    // the three warp prompts. Routing them through regular `SubText` places
    // them at Screen.height / 4 over the journal contents.
    let uses_secondary_instruction = matches!(
        stage,
        TutorialStage::Mission(MissionStage::CloseAccept | MissionStage::CloseReward)
            | TutorialStage::Infection(
                InfectionStage::SelectMission
                    | InfectionStage::CloseMission
                    | InfectionStage::WarpFromTechSquare
                    | InfectionStage::WarpIntoLair
                    | InfectionStage::SelectDexterMission
                    | InfectionStage::WarpOut
            )
    );
    TutorialUi {
        instruction: if instruction_owned_by_auxiliary || uses_secondary_instruction {
            None
        } else {
            instruction.clone()
        },
        secondary_instruction: if !instruction_owned_by_auxiliary && uses_secondary_instruction {
            instruction
        } else {
            None
        },
        illustration,
        arrow,
    }
}

pub(super) fn forward_key_binding(input: &InputSettings) -> Option<LegacyPhysicalKey> {
    let row = input
        .mappings
        .iter()
        .find(|row| row.action == LegacyOptionAction::Up)?;
    [row.primary, row.alternate]
        .into_iter()
        .find_map(|binding| match binding {
            LegacyInputBinding::Key(key)
                if !matches!(key, LegacyPhysicalKey::Mouse0 | LegacyPhysicalKey::Mouse1) =>
            {
                Some(key)
            }
            _ => None,
        })
}

pub(super) fn bind_forward_key(instruction: &mut Option<String>, input: &InputSettings) {
    let Some(instruction) = instruction else {
        return;
    };
    let key = forward_key_binding(input);
    if let Some(key) = key {
        let key = format!("{key:?}").to_uppercase();
        *instruction = instruction
            .replace("«W»", &format!("«{key}»"))
            .replace("\"W\"", &format!("\"{key}\""));
    } else {
        instruction.clear();
    }
}

#[cfg(test)]
mod forward_key_tests {
    use super::*;

    #[test]
    fn tutorial_forward_prompt_uses_the_committed_key_binding() {
        let mut input = InputSettings::default();
        let forward = input
            .mappings
            .iter_mut()
            .find(|row| row.action == LegacyOptionAction::Up)
            .unwrap();
        forward.primary = LegacyInputBinding::Key(LegacyPhysicalKey::Q);
        assert_eq!(forward_key_binding(&input), Some(LegacyPhysicalKey::Q));
        let mut russian =
            Some("НАЖМИТЕ И УДЕРЖИВАЙТЕ КЛАВИШУ «W», ЧТОБЫ ДВИГАТЬСЯ ВПЕРЁД.".to_owned());
        let mut english = Some("PRESS AND HOLD THE \"W\" KEY TO MOVE FORWARD.".to_owned());
        bind_forward_key(&mut russian, &input);
        bind_forward_key(&mut english, &input);
        assert!(russian.unwrap().contains("«Q»"));
        assert!(english.unwrap().contains("\"Q\""));
    }
}

pub(in super::super) fn tutorial_auxiliary_presentation_matches(
    presentation: &TutorialAuxiliaryPresentation,
    mission_runtime: &TutorialMissionRuntime,
    stage: Option<TutorialStage>,
) -> bool {
    // Chapter_06 owns both the SubText2 prompt and mission-row cursor directly
    // in these stages. `TalkBCup_1` may still be retained by the native
    // 27-second FM timeline, but its earlier top-right FM-meter cursor must not
    // override the new stage. This was the stale arrow visible above the Nano
    // reward journal in the reported frame.
    if matches!(
        stage,
        Some(TutorialStage::Infection(
            InfectionStage::SelectMission | InfectionStage::SelectDexterMission
        ))
    ) {
        return false;
    }
    if presentation.choreography_owned {
        return true;
    }
    let Some(sequence) = presentation.sequence else {
        return false;
    };
    match sequence {
        TutorialAuxiliarySequence::BasicArrowKey => {
            stage == Some(TutorialStage::Movement(MovementStage::MoveForward))
        }
        TutorialAuxiliarySequence::NanoPowerTalk1 => {
            stage == Some(TutorialStage::NanoPower(NanoPowerStage::UseNanoPower))
        }
        _ => mission_runtime
            .auxiliary_dialogue
            .is_some_and(|dialogue| tutorial_auxiliary_for_dialogue(dialogue) == sequence),
    }
}

pub(super) fn tutorial_illustration_cue(
    resource: &str,
    position: TutorialScreenPoint,
    pivot: AuxiliaryScreenPivot,
    viewport_width: i32,
    viewport_height: i32,
) -> Option<TutorialIllustrationCue> {
    let illustration = match resource {
        "tut_mouse" => TutorialIllustration::Mouse,
        "tut_lmouse" => TutorialIllustration::LeftMouse,
        "tut_rmouse" => TutorialIllustration::RightMouse,
        "tut_move" => TutorialIllustration::Move,
        "tut_move_s" => TutorialIllustration::MoveBackward,
        "tut_move_w" => TutorialIllustration::MoveForward,
        "tut_jump" => TutorialIllustration::Jump,
        "tut_one" => TutorialIllustration::NanoOne,
        _ => return None,
    };
    let [left, top] = position.evaluate(viewport_width, viewport_height);
    Some(
        TutorialIllustrationCue::at(illustration, left as f32, top as f32).with_scale_pivot(
            tutorial_cue_scale_pivot(pivot, viewport_width, viewport_height)?,
        ),
    )
}

pub(super) fn tutorial_arrow_cue(
    resource: &str,
    position: TutorialScreenPoint,
    pivot: AuxiliaryScreenPivot,
    viewport_width: i32,
    viewport_height: i32,
) -> Option<TutorialArrowCue> {
    let direction = match resource {
        "tut_left" => TutorialArrowDirection::Left,
        "tut_right" => TutorialArrowDirection::Right,
        "tut_up" => TutorialArrowDirection::Up,
        "tut_down" => TutorialArrowDirection::Down,
        _ => return None,
    };
    let [left, top] = position.evaluate(viewport_width, viewport_height);
    Some(
        TutorialArrowCue::at(direction, left as f32, top as f32).with_scale_pivot(
            tutorial_cue_scale_pivot(pivot, viewport_width, viewport_height)?,
        ),
    )
}

pub(super) fn tutorial_cue_scale_pivot(
    pivot: AuxiliaryScreenPivot,
    viewport_width: i32,
    viewport_height: i32,
) -> Option<TutorialCueScalePivot> {
    match pivot {
        AuxiliaryScreenPivot::Legacy(value) => TutorialCueScalePivot::from_legacy(value),
        AuxiliaryScreenPivot::Point(point) => {
            let [x, y] = point.evaluate(viewport_width, viewport_height);
            Some(TutorialCueScalePivot::Point(Vec2::new(x as f32, y as f32)))
        }
    }
}
