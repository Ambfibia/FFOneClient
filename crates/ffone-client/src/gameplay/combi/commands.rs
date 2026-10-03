use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CombiReplyPacket0104 {
    Success(CombiSuccessReply0104),
    Failure(CombiFailureReply0104),
}
