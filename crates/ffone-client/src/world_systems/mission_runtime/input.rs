
/// `cnMissionNode.InitializeBeforeStart` initializes `m_fRemainTime` to
/// exactly -1 second. PC-load restoration calls it but does not call
/// `SetRemainTime`, so a grant-timer task is already expired on the first
/// clean `cnMissionManager.Update` pass.
pub(super) const CLEAN_PC_LOAD_TIMER_INITIAL_MILLIS: i64 = -1_000;
