use super::*;

pub(super) const NANOCOM_PASSIVE_REQUEST_ID_BASE: u64 = 0x4E41_4E4F_434D_0000;

#[derive(Clone, Debug, PartialEq)]
pub struct NanocomMessageRequest {
    pub request_id: u64,
    pub kind: NanocomMessageKind,
    pub title: String,
    pub body: String,
    pub compact_frame_path: String,
    /// Exact clean icon request when one exists. Some primary TableData rows
    /// deliberately resolve to an asset that is absent from the clean build;
    /// that must hide only the portrait, never the complete message panel.
    pub compact_icon_path: Option<String>,
    pub lifetime_seconds: f32,
    /// Stable localization identities for source-authored NPC copy. Dynamic
    /// buddy/group content continues to use its keyed templates below.
    pub localized_title: Option<LocalizedText>,
    pub localized_body: Option<LocalizedText>,
    /// Semantic localized voice selected from the NPC's TableData owner.
    pub voice_true_name: Option<String>,
    /// Player-authored display name retained separately so localized copy can
    /// use a keyed template instead of parsing the rendered English sentence.
    pub buddy_name: Option<String>,
}

impl NanocomMessageRequest {
    #[must_use]
    pub fn buddy_invite(request_id: u64, buddy_name: impl AsRef<str>) -> Self {
        let buddy_name = buddy_name.as_ref().to_owned();
        Self {
            request_id,
            kind: NanocomMessageKind::BuddyInvite,
            title: NANOCOM_COMPACT_TITLE.to_owned(),
            body: format!("{buddy_name} has invited you to be buddies."),
            compact_frame_path: NANOCOM_BUDDY_FRAME_PATH.to_owned(),
            compact_icon_path: Some(NANOCOM_BUDDY_ICON_PATH.to_owned()),
            lifetime_seconds: NANOCOM_BUDDY_LIFETIME_SECONDS,
            localized_title: None,
            localized_body: None,
            voice_true_name: None,
            buddy_name: Some(buddy_name),
        }
    }

    #[must_use]
    pub fn group_invite(request_id: u64, group_host_name: impl AsRef<str>) -> Self {
        let group_host_name = group_host_name.as_ref();
        Self {
            request_id,
            kind: NanocomMessageKind::GroupInvite,
            title: NANOCOM_GROUP_COMPACT_TITLE.to_owned(),
            body: format!("{group_host_name}{NANOCOM_GROUP_INVITATION_SUFFIX}"),
            compact_frame_path: NANOCOM_BUDDY_FRAME_PATH.to_owned(),
            compact_icon_path: Some(NANOCOM_GROUP_ICON_PATH.to_owned()),
            lifetime_seconds: NANOCOM_BUDDY_LIFETIME_SECONDS,
            localized_title: None,
            localized_body: None,
            voice_true_name: None,
            buddy_name: None,
        }
    }

    /// Adapter boundary for the existing tutorial/NPC type-9 producer.
    #[must_use]
    pub fn type_9(
        request_id: u64,
        title: impl Into<String>,
        body: impl Into<String>,
        compact_icon_path: impl Into<String>,
    ) -> Self {
        Self {
            request_id,
            kind: NanocomMessageKind::Npc,
            title: title.into(),
            body: body.into(),
            compact_frame_path: NANOCOM_TYPE_9_FRAME_PATH.to_owned(),
            compact_icon_path: Some(compact_icon_path.into()),
            lifetime_seconds: NANOCOM_TYPE_9_LIFETIME_SECONDS,
            localized_title: None,
            localized_body: None,
            voice_true_name: None,
            buddy_name: None,
        }
    }

    #[must_use]
    pub fn type_9_localized(
        request_id: u64,
        title: LocalizedText,
        body: LocalizedText,
        compact_icon_path: impl Into<String>,
        voice_owner: Option<&str>,
    ) -> Self {
        Self::type_9_localized_optional_icon(
            request_id,
            title,
            body,
            Some(compact_icon_path.into()),
            voice_owner,
        )
    }

    #[must_use]
    pub fn type_9_localized_optional_icon(
        request_id: u64,
        title: LocalizedText,
        body: LocalizedText,
        compact_icon_path: Option<String>,
        voice_owner: Option<&str>,
    ) -> Self {
        let voice_true_name = voice_owner
            .filter(|owner| !owner.is_empty())
            .map(|owner| nanocom_comm_out_true_name(owner, request_id));
        Self {
            request_id,
            kind: NanocomMessageKind::Npc,
            title: title.fallback.clone(),
            body: body.fallback.clone(),
            compact_frame_path: NANOCOM_TYPE_9_FRAME_PATH.to_owned(),
            compact_icon_path,
            lifetime_seconds: NANOCOM_TYPE_9_LIFETIME_SECONDS,
            localized_title: Some(title),
            localized_body: Some(body),
            voice_true_name,
            buddy_name: None,
        }
    }

