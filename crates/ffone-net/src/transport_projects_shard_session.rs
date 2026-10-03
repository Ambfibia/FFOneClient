use super::*;

impl ShardSession {
    pub fn connect(ticket: ShardTicket) -> Result<Self> {
        let mut io = TcpFrameIo::connect(ticket.endpoint)?;
        io.send_payload(
            packet::P_CL2FE_REQ_PC_ENTER,
            &PcEnterRequest {
                id: ticket.login_id,
                temporary_value: 0,
                enter_serial_key: ticket.enter_serial_key,
            },
            DEFAULT_KEY,
        )?;

        // OpenFusion first sends a resize-only Nano Book packet with FE, then PC_ENTER_SUCC, then
        // the populated Nano Book subsets. Invalid-serial failures are emitted earlier with the
        // default E key, so failed FE decoding must retain that fallback.
        let mut entry_prelude = Vec::new();
        let success = loop {
            let encrypted = io.read_encrypted_frame()?;
            match decode_server_frame(&encrypted, ticket.fe_key) {
                Ok(frame) if frame.packet_type == packet::P_FE2CL_REP_PC_ENTER_SUCC => {
                    break PcEnterSuccess::decode(&frame.payload)?;
                }
                Ok(frame) if frame.packet_type == packet::P_FE2CL_REP_PC_ENTER_FAIL => {
                    let failure = PcEnterFailure::decode(&frame.payload)?;
                    return Err(NetError::PcEnterRejected {
                        error_code: failure.error_code,
                    });
                }
                Ok(frame) if frame.packet_type == packet::P_FE2CL_REP_NANO_BOOK_SUBSET => {
                    if frame.payload.len() != 76 {
                        return Err(NetError::Payload(PayloadError::WrongSize {
                            expected: 76,
                            actual: frame.payload.len(),
                        }));
                    }
                    entry_prelude.push(frame);
                }
                Ok(fe_frame) => match decode_server_frame(&encrypted, DEFAULT_KEY) {
                    Ok(frame) if frame.packet_type == packet::P_FE2CL_REP_PC_ENTER_FAIL => {
                        let failure = PcEnterFailure::decode(&frame.payload)?;
                        return Err(NetError::PcEnterRejected {
                            error_code: failure.error_code,
                        });
                    }
                    _ => {
                        return Err(NetError::UnexpectedPacket {
                            phase: "shard entry",
                            packet_type: fe_frame.packet_type,
                        });
                    }
                },
                Err(fe_error) => match decode_server_frame(&encrypted, DEFAULT_KEY) {
                    Ok(frame) if frame.packet_type == packet::P_FE2CL_REP_PC_ENTER_FAIL => {
                        let failure = PcEnterFailure::decode(&frame.payload)?;
                        return Err(NetError::PcEnterRejected {
                            error_code: failure.error_code,
                        });
                    }
                    Ok(frame) => {
                        return Err(NetError::UnexpectedPacket {
                            phase: "shard entry",
                            packet_type: frame.packet_type,
                        });
                    }
                    Err(_) => return Err(NetError::Frame(fe_error)),
                },
            }
        };
        let outbound_e_key = derive_shard_e_key(
            success.server_time,
            success.id,
            success.load.fusion_matter(),
        );
        Ok(Self {
            io,
            player_id: success.id,
            load: success.load,
            server_time: success.server_time,
            outbound_e_key,
            inbound_fe_key: ticket.fe_key,
            entry_prelude,
        })
    }

    pub fn player_id(&self) -> i32 {
        self.player_id
    }

    pub fn load_data(&self) -> &PcLoadData0104 {
        &self.load
    }

    pub fn server_time(&self) -> u64 {
        self.server_time
    }

    pub fn entry_prelude(&self) -> &[DecodedFrame] {
        &self.entry_prelude
    }

    /// Set gameplay socket timeouts. A short read timeout lets an engine worker interleave
    /// incoming packets with its command queue without blocking the render thread.
    pub fn set_timeouts(
        &self,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    ) -> Result<()> {
        self.io.set_timeouts(read_timeout, write_timeout)
    }

    pub fn send_move(&mut self, request: &PcMoveRequest0104) -> Result<()> {
        self.io
            .send_payload(packet::P_CL2FE_REQ_PC_MOVE, request, self.outbound_e_key)
    }

    pub fn send_stop(&mut self, request: &PcStopRequest0104) -> Result<()> {
        self.io
            .send_payload(packet::P_CL2FE_REQ_PC_STOP, request, self.outbound_e_key)
    }

    pub fn send_jump(&mut self, request: &PcJumpRequest0104) -> Result<()> {
        self.io
            .send_payload(packet::P_CL2FE_REQ_PC_JUMP, request, self.outbound_e_key)
    }

    pub fn shutdown(&self) -> Result<()> {
        self.io.shutdown()
    }

    /// Split a loaded shard connection into a blocking receiver and a cloneable synchronized
    /// sender. All outbound gameplay packets (including heartbeat replies) share one legacy
    /// encoder, so packet sequence numbers cannot diverge between engine systems.
    pub fn into_gameplay(self) -> Result<GameplayConnection> {
        let reader_stream = self.io.stream.try_clone()?;
        let sender = GameplaySender {
            inner: Arc::new(Mutex::new(GameplayWriter {
                io: self.io,
                outbound_e_key: self.outbound_e_key,
            })),
        };
        let receiver = GameplayReceiver {
            io: TcpFrameIo::from_stream(reader_stream)?,
            inbound_fe_key: self.inbound_fe_key,
            sender: sender.clone(),
        };
        Ok(GameplayConnection {
            player_id: self.player_id,
            load: self.load,
            sender,
            receiver,
        })
    }

    /// Send the exact legacy loading-complete body (`iPC_ID = 0`) and wait for its response.
    ///
    /// OpenFusion populates the player's chunks before sending the success packet, so initial
    /// PC/NPC/transportation/shiny AROUND packets are returned losslessly in `prelude`.
    pub fn complete_loading(&mut self) -> Result<LoadingComplete> {
        self.io.send_payload(
            packet::P_CL2FE_REQ_PC_LOADING_COMPLETE,
            &PcLoadingCompleteRequest { pc_id: 0 },
            self.outbound_e_key,
        )?;
        let mut prelude = std::mem::take(&mut self.entry_prelude);
        loop {
            let frame = self.io.read_server_frame(self.inbound_fe_key)?;
            if self.respond_to_heartbeat(&frame)? {
                continue;
            }
            if frame.packet_type == packet::P_FE2CL_REP_PC_LOADING_COMPLETE_SUCC {
                return Ok(LoadingComplete {
                    response: PcLoadingCompleteSuccess::decode(&frame.payload)?,
                    prelude,
                });
            }
            prelude.push(frame);
        }
    }

    /// Read one non-heartbeat gameplay frame.
    pub fn read_next(&mut self) -> Result<DecodedFrame> {
        loop {
            let frame = self.io.read_server_frame(self.inbound_fe_key)?;
            if self.respond_to_heartbeat(&frame)? {
                continue;
            }
            return Ok(frame);
        }
    }

    pub fn respond_to_heartbeat(&mut self, frame: &DecodedFrame) -> Result<bool> {
        if frame.packet_type != packet::P_FE2CL_REQ_LIVE_CHECK {
            return Ok(false);
        }
        self.io.send_bytes(
            packet::P_CL2FE_REP_LIVE_CHECK,
            heartbeat_payload(frame)?,
            self.outbound_e_key,
        )?;
        Ok(true)
    }
}
