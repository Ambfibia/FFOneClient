
pub const CASHMALL_HIDDEN_CHAT_COMMAND_0104: &str = "/cashmall";

pub const CASHMALL_HELP_EVENT_ID: i32 = 24;

pub const CASHMALL_VENDOR_POPUP_ACTION: i32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CashmallActionBlocked0104 {
    NotActive,
    ControlsDisabled,
    ModeRejectedEscape,
    ExitArbitrationRejected,
    MissingRow,
    /// Clean switches to a Nano projection owned by `Panel_PCStuffScript`, but
    /// this bounded native Cash Mall projection contains only item slots.
    NanoProjectionUnavailable,
    /// The clean redeem modal and free-chat transport are owned by the shared
    /// inventory runtime. Cash Mall must not synthesize that packet locally.
    RedeemRuntimeUnavailable,
}
