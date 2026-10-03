use super::*;

pub struct LoginSession {
    pub(super) io: TcpFrameIo,
    pub(super) login: LoginSuccess,
    pub(super) characters: Vec<CharacterInfo0104>,
    pub(super) checked_name: Option<(CharacterNameCheckRequest0104, CharacterNameCheckSuccess0104)>,
    pub(super) pending_creation: Option<CharacterNameSaveSuccess0104>,
    pub(super) e_key: u64,
    pub(super) fe_key: u64,
}

impl LoginSession {
    /// Ask the login server to disconnect an already logged-in session for the
    /// supplied account. OpenFusion's pinned handler is deliberately
    /// fire-and-forget and currently declares no reliable response.
    pub fn request_duplicate_exit<A: ToSocketAddrs>(
        address: A,
        username: &str,
        password: &str,
    ) -> Result<()> {
        let mut io = TcpFrameIo::connect(address)?;
        let request = DuplicateExitRequest0104 {
            id: FixedUtf16::from_str(username)?,
            password: FixedUtf16::from_str(password)?,
        };
        io.send_payload(packet::P_CL2LS_REQ_PC_EXIT_DUPLICATE, &request, DEFAULT_KEY)?;
        io.shutdown()
    }

    /// Connect and perform the legacy password login using client version 1.0.44.
    pub fn connect_password<A: ToSocketAddrs>(
        address: A,
        username: &str,
        password: &str,
    ) -> Result<Self> {
        let mut io = TcpFrameIo::connect(address)?;
        let request = LoginRequest::password_login(username, password, 1, 0, 44)?;
        io.send_payload(packet::P_CL2LS_REQ_LOGIN, &request, DEFAULT_KEY)?;

        let first = loop {
            let frame = io.read_server_frame(DEFAULT_KEY)?;
            if frame.packet_type == packet::P_LS2CL_REQ_LIVE_CHECK {
                io.send_bytes(
                    packet::P_CL2LS_REP_LIVE_CHECK,
                    heartbeat_payload(&frame)?,
                    DEFAULT_KEY,
                )?;
                continue;
            }
            break frame;
        };
        let login = match first.packet_type {
            packet::P_LS2CL_REP_LOGIN_SUCC => LoginSuccess::decode(&first.payload)?,
            packet::P_LS2CL_REP_LOGIN_FAIL => {
                let failure = LoginFailure::decode(&first.payload)?;
                return Err(NetError::LoginRejected {
                    error_code: failure.error_code,
                });
            }
            0x3100_00c8 => {
                return Err(NetError::ServerAnnouncement(decode_server_announcement(
                    &first.payload,
                )));
            }
            packet_type => {
                return Err(NetError::UnexpectedPacket {
                    phase: "login",
                    packet_type,
                });
            }
        };

        let character_count = usize::try_from(login.character_count)
            .map_err(|_| NetError::InvalidCharacterCount(login.character_count))?;
        let e_key = derive_login_e_key(
            login.server_time,
            login.character_count,
            login.selected_slot,
        );
        let fe_key = derive_frontend_key(44);
        let mut session = Self {
            io,
            login,
            characters: Vec::with_capacity(character_count),
            checked_name: None,
            pending_creation: None,
            e_key,
            fe_key,
        };
        while session.characters.len() < character_count {
            let frame = session.read_next()?;
            if frame.packet_type != packet::P_LS2CL_REP_CHAR_INFO {
                return Err(NetError::UnexpectedPacket {
                    phase: "character list",
                    packet_type: frame.packet_type,
                });
            }
            session
                .characters
                .push(CharacterInfo0104::decode(&frame.payload)?);
        }
        Ok(session)
    }

    pub fn login_success(&self) -> &LoginSuccess {
        &self.login
    }

    pub fn characters(&self) -> &[CharacterInfo0104] {
        &self.characters
    }

