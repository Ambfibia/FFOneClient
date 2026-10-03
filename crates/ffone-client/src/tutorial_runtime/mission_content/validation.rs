use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TutorialMissionContentError {
    Io { path: String, detail: String },
    Invalid(String),
    UnknownMission(i32),
    UnknownNpc(i32),
    UnknownReward(i32),
    UnknownSceneEvent(i32),
    UnknownSceneText { event: i32, line: i32 },
    UnknownWarp(i32),
}

impl fmt::Display for TutorialMissionContentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, detail } => write!(formatter, "{path}: {detail}"),
            Self::Invalid(detail) => formatter.write_str(detail),
            Self::UnknownMission(task_id) => {
                write!(formatter, "tutorial mission task {task_id} is not loaded")
            }
            Self::UnknownNpc(npc_type) => {
                write!(formatter, "tutorial NPC type {npc_type} is not loaded")
            }
            Self::UnknownReward(reward_id) => {
                write!(formatter, "tutorial reward {reward_id} is not loaded")
            }
            Self::UnknownSceneEvent(event) => {
                write!(formatter, "tutorial cut-scene event {event} is not loaded")
            }
            Self::UnknownSceneText { event, line } => {
                write!(
                    formatter,
                    "tutorial cut-scene event {event} line {line} is not loaded"
                )
            }
            Self::UnknownWarp(npc_type) => {
                write!(formatter, "tutorial warp NPC type {npc_type} is not loaded")
            }
        }
    }
}

impl Error for TutorialMissionContentError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameplayWarpResolveError {
    NonFiniteNpcPosition { map_number: i32 },
}

impl fmt::Display for GameplayWarpResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteNpcPosition { map_number } => write!(
                formatter,
                "normal warp NPC position on map {map_number} contains a non-finite coordinate"
            ),
        }
    }
}

impl Error for GameplayWarpResolveError {}
