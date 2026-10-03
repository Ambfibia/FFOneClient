//! Recall registration and eligibility; the server owns the destination.
use crate::app::*;
use ffone_protocol::wire_0104::{
    RegistRxcomFailure0104, RegistRxcomReply0104, RegistRxcomRequest0104,
};

#[derive(Debug, Default)]
pub(super) struct NanoRecallState {
    instance: Option<i32>,
    point: Option<(i32, [i32; 3])>,
    pending_npc: Option<i32>,
    registered_npc: Option<i32>,
}

impl NanoRecallState {
    pub(super) fn registered_here(&self, npc_id: i32) -> bool {
        self.registered_npc == Some(npc_id) && self.restriction().is_none()
    }

    pub(super) fn leave_instance(&mut self) {
        *self = Self::default();
    }

    pub(super) fn restriction(&self) -> Option<LocalizedText> {
        if self.instance.is_none() {
            Some(LocalizedText::new(
                "content.tabledata.message.message.217.sz_string",
                "The Recall Power can not be used outside of an Infected Zone.",
            ))
        } else if !self
            .point
            .is_some_and(|(map, _)| Some(map) == self.instance)
        {
            Some(LocalizedText::new(
                "content.tabledata.message.message.218.sz_string",
                "You must first register at a Recall Point.",
            ))
        } else {
            None
        }
    }

    pub(super) fn register(&mut self, npc_id: i32, bridge: &NetworkBridge) -> Result<(), String> {
        if self.pending_npc.is_some() {
            return Ok(());
        }
        let request = ffone_protocol::RegisteredGameplayRequest0104::new(
            packet::P_CL2FE_REQ_REGIST_RXCOM,
            RegistRxcomRequest0104 { npcid: npc_id }.encode(),
        )
        .map_err(|error| error.to_string())?;
        bridge.send(NetworkCommand::SendRegisteredGameplay0104(request))?;
        self.pending_npc = Some(npc_id);
        Ok(())
    }

    pub(super) fn apply(&mut self, frame: &DecodedFrame) -> Result<Option<LocalizedText>, String> {
        match frame.packet_type {
            packet::P_FE2CL_INSTANCE_MAP_INFO => {
                decode_race_instance_map_info_0104(&frame.payload)?;
                let instance = Some(read_race_i32(&frame.payload, 0));
                if self.instance != instance {
                    self.point = None;
                    self.pending_npc = None;
                    self.registered_npc = None;
                }
                self.instance = instance;
            }
            packet::P_FE2CL_REP_REGIST_RXCOM => {
                let reply = RegistRxcomReply0104::decode(&frame.payload)
                    .map_err(|error| error.to_string())?;
                if let Some(npc_id) = self.pending_npc.take()
                    && self.instance == Some(reply.map_num)
                {
                    self.registered_npc = Some(npc_id);
                    self.point = Some((reply.map_num, [reply.x, reply.y, reply.z]));
                    return Ok(Some(LocalizedText::new(
                        "ui.nano.recall.registered",
                        "Recall Point registered! Use your Recall Nano to return to this point.",
                    )));
                }
            }
            packet::P_FE2CL_REP_REGIST_RXCOM_FAIL => {
                RegistRxcomFailure0104::decode(&frame.payload)
                    .map_err(|error| error.to_string())?;
                if self.pending_npc.take().is_some() {
                    return Ok(Some(LocalizedText::new(
                        "ui.nano.recall.registration_failed",
                        "Unable to register this Recall Point.",
                    )));
                }
            }
            _ => {}
        }
        Ok(None)
    }
}

pub(super) fn notice(messages: &mut NanocomMessageUiModel, body: LocalizedText) {
    messages.enqueue_type_9_localized_optional_icon(
        LocalizedText::new("ui.nano.recall.title", "Recall"),
        body,
        None,
        None,
    );
}

#[cfg(test)]
mod tests;