    /// Perform the first packet in the legacy three-stage creation handshake.
    ///
    /// OpenFusion currently echoes success here and performs its authoritative name validation in
    /// [`Self::save_character_name`]. The echo is still checked byte-for-byte so a stale or
    /// cross-request response cannot advance this session.
    pub fn check_character_name(
        &mut self,
        request: &CharacterNameCheckRequest0104,
    ) -> Result<CharacterNameCheckSuccess0104> {
        if self.pending_creation.is_some() {
            return Err(NetError::CharacterCreationState {
                reason: "finish or delete the pending character before checking another name",
            });
        }
        self.checked_name = None;
        self.io
            .send_payload(packet::P_CL2LS_REQ_CHECK_CHAR_NAME, request, self.e_key)?;
        let frame = self.read_next()?;
        let checked = match frame.packet_type {
            packet::P_LS2CL_REP_CHECK_CHAR_NAME_SUCC => {
                CharacterNameCheckSuccess0104::decode(&frame.payload)?
            }
            packet::P_LS2CL_REP_CHECK_CHAR_NAME_FAIL => {
                let failure = CharacterNameCheckFailure0104::decode(&frame.payload)?;
                return Err(NetError::CharacterNameCheckRejected {
                    error_code: failure.error_code,
                });
            }
            packet_type => {
                return Err(NetError::UnexpectedPacket {
                    phase: "character name check",
                    packet_type,
                });
            }
        };
        if checked.first_name != request.first_name || checked.last_name != request.last_name {
            return Err(NetError::CharacterResponseMismatch {
                phase: "character name check",
            });
        }
        self.checked_name = Some((request.clone(), checked.clone()));
        Ok(checked)
    }

    /// Reserve the checked name and create the account-owned, unfinished character row.
    ///
    /// OpenFusion validates the name in this phase. Its current failure path uses either the
    /// declared SAVE/CHECK failure packets or `P_LS2CL_REP_SHARD_SELECT_FAIL` for an invalid slot;
    /// all three are mapped to a typed name-save rejection.
    pub fn save_character_name(
        &mut self,
        request: &CharacterNameSaveRequest0104,
    ) -> Result<CharacterNameSaveSuccess0104> {
        if self.pending_creation.is_some() {
            return Err(NetError::CharacterCreationState {
                reason: "finish or delete the pending character before saving another name",
            });
        }
        let Some((checked_request, checked_response)) = self.checked_name.as_ref() else {
            return Err(NetError::CharacterCreationState {
                reason: "name save requires a successful name check",
            });
        };
        if checked_request.first_name_code != request.first_name_code
            || checked_request.last_name_code != request.last_name_code
            || checked_request.middle_name_code != request.middle_name_code
            || checked_response.first_name != request.first_name
            || checked_response.last_name != request.last_name
        {
            return Err(NetError::CharacterCreationState {
                reason: "name save does not match the last checked name",
            });
        }
        self.io
            .send_payload(packet::P_CL2LS_REQ_SAVE_CHAR_NAME, request, self.e_key)?;
        let frame = self.read_next()?;
        let saved = match frame.packet_type {
            packet::P_LS2CL_REP_SAVE_CHAR_NAME_SUCC => {
                CharacterNameSaveSuccess0104::decode(&frame.payload)?
            }
            packet::P_LS2CL_REP_CHECK_CHAR_NAME_FAIL => {
                let failure = CharacterNameCheckFailure0104::decode(&frame.payload)?;
                return Err(NetError::CharacterNameSaveRejected {
                    error_code: failure.error_code,
                });
            }
            packet::P_LS2CL_REP_SAVE_CHAR_NAME_FAIL => {
                let failure = CharacterNameSaveFailure0104::decode(&frame.payload)?;
                return Err(NetError::CharacterNameSaveRejected {
                    error_code: failure.error_code,
                });
            }
            packet::P_LS2CL_REP_SHARD_SELECT_FAIL => {
                let failure = ShardSelectFailure::decode(&frame.payload)?;
                return Err(NetError::CharacterNameSaveRejected {
                    error_code: failure.error_code,
                });
            }
            packet_type => {
                return Err(NetError::UnexpectedPacket {
                    phase: "character name save",
                    packet_type,
                });
            }
        };
        if saved.slot != request.slot
            || saved.gender != request.gender
            || saved.first_name != request.first_name
            || saved.last_name != request.last_name
        {
            return Err(NetError::CharacterResponseMismatch {
                phase: "character name save",
            });
        }
        self.checked_name = None;
        self.pending_creation = Some(saved.clone());
        Ok(saved)
    }

