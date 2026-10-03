use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialIntent {
    StartScene(TutorialScene),
    FinishScene {
        scene: TutorialScene,
        completion: SceneCompletion,
    },
    SpawnNpc(TutorialNpcSpawn),
    DeleteNpc(i32),
    FaceNpc {
        id: i32,
        target: i32,
    },
    PlayNpcAnimation {
        id: i32,
        clip: &'static str,
        once: bool,
    },
    WarpPlayer(ClientPosition),
    AttackNpc(i32),
    StartTask(i32),
    ClearEventFlags,
    EndTask(i32),
    EquipNano {
        slot: i32,
        nano_id: i32,
        skill_id: i32,
    },
    SetFusionMatter(i32),
    SetFatigueLevel(i32),
    SetInstanceMap(bool),
    SetEpisode(i32),
    ConfigureDemoMonster {
        dont_kill: bool,
    },
    StartDialogue(TutorialDialogue),
    StopDialogue(TutorialDialogue),
    StartReminderTimer,
    StartDelayMillis(u32),
    SetWaypointToNpc(i32),
    ClearWaypoint,
    LockUi,
    UnlockUi,
    ExitUi,
    PushInputFilter,
    PopInputFilter,
    HideTutorialPointer,
    StopLoopSound,
    ExitTutorial,
}
