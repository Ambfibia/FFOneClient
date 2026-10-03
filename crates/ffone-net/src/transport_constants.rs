use super::*;

pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub const DEFAULT_IO_TIMEOUT: Duration = Duration::from_secs(60);

/// Shard number the clean web-player client puts into `SHARD_SELECT`.
///
/// `CnGuiLogin.Start` initializes `iServerSelected = 1` and immediately resets
/// it to 0 on `RuntimePlatform.WindowsWebPlayer`/`OSXWebPlayer`, which is the
/// only way Retrobution ships. The fixed `US` locale never opens
/// `ServerSelectionMode`, so nothing else assigns the value.
pub const CLEAN_WEB_PLAYER_SHARD_NUM_0104: i8 = 0;