    /// Finalize appearance and starter equipment for an unfinished character.
    ///
    /// The request is accepted only for the UID just returned by name save, or for an unfinished
    /// (`appearance_flag == 0`) character returned by the initial login roster. This preserves the
    /// official creation ordering while allowing a reconnect to resume the constructor.
    pub fn create_character(
        &mut self,
        request: &CharacterCreateRequest0104,
    ) -> Result<CharacterCreateSuccess0104> {
        let slot = if let Some(saved) = self.pending_creation.as_ref() {
            if saved.pc_uid != request.style.pc_uid
                || saved.first_name != request.style.first_name
                || saved.last_name != request.style.last_name
            {
                return Err(NetError::CharacterCreationState {
                    reason: "appearance does not match the name-save identity",
                });
            }
            saved.slot
        } else if let Some(character) = self
            .characters
            .iter()
            .find(|character| character.pc_uid() == request.style.pc_uid)
        {
            if character.style().appearance_flag != 0
                || character.first_name() != request.style.first_name
                || character.last_name() != request.style.last_name
            {
                return Err(NetError::CharacterCreationState {
                    reason: "only the matching unfinished roster character can resume creation",
                });
            }
            character.slot()
        } else {
            return Err(NetError::CharacterCreationState {
                reason: "appearance requires a prior name save or unfinished roster character",
            });
        };

        self.io
            .send_payload(packet::P_CL2LS_REQ_CHAR_CREATE, request, self.e_key)?;
        let frame = self.read_next()?;
        let created = match frame.packet_type {
            packet::P_LS2CL_REP_CHAR_CREATE_SUCC => {
                CharacterCreateSuccess0104::decode(&frame.payload)?
            }
            packet::P_LS2CL_REP_CHAR_CREATE_FAIL => {
                let failure = CharacterCreateFailure0104::decode(&frame.payload)?;
                return Err(NetError::CharacterCreateRejected {
                    error_code: failure.error_code,
                });
            }
            packet::P_LS2CL_REP_SHARD_SELECT_FAIL => {
                let failure = ShardSelectFailure::decode(&frame.payload)?;
                return Err(NetError::CharacterCreateRejected {
                    error_code: failure.error_code,
                });
            }
            packet_type => {
                return Err(NetError::UnexpectedPacket {
                    phase: "character create",
                    packet_type,
                });
            }
        };
        if created.style.pc_uid != request.style.pc_uid
            || created.style.first_name != request.style.first_name
            || created.style.last_name != request.style.last_name
            || created.style.gender != request.style.gender
        {
            return Err(NetError::CharacterResponseMismatch {
                phase: "character create",
            });
        }
        self.upsert_created_character(slot, &created);
        self.pending_creation = None;
        Ok(created)
    }

    /// Delete one character by UID.
    ///
    /// Protocol 0104 carries only the UID. Account authorization is performed by OpenFusion using
    /// the authenticated login socket; no password, PIN, or confirmation code exists on this wire
    /// request.
    pub fn delete_character(&mut self, pc_uid: i64) -> Result<CharacterDeleteSuccess0104> {
        let expected_slot = self
            .characters
            .iter()
            .find(|character| character.pc_uid() == pc_uid)
            .map(CharacterInfo0104::slot)
            .or_else(|| {
                self.pending_creation
                    .as_ref()
                    .filter(|pending| pending.pc_uid == pc_uid)
                    .map(|pending| pending.slot)
            })
            .ok_or(NetError::UnknownCharacter(pc_uid))?;

        self.io.send_payload(
            packet::P_CL2LS_REQ_CHAR_DELETE,
            &CharacterDeleteRequest0104 { pc_uid },
            self.e_key,
        )?;
        let frame = self.read_next()?;
        let deleted = match frame.packet_type {
            packet::P_LS2CL_REP_CHAR_DELETE_SUCC => {
                CharacterDeleteSuccess0104::decode(&frame.payload)?
            }
            packet::P_LS2CL_REP_CHAR_DELETE_FAIL => {
                let failure = CharacterDeleteFailure0104::decode(&frame.payload)?;
                return Err(NetError::CharacterDeleteRejected {
                    error_code: failure.error_code,
                });
            }
            packet::P_LS2CL_REP_SHARD_SELECT_FAIL => {
                let failure = ShardSelectFailure::decode(&frame.payload)?;
                return Err(NetError::CharacterDeleteRejected {
                    error_code: failure.error_code,
                });
            }
            packet_type => {
                return Err(NetError::UnexpectedPacket {
                    phase: "character delete",
                    packet_type,
                });
            }
        };
        if deleted.slot != expected_slot {
            return Err(NetError::CharacterResponseMismatch {
                phase: "character delete",
            });
        }
        self.characters
            .retain(|character| character.pc_uid() != pc_uid);
        if self
            .pending_creation
            .as_ref()
            .is_some_and(|pending| pending.pc_uid == pc_uid)
        {
            self.pending_creation = None;
        }
        Ok(deleted)
    }

