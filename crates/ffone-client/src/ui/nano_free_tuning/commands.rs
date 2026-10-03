use super::*;

pub const NANO_TUNE_REQUEST_SIZE: usize = 44;

pub const NANO_TUNE_REQUEST_ABI: [NanoFreeTuningAbiField; 3] = [
    NanoFreeTuningAbiField {
        clean_name: "iNanoID",
        offset: 0,
        scalar: NanoFreeTuningAbiScalar::I16,
        count: 1,
    },
    NanoFreeTuningAbiField {
        clean_name: "iTuneID",
        offset: 2,
        scalar: NanoFreeTuningAbiScalar::I16,
        count: 1,
    },
    NanoFreeTuningAbiField {
        clean_name: "aiNeedItemSlotNum",
        offset: 4,
        scalar: NanoFreeTuningAbiScalar::I32,
        count: NANO_TUNE_ITEM_SLOT_COUNT,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoTuneRequest {
    pub nano_id: i16,
    pub tune_id: i16,
    pub needed_item_slots: [i32; NANO_TUNE_ITEM_SLOT_COUNT],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningReplyBody {
    Success(NanoTuneSuccess),
    Failure(NanoTuneFailure),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningReplyEnvelope {
    /// Local correlation only; it is not part of the clean packet bytes.
    pub request_token: u64,
    pub packet_id: u32,
    pub payload_size: usize,
    pub body: NanoFreeTuningReplyBody,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NanoFreeTuningEffectIntent {
    Preload {
        effect_id: i32,
    },
    Instantiate {
        effect_id: i32,
        position: Vec3,
        rotation: Quat,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NanoFreeTuningWorldIntent {
    FacePlayerToward(Vec3),
    SpawnPreviewNano {
        nano_id: i16,
        initial_position: Vec3,
    },
    SpawnCreationBullets {
        bullet_types: [i32; 2],
        position: Vec3,
    },
    RevealPreviewNano {
        position: Vec3,
        rotation: Quat,
    },
    DestroyPreviewNano,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NanoFreeTuningCinematicIntent {
    LookAt(Vec3),
    ConfigureCreationCamera {
        player_rotation: Quat,
        rotation_x: f32,
        distance: f32,
    },
    ApproachFrame {
        rotation_x_delta: f32,
        distance_delta: f32,
    },
    SetSubTarget {
        preview_rotation: Quat,
        player_rotation: Quat,
        distance: f32,
        height: f32,
    },
    CallPreviewNano,
    SetStandMotion,
    PlaySelectedSkill {
        power_index: usize,
    },
    PlayIdleHappy,
    HidePreviewNano,
    EndSubTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningUiIntent {
    Show,
    Hide,
    SetEnabled(bool),
    CaptureAndUnlockCursor,
    RestoreCapturedCursor,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NanoFreeTuningIntent {
    Wire(NanoTuneWireIntent),
    Sound(NanoFreeTuningSoundIntent),
    Effect(NanoFreeTuningEffectIntent),
    World(NanoFreeTuningWorldIntent),
    Cinematic(NanoFreeTuningCinematicIntent),
    Ui(NanoFreeTuningUiIntent),
    FirstUseCheck(i32),
    SetUpsellUpdate(bool),
    DeleteEcomIcon(i32),
    AuthoritativeCommit(NanoTuneSuccess),
    AuthoritativeFailure(NanoTuneFailure),
    RequestIdleRoll { exclusive_max: i32 },
    QueryAcquisitionContinuation,
    ProtocolFault(NanoFreeTuningProtocolFault),
    ExitMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningPendingRequest {
    pub request_token: u64,
    pub power_index: usize,
    pub nano_id: i16,
    pub tune_id: i16,
    pub skill_id: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningUiCommand {
    SelectAndConfirm { power_index: usize },
    Cancel,
}

#[derive(Debug, Default, Resource)]
pub struct NanoFreeTuningUiCommandOutbox(pub(super) VecDeque<NanoFreeTuningUiCommand>);

impl NanoFreeTuningUiCommandOutbox {
    pub fn push(&mut self, command: NanoFreeTuningUiCommand) {
        self.0.push_back(command);
    }

    pub fn pop(&mut self) -> Option<NanoFreeTuningUiCommand> {
        self.0.pop_front()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