    /// Clean button type 10 used only for the first serialized task of an
    /// assigned Nano mission.
    #[must_use]
    pub fn nano_mission_localized(
        request_id: u64,
        body: LocalizedText,
        compact_icon_path: Option<String>,
        voice_owner: Option<&str>,
    ) -> Self {
        let title = LocalizedText::new(
            NANOCOM_NANO_MISSION_TITLE_LOCALIZATION_KEY,
            NANOCOM_NANO_MISSION_TITLE,
        );
        let voice_true_name = voice_owner
            .filter(|owner| !owner.is_empty())
            .map(|owner| nanocom_comm_out_true_name(owner, request_id));
        Self {
            request_id,
            kind: NanocomMessageKind::Nano,
            title: title.fallback.clone(),
            body: body.fallback.clone(),
            compact_frame_path: NANOCOM_NANO_FRAME_PATH.to_owned(),
            compact_icon_path,
            lifetime_seconds: NANOCOM_NANO_LIFETIME_SECONDS,
            localized_title: Some(title),
            localized_body: Some(body),
            voice_true_name,
            buddy_name: None,
        }
    }

    #[must_use]
    pub const fn compact_frame_rect(&self) -> NanocomRect {
        match self.kind {
            NanocomMessageKind::Nano => NANOCOM_NANO_FRAME_RECT,
            _ => NANOCOM_COMPACT_FRAME_RECT,
        }
    }

    #[must_use]
    pub const fn compact_icon_rect(&self) -> NanocomRect {
        match self.kind {
            NanocomMessageKind::Nano => NANOCOM_NANO_ICON_RECT,
            _ => NANOCOM_COMPACT_ICON_RECT,
        }
    }

    #[must_use]
    pub fn type_9_numbuh_two(
        request_id: u64,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self::type_9(request_id, title, body, NANOCOM_NUMBUH_TWO_ICON_PATH)
    }

    #[must_use]
    pub fn compact_title_localized(&self) -> LocalizedText {
        if let Some(localized) = self.localized_title.as_ref() {
            return localized.clone();
        }
        match self.kind {
            NanocomMessageKind::BuddyInvite => LocalizedText::new(
                NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY,
                NANOCOM_COMPACT_TITLE,
            ),
            NanocomMessageKind::GroupInvite => LocalizedText::new(
                NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY,
                NANOCOM_GROUP_COMPACT_TITLE,
            ),
            _ => nanocom_passthrough(&self.title),
        }
    }

    #[must_use]
    pub fn compact_body_localized(&self, remaining_seconds: f32) -> LocalizedText {
        if let Some(localized) = self.localized_body.as_ref() {
            return localized.clone();
        }
        if let Some(name) = self.buddy_name.as_deref() {
            return LocalizedText::new(
                NANOCOM_COMPACT_BODY_LOCALIZATION_KEY,
                NANOCOM_COMPACT_BODY_FALLBACK,
            )
            .with_arg("name", name)
            .with_arg("seconds", rounded_seconds(remaining_seconds));
        }
        if self.kind == NanocomMessageKind::GroupInvite {
            if let Some(name) = self.clean_group_host_name() {
                return LocalizedText::new(
                    NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY,
                    NANOCOM_GROUP_COMPACT_BODY_FALLBACK,
                )
                .with_arg("name", name)
                .with_arg("seconds", rounded_seconds(remaining_seconds));
            }
        }
        nanocom_passthrough(&self.body)
    }

    #[must_use]
    pub fn expanded_title_localized(&self) -> LocalizedText {
        match self.kind {
            NanocomMessageKind::BuddyInvite => LocalizedText::new(
                NANOCOM_EXPANDED_TITLE_LOCALIZATION_KEY,
                NANOCOM_EXPANDED_TITLE,
            ),
            NanocomMessageKind::GroupInvite => LocalizedText::new(
                NANOCOM_GROUP_EXPANDED_TITLE_LOCALIZATION_KEY,
                NANOCOM_GROUP_EXPANDED_TITLE,
            ),
            _ => nanocom_passthrough(""),
        }
    }

    #[must_use]
    pub fn expanded_body_localized(&self) -> LocalizedText {
        if let Some(name) = self.buddy_name.as_deref() {
            return LocalizedText::new(
                NANOCOM_INVITATION_LOCALIZATION_KEY,
                NANOCOM_INVITATION_FALLBACK,
            )
            .with_arg("name", name);
        }
        if self.kind == NanocomMessageKind::GroupInvite {
            if let Some(name) = self.clean_group_host_name() {
                return LocalizedText::new(
                    NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY,
                    NANOCOM_GROUP_INVITATION_FALLBACK,
                )
                .with_arg("name", name);
            }
        }
        nanocom_passthrough(&self.body)
    }

    #[must_use]
    pub fn expanded_expiration_localized(&self, remaining_seconds: f32) -> LocalizedText {
        match self.kind {
            NanocomMessageKind::BuddyInvite => LocalizedText::new(
                NANOCOM_EXPIRATION_LOCALIZATION_KEY,
                NANOCOM_EXPIRATION_FALLBACK,
            )
            .with_arg("seconds", rounded_seconds(remaining_seconds)),
            NanocomMessageKind::GroupInvite => LocalizedText::new(
                NANOCOM_GROUP_EXPIRATION_LOCALIZATION_KEY,
                NANOCOM_GROUP_EXPIRATION_FALLBACK,
            )
            .with_arg("seconds", rounded_seconds(remaining_seconds)),
            _ => nanocom_passthrough(""),
        }
    }

    pub(super) fn clean_group_host_name(&self) -> Option<&str> {
        self.body
            .strip_suffix(NANOCOM_GROUP_INVITATION_SUFFIX)
            .filter(|name| !name.is_empty() && name.trim() == *name)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NanocomMessageUiAction {
    pub request_id: u64,
    pub kind: NanocomMessageKind,
    pub resolution: NanocomMessageResolution,
}