    /// Change one account-owned character name and update the retained login
    /// roster only after an identity-matched success response.
    pub fn change_character_name(
        &mut self,
        request: &CharacterNameChangeRequest0104,
    ) -> Result<CharacterNameChangeSuccess0104> {
        let character = self
            .characters
            .iter()
            .find(|character| character.pc_uid() == request.pc_uid)
            .ok_or(NetError::UnknownCharacter(request.pc_uid))?;
        if character.slot() != request.slot {
            return Err(NetError::CharacterCreationState {
                reason: "rename slot does not match the login roster",
            });
        }

        self.io
            .send_payload(packet::P_CL2LS_REQ_CHANGE_CHAR_NAME, request, self.e_key)?;
        let frame = self.read_next()?;
        let renamed = match frame.packet_type {
            packet::P_LS2CL_REP_CHANGE_CHAR_NAME_SUCC => {
                CharacterNameChangeSuccess0104::decode(&frame.payload)?
            }
            // The pinned OpenFusion handler uses CHECK_CHAR_NAME_FAIL for its
            // validation failures even though CHANGE_CHAR_NAME_FAIL exists in
            // the generated 0104 protocol.
            packet::P_LS2CL_REP_CHECK_CHAR_NAME_FAIL => {
                let failure = CharacterNameCheckFailure0104::decode(&frame.payload)?;
                return Err(NetError::CharacterNameChangeRejected {
                    error_code: failure.error_code,
                });
            }
            packet::P_LS2CL_REP_CHANGE_CHAR_NAME_FAIL => {
                let failure = CharacterNameChangeFailure0104::decode(&frame.payload)?;
                return Err(NetError::CharacterNameChangeRejected {
                    error_code: failure.error_code,
                });
            }
            packet::P_LS2CL_REP_SHARD_SELECT_FAIL => {
                let failure = ShardSelectFailure::decode(&frame.payload)?;
                return Err(NetError::CharacterNameChangeRejected {
                    error_code: failure.error_code,
                });
            }
            packet_type => {
                return Err(NetError::UnexpectedPacket {
                    phase: "character rename",
                    packet_type,
                });
            }
        };
        if renamed.pc_uid != request.pc_uid
            || renamed.slot != request.slot
            || renamed.first_name != request.first_name
            || renamed.last_name != request.last_name
        {
            return Err(NetError::CharacterResponseMismatch {
                phase: "character rename",
            });
        }

        let roster = self
            .characters
            .iter_mut()
            .find(|character| character.pc_uid() == request.pc_uid)
            .expect("character checked before request");
        roster.as_bytes_mut()[66] = request.gender as u8;
        for (index, unit) in renamed.first_name.as_units().iter().enumerate() {
            let offset = 14 + index * 2;
            roster.as_bytes_mut()[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
        }
        for (index, unit) in renamed.last_name.as_units().iter().enumerate() {
            let offset = 32 + index * 2;
            roster.as_bytes_mut()[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
        }
        Ok(renamed)
    }

    pub fn first_character_uid(&self) -> Result<i64> {
        self.characters
            .first()
            .map(CharacterInfo0104::pc_uid)
            .ok_or(NetError::NoCharacters)
    }

    pub(super) fn upsert_created_character(&mut self, slot: i8, created: &CharacterCreateSuccess0104) {
        let existing_position = self
            .characters
            .iter()
            .find(|character| character.pc_uid() == created.style.pc_uid)
            .map(CharacterInfo0104::position)
            .unwrap_or([0; 3]);
        let response = created.encode();
        let mut roster = CharacterInfo0104::zeroed();
        roster.as_bytes_mut()[0] = slot as u8;
        roster.as_bytes_mut()[2..4].copy_from_slice(&created.level.to_le_bytes());
        roster.as_bytes_mut()[4..83].copy_from_slice(&response[4..83]);
        for (index, coordinate) in existing_position.into_iter().enumerate() {
            let offset = 84 + index * 4;
            roster.as_bytes_mut()[offset..offset + 4].copy_from_slice(&coordinate.to_le_bytes());
        }
        for (slot_index, item_id) in [
            created.equipped.hand_id,
            created.equipped.upper_body_id,
            created.equipped.lower_body_id,
            created.equipped.foot_id,
            created.equipped.head_id,
            created.equipped.face_id,
            created.equipped.back_id,
        ]
        .into_iter()
        .enumerate()
        {
            if item_id == 0 {
                continue;
            }
            let offset = 96 + slot_index * 12;
            roster.as_bytes_mut()[offset..offset + 2]
                .copy_from_slice(&(slot_index as i16).to_le_bytes());
            roster.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&item_id.to_le_bytes());
            roster.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&1i32.to_le_bytes());
        }
        if let Some(existing) = self
            .characters
            .iter_mut()
            .find(|character| character.pc_uid() == created.style.pc_uid)
        {
            *existing = roster;
        } else {
            self.characters.push(roster);
        }
        self.characters.sort_by_key(CharacterInfo0104::slot);
    }

