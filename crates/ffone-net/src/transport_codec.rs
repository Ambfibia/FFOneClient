use super::*;

/// Exact blocking TCP framing. Every packet uses `read_exact` for the four-byte length and body.
pub struct TcpFrameIo {
    pub(super) stream: TcpStream,
    pub(super) encoder: LegacyClientEncoder,
}

impl TcpFrameIo {
    pub fn connect<A: ToSocketAddrs>(address: A) -> Result<Self> {
        let mut last_error = None;
        for endpoint in address.to_socket_addrs()? {
            match TcpStream::connect_timeout(&endpoint, DEFAULT_CONNECT_TIMEOUT) {
                Ok(stream) => {
                    let io = Self::from_stream(stream)?;
                    io.set_timeouts(Some(DEFAULT_IO_TIMEOUT), Some(DEFAULT_IO_TIMEOUT))?;
                    return Ok(io);
                }
                Err(error) => last_error = Some(error),
            }
        }
        Err(NetError::Io(last_error.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "network address resolved to no endpoints",
            )
        })))
    }

    pub fn from_stream(stream: TcpStream) -> Result<Self> {
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            encoder: LegacyClientEncoder::new(),
        })
    }

    pub fn set_timeouts(
        &self,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    ) -> Result<()> {
        self.stream.set_read_timeout(read_timeout)?;
        self.stream.set_write_timeout(write_timeout)?;
        Ok(())
    }

    pub fn send_payload<P: WirePayload>(
        &mut self,
        packet_type: u32,
        payload: &P,
        key: u64,
    ) -> Result<()> {
        self.send_bytes(packet_type, &payload.encode(), key)
    }

    pub fn send_empty(&mut self, packet_type: u32, key: u64) -> Result<()> {
        self.send_bytes(packet_type, &[], key)
    }

    pub fn send_bytes(&mut self, packet_type: u32, payload: &[u8], key: u64) -> Result<()> {
        let frame = self.encoder.encode(packet_type, payload, key)?;
        self.stream.write_all(&frame)?;
        Ok(())
    }

    pub fn read_encrypted_frame(&mut self) -> Result<Vec<u8>> {
        let mut length = [0u8; 4];
        self.stream.read_exact(&mut length)?;
        let body_len = u32::from_le_bytes(length) as usize;
        if !(4..=MAX_BODY_SIZE_0104).contains(&body_len) {
            return Err(NetError::Frame(if body_len < 4 {
                FrameError::InvalidBodyLength { declared: body_len }
            } else {
                FrameError::BodyTooLarge {
                    declared: body_len,
                    maximum: MAX_BODY_SIZE_0104,
                }
            }));
        }
        let mut frame = vec![0u8; body_len + 4];
        frame[..4].copy_from_slice(&length);
        self.stream.read_exact(&mut frame[4..])?;
        Ok(frame)
    }

    pub fn read_server_frame(&mut self, key: u64) -> Result<DecodedFrame> {
        let encrypted = self.read_encrypted_frame()?;
        Ok(decode_server_frame(&encrypted, key)?)
    }

    pub fn sequence(&self) -> u16 {
        self.encoder.sequence()
    }

    pub fn shutdown(&self) -> Result<()> {
        self.stream.shutdown(Shutdown::Both)?;
        Ok(())
    }

    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}

pub(super) fn decode_server_announcement(payload: &[u8]) -> String {
    let units = payload
        .chunks_exact(2)
        .take(512)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .take_while(|unit| *unit != 0)
        .collect::<Vec<_>>();
    let message = String::from_utf16_lossy(&units);
    if message.trim().is_empty() {
        "OpenFusion returned an empty server announcement".to_owned()
    } else {
        message
    }
}

pub(super) fn shard_ticket_from_login_frame(
    login_id: &FixedUtf16<33>,
    fe_key: u64,
    pc_uid: i64,
    frame: &DecodedFrame,
) -> Result<ShardTicket> {
    let selection = match frame.packet_type {
        packet::P_LS2CL_REP_SHARD_SELECT_SUCC => ShardSelectSuccess::decode(&frame.payload)?,
        packet::P_LS2CL_REP_SHARD_SELECT_FAIL => {
            let failure = ShardSelectFailure::decode(&frame.payload)?;
            return Err(NetError::ShardSelectRejected {
                error_code: failure.error_code,
            });
        }
        packet::P_LS2CL_REP_CHAR_SELECT_FAIL => return Err(character_select_rejected_0104(frame)),
        packet_type => {
            return Err(NetError::UnexpectedPacket {
                phase: "shard selection",
                packet_type,
            });
        }
    };
    let ip_text = selection.server_ip_string();
    let ip = ip_text
        .parse::<IpAddr>()
        .map_err(|_| NetError::InvalidShardEndpoint {
            ip: ip_text.clone(),
            port: selection.server_port,
        })?;
    let port =
        u16::try_from(selection.server_port).map_err(|_| NetError::InvalidShardEndpoint {
            ip: ip_text,
            port: selection.server_port,
        })?;
    Ok(ShardTicket {
        endpoint: SocketAddr::new(ip, port),
        login_id: login_id.clone(),
        pc_uid,
        enter_serial_key: selection.enter_serial_key,
        fe_key,
    })
}

/// One ordered pre-loading packet, with initial entity buckets decoded when possible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldBootstrapPacket {
    InitialEntities {
        /// Original frame, including reserved AROUND bytes not represented by typed entities.
        frame: DecodedFrame,
        entities: InitialAroundPacket0104,
    },
    MalformedInitialEntities {
        /// Original frame is retained even when its payload cannot be decoded.
        frame: DecodedFrame,
        error: AroundDecodeError,
    },
    /// Any non-AROUND packet is passed through byte-for-byte.
    Passthrough(DecodedFrame),
}

impl WorldBootstrapPacket {
    pub(super) fn decode(frame: DecodedFrame) -> Self {
        if !is_initial_around_packet(frame.packet_type) {
            return Self::Passthrough(frame);
        }

        match decode_initial_around_0104(frame.packet_type, &frame.payload) {
            Ok(entities) => Self::InitialEntities { frame, entities },
            Err(error) => Self::MalformedInitialEntities { frame, error },
        }
    }

    pub fn raw_frame(&self) -> &DecodedFrame {
        match self {
            Self::InitialEntities { frame, .. }
            | Self::MalformedInitialEntities { frame, .. }
            | Self::Passthrough(frame) => frame,
        }
    }

    pub fn is_malformed_initial_entities(&self) -> bool {
        matches!(self, Self::MalformedInitialEntities { .. })
    }
}

pub(super) fn is_initial_around_packet(packet_type: u32) -> bool {
    matches!(
        packet_type,
        packet::P_FE2CL_PC_AROUND
            | packet::P_FE2CL_NPC_AROUND
            | packet::P_FE2CL_TRANSPORTATION_AROUND
            | packet::P_FE2CL_SHINY_AROUND
    )
}

pub(super) fn heartbeat_payload(frame: &DecodedFrame) -> Result<&[u8]> {
    if frame.payload.len() != 4 {
        return Err(NetError::Payload(PayloadError::WrongSize {
            expected: 4,
            actual: frame.payload.len(),
        }));
    }
    Ok(&frame.payload)
}
