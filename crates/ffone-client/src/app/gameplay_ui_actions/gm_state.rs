use ffone_protocol::{
    DecodedFrame, WirePayload,
    wire_0104::{PcGiveItemRequest0104, PcGiveItemSuccess0104},
};
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub(crate) struct GmRuntime {
    pub owner: Option<i32>,
    pub items: VecDeque<String>,
    pub pending: Option<ffone_protocol::wire_0104::PcGiveItemRequest0104>,
    pub age: f32,
    pub stalled: bool,
    pub notice: Option<(&'static str, &'static str)>,
    pub view_location: bool,
    pub view_network: bool,
    pub view_ids: bool,
    pub view_collision: bool,
    pub hide_ui: bool,
    pub announcement: Option<(String, f32)>,
    pub record_history: bool,
    pub history: VecDeque<(u32, usize)>,
    pub store_open: Option<i32>,
    pub store_frames: VecDeque<DecodedFrame>,
    pub shiny_random: u32,
}

impl GmRuntime {
    /// Parser adds the primary 200-unit minimum; choose its remaining 0..399
    /// jitter independently for X and Y, without changing the altitude.
    pub fn shiny_jitter(&mut self, mut position: [i32; 3]) -> Option<[i32; 3]> {
        if self.shiny_random == 0 {
            self.shiny_random = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .subsec_nanos()
                .max(1);
        }
        for coordinate in &mut position[..2] {
            self.shiny_random ^= self.shiny_random << 13;
            self.shiny_random ^= self.shiny_random >> 17;
            self.shiny_random ^= self.shiny_random << 5;
            *coordinate = coordinate.checked_add((self.shiny_random % 400) as i32)?;
        }
        Some(position)
    }
    pub fn next_item(&mut self) -> Option<String> {
        if self.pending.is_some() || self.stalled {
            None
        } else {
            self.items.pop_front()
        }
    }
    pub fn start_item(&mut self, request: PcGiveItemRequest0104) {
        self.pending = Some(request);
        self.age = 0.0;
        self.stalled = false;
    }
    pub fn accept_item_reply(&mut self, frame: &DecodedFrame) {
        if frame.packet_type == 0x31000062
            && frame.payload.len() == 4
            && self.pending.take().is_some()
        {
            self.stalled = false;
            self.notice = Some((
                "ui.chat.command.gm.item_rejected",
                "The server rejected the item request.",
            ));
        } else if frame.packet_type == 0x31000061 {
            if let Ok(reply) = PcGiveItemSuccess0104::decode(&frame.payload) {
                if self.pending.as_ref().is_some_and(|p| {
                    p.e_il == reply.e_il
                        && p.item.id == reply.item.id
                        && p.item.type_ == reply.item.type_
                        && p.item.opt == reply.item.opt
                }) {
                    self.pending = None;
                    self.stalled = false;
                    self.notice = Some(("ui.chat.command.gm.item_received", "Item received."));
                }
            }
        }
    }
}