    /// Keep the menu roster's location in sync with a shard entry snapshot.
    /// Creation replies carry no coordinates, so a new entry starts at zero.
    pub fn observe_character_position(&mut self, pc_uid: i64, position: [i32; 3]) {
        if let Some(character) = self
            .characters
            .iter_mut()
            .find(|character| character.pc_uid() == pc_uid)
        {
            for (index, coordinate) in position.into_iter().enumerate() {
                let offset = 84 + index * 4;
                character.as_bytes_mut()[offset..offset + 4].copy_from_slice(&coordinate.to_le_bytes());
            }
        }
    }

    /// Service an idle menu connection without waiting for the next UI command.
    /// Peek never consumes a partial frame. Once data arrives, the normal blocking
    /// frame reader retains ownership through the whole packet and its timeout.
    pub fn poll_idle(&mut self) -> Result<Option<DecodedFrame>> {
        self.io.stream.set_nonblocking(true)?;
        let available = self.io.stream.peek(&mut [0u8; 1]);
        self.io.stream.set_nonblocking(false)?;
        match available {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(None),
            Err(error) => return Err(error.into()),
            Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "login server closed the connection").into()),
            Ok(_) => {}
        }
        let frame = self.io.read_server_frame(self.e_key)?;
        if self.respond_to_heartbeat(&frame)? {
            Ok(None)
        } else {
            Ok(Some(frame))
        }
    }

    /// Read one non-heartbeat login packet. Heartbeats use the current login E key.
    pub fn read_next(&mut self) -> Result<DecodedFrame> {
        loop {
            let frame = self.io.read_server_frame(self.e_key)?;
            if self.respond_to_heartbeat(&frame)? {
                continue;
            }
            return Ok(frame);
        }
    }

    pub fn respond_to_heartbeat(&mut self, frame: &DecodedFrame) -> Result<bool> {
        if frame.packet_type != packet::P_LS2CL_REQ_LIVE_CHECK {
            return Ok(false);
        }
        self.io.send_bytes(
            packet::P_CL2LS_REP_LIVE_CHECK,
            heartbeat_payload(frame)?,
            self.e_key,
        )?;
        Ok(true)
    }

    /// Fire-and-forget tutorial completion before character/shard selection.
    ///
    /// This is the path used by a newly created character: the reference client
    /// runs the tutorial locally while the original login session is still
    /// active, then sends `SAVE_CHAR_TUTOR` immediately before `CHAR_SELECT`.
    pub fn send_tutorial_completion(
        &mut self,
        request: &CharacterTutorialSaveRequest0104,
    ) -> Result<()> {
        if !self
            .characters
            .iter()
            .any(|character| character.pc_uid() == request.pc_uid)
        {
            return Err(NetError::UnknownCharacter(request.pc_uid));
        }
        self.io
            .send_payload(packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR, request, self.e_key)?;
        // OpenFusion deliberately has no success response for this packet. Once
        // the complete frame has been written, mirror the fire-and-forget state
        // in the retained login roster so a later shard-selection failure can
        // return to character selection without relaunching the tutorial.
        if let Some(character) = self
            .characters
            .iter_mut()
            .find(|character| character.pc_uid() == request.pc_uid)
        {
            character.as_bytes_mut()[81] = request.tutorial_flag as u8;
        }
        Ok(())
    }

    pub fn select_character(&mut self, pc_uid: i64) -> Result<ShardTicket> {
        if !self
            .characters
            .iter()
            .any(|character| character.pc_uid() == pc_uid)
        {
            return Err(NetError::UnknownCharacter(pc_uid));
        }
        self.io.send_payload(
            packet::P_CL2LS_REQ_CHAR_SELECT,
            &CharacterSelectRequest { pc_uid },
            self.e_key,
        )?;
        let frame = self.read_next()?;
        let frame = self.follow_character_select_0104(frame)?;
        shard_ticket_from_login_frame(&self.login.id, self.fe_key, pc_uid, &frame)
    }

    /// The clean login server answers `CHAR_SELECT` with `CHAR_SELECT_SUCC`
    /// and waits for `SHARD_SELECT` before it sends the shard ticket
    /// (`CnCharSelectionMode.ReceivePacket`, case 553648139). OpenFusion skips
    /// straight to `SHARD_SELECT_SUCC`. Both routes end on the same
    /// shard-selection frame, which the caller decodes.
    pub(super) fn follow_character_select_0104(&mut self, frame: DecodedFrame) -> Result<DecodedFrame> {
        match frame.packet_type {
            packet::P_LS2CL_REP_CHAR_SELECT_SUCC => {
                self.io.send_payload(
                    packet::P_CL2LS_REQ_SHARD_SELECT,
                    &clean_shard_select_request_0104(),
                    self.e_key,
                )?;
                self.read_next()
            }
            packet::P_LS2CL_REP_CHAR_SELECT_FAIL => Err(character_select_rejected_0104(&frame)),
            _ => Ok(frame),
        }
    }

    /// Preserve the selected login-server socket while gameplay runs.
    ///
    /// OpenFusion expects tutorial completion on this socket, not on the shard socket. The
    /// receiver owns a cloned read handle while all writes share the original encoder, preserving
    /// the legacy outbound packet sequence across selection, heartbeat replies, and tutorial save.
    pub fn into_world_connection(self) -> Result<LoginWorldConnection> {
        let reader_stream = self.io.stream.try_clone()?;
        let sender = LoginWorldSender {
            inner: Arc::new(Mutex::new(LoginWorldWriter {
                io: self.io,
                outbound_e_key: self.e_key,
                login_id: self.login.id,
                fe_key: self.fe_key,
            })),
        };
        let receiver = LoginWorldReceiver {
            io: TcpFrameIo::from_stream(reader_stream)?,
            inbound_e_key: self.e_key,
            sender: sender.clone(),
        };
        Ok(LoginWorldConnection { sender, receiver })
    }
}

pub struct ShardSession {
    pub(super) io: TcpFrameIo,
    pub(super) player_id: i32,
    pub(super) load: PcLoadData0104,
    /// Exact `PC_ENTER_SUCC.uiSvrTime` observed from the shard.
    pub(super) server_time: u64,
    /// Client-to-shard traffic is encrypted with E after PC_ENTER_SUCC.
    pub(super) outbound_e_key: u64,
    /// OpenFusion keeps all post-enter shard responses on FE.
    pub(super) inbound_fe_key: u64,
    /// FE-encrypted bootstrap packets OpenFusion sends before PC_ENTER_SUCC.
    pub(super) entry_prelude: Vec<DecodedFrame>,
}
