use super::*;

#[derive(Debug, Default, Resource)]
pub(in super::super) struct CombiNetworkFrameInbox0104(pub(in super::super) VecDeque<DecodedFrame>);

impl CombiNetworkFrameInbox0104 {
    pub(in super::super) fn push_if_owned(&mut self, runtime: &CombiProductionRuntime0104, frame: DecodedFrame) -> bool {
        if !runtime.request_pending()
            || !matches!(
                frame.packet_type,
                COMBI_SUCCESS_PACKET_ID_0104 | COMBI_FAILURE_PACKET_ID_0104
            )
        {
            return false;
        }
        self.0.push_back(frame);
        true
    }

    pub(in super::super) fn pop_front(&mut self) -> Option<DecodedFrame> {
        self.0.pop_front()
    }

    pub(in super::super) fn clear(&mut self) {
        self.0.clear();
    }
}
