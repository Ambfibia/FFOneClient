use super::*;

#[derive(Debug)]
pub enum NetError {
    Io(io::Error),
    Frame(FrameError),
    Payload(PayloadError),
    NpcCombatPayload(NpcCombatDecodeError0104),
    UnexpectedPacket {
        phase: &'static str,
        packet_type: u32,
    },
    LoginRejected {
        error_code: i32,
    },
    ServerAnnouncement(String),
    CharacterNameCheckRejected {
        error_code: i32,
    },
    CharacterNameSaveRejected {
        error_code: i32,
    },
    CharacterCreateRejected {
        error_code: i32,
    },
    CharacterDeleteRejected {
        error_code: i32,
    },
    CharacterNameChangeRejected {
        error_code: i32,
    },
    CharacterCreationState {
        reason: &'static str,
    },
    CharacterResponseMismatch {
        phase: &'static str,
    },
    InvalidCharacterCount(i8),
    NoCharacters,
    UnknownCharacter(i64),
    ShardSelectRejected {
        error_code: i32,
    },
    /// The clean login server answered `CHAR_SELECT` with `CHAR_SELECT_FAIL`.
    /// OpenFusion never emits this packet; it rejects through `SHARD_SELECT_FAIL`.
    CharacterSelectRejected {
        error_code: i32,
    },
    InvalidShardEndpoint {
        ip: String,
        port: i32,
    },
    PcEnterRejected {
        error_code: i32,
    },
    LoginSenderPoisoned,
    GameplaySenderPoisoned,
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "network I/O failed: {error}"),
            Self::Frame(error) => write!(f, "wire frame failed: {error}"),
            Self::Payload(error) => write!(f, "packet payload failed: {error}"),
            Self::NpcCombatPayload(error) => write!(f, "NPC/combat packet failed: {error}"),
            Self::UnexpectedPacket { phase, packet_type } => {
                write!(f, "unexpected packet {packet_type:#010x} during {phase}")
            }
            Self::LoginRejected { error_code } => {
                write!(f, "OpenFusion rejected login with code {error_code}")
            }
            Self::ServerAnnouncement(message) => write!(f, "{message}"),
            Self::CharacterNameCheckRejected { error_code } => {
                write!(
                    f,
                    "OpenFusion rejected the character name with code {error_code}"
                )
            }
            Self::CharacterNameSaveRejected { error_code } => {
                write!(
                    f,
                    "OpenFusion rejected character name reservation with code {error_code}"
                )
            }
            Self::CharacterCreateRejected { error_code } => {
                write!(
                    f,
                    "OpenFusion rejected character creation with code {error_code}"
                )
            }
            Self::CharacterDeleteRejected { error_code } => {
                write!(
                    f,
                    "OpenFusion rejected character deletion with code {error_code}"
                )
            }
            Self::CharacterNameChangeRejected { error_code } => write!(
                f,
                "OpenFusion rejected character rename with code {error_code}"
            ),
            Self::CharacterCreationState { reason } => {
                write!(f, "invalid character-creation state: {reason}")
            }
            Self::CharacterResponseMismatch { phase } => {
                write!(
                    f,
                    "OpenFusion returned identity fields that do not match the {phase} request"
                )
            }
            Self::InvalidCharacterCount(count) => {
                write!(f, "login returned invalid character count {count}")
            }
            Self::NoCharacters => write!(f, "account has no selectable characters"),
            Self::UnknownCharacter(uid) => {
                write!(f, "character UID {uid} was not returned by login")
            }
            Self::ShardSelectRejected { error_code } => {
                write!(
                    f,
                    "OpenFusion rejected shard selection with code {error_code}"
                )
            }
            Self::CharacterSelectRejected { error_code } => {
                write!(
                    f,
                    "the login server rejected character selection with code {error_code}"
                )
            }
            Self::InvalidShardEndpoint { ip, port } => {
                write!(f, "OpenFusion returned invalid shard endpoint {ip}:{port}")
            }
            Self::PcEnterRejected { error_code } => {
                write!(f, "OpenFusion rejected shard entry with code {error_code}")
            }
            Self::LoginSenderPoisoned => write!(f, "login sender lock was poisoned"),
            Self::GameplaySenderPoisoned => write!(f, "gameplay sender lock was poisoned"),
        }
    }
}

impl std::error::Error for NetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Frame(error) => Some(error),
            Self::Payload(error) => Some(error),
            Self::NpcCombatPayload(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for NetError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FrameError> for NetError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}

impl From<PayloadError> for NetError {
    fn from(error: PayloadError) -> Self {
        Self::Payload(error)
    }
}

impl From<NpcCombatDecodeError0104> for NetError {
    fn from(error: NpcCombatDecodeError0104) -> Self {
        Self::NpcCombatPayload(error)
    }
}
