use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemMessageRequest {
    /// Native correlation token returned through `SystemMessageUiOutbox`.
    pub request_id: u64,
    pub text: String,
    /// Key-first copy attached to the spawned body. Arbitrary legacy/server
    /// text is represented through `ui.content.passthrough`; native callers
    /// can retain a semantic key with [`Self::new_localized`].
    pub localized: LocalizedText,
    pub button_type: SystemMessageButtonType,
    /// Optional semantic runtime image. Clean Retrobution initializes every
    /// message from serialized warning icon index zero; callers may replace it
    /// with an item icon or another proven legacy icon slot.
    pub icon_path: Option<String>,
    /// Clean `iDeleteNums`; rendered only for positive values on type 12.
    pub icon_quantity: i32,
    /// Clean `IconTexture2`/`IconTexture3`. The pair is all-or-nothing in
    /// `DrawAll`; an incomplete pair remains hidden rather than half-painted.
    pub comparison_icon_paths: [Option<String>; 2],
    /// Clean `bIconCombi2`/`bIconCombi3` overlays for the comparison pair.
    pub comparison_icon_combined: [bool; 2],
}

impl SystemMessageRequest {
    #[must_use]
    pub fn new(
        request_id: u64,
        text: impl Into<String>,
        button_type: SystemMessageButtonType,
    ) -> Self {
        let text = text.into();
        Self {
            request_id,
            localized: LocalizedText::new("ui.content.passthrough", "{text}")
                .with_arg("text", text.clone()),
            text,
            button_type,
            icon_path: SystemMessageIconIndex::Warning
                .runtime_path()
                .map(str::to_owned),
            icon_quantity: 0,
            comparison_icon_paths: [None, None],
            comparison_icon_combined: [false, false],
        }
    }

    #[must_use]
    pub fn new_localized(
        request_id: u64,
        localized: LocalizedText,
        button_type: SystemMessageButtonType,
    ) -> Self {
        let text = localized
            .args
            .iter()
            .fold(localized.fallback.clone(), |text, (name, value)| {
                text.replace(&format!("{{{name}}}"), value)
            });
        Self {
            request_id,
            text,
            localized,
            button_type,
            icon_path: SystemMessageIconIndex::Warning
                .runtime_path()
                .map(str::to_owned),
            icon_quantity: 0,
            comparison_icon_paths: [None, None],
            comparison_icon_combined: [false, false],
        }
    }

    pub fn try_from_legacy(
        request_id: u64,
        text: impl Into<String>,
        raw_button_type: i32,
    ) -> Result<Self, SystemMessageButtonTypeError> {
        Ok(Self::new(
            request_id,
            text,
            SystemMessageButtonType::try_from(raw_button_type)?,
        ))
    }

    #[must_use]
    pub fn with_icon_path(mut self, path: impl Into<String>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    pub fn try_with_legacy_icon_index(
        mut self,
        raw: i32,
    ) -> Result<Self, SystemMessageIconIndexError> {
        self.icon_path = SystemMessageIconIndex::try_from(raw)?
            .runtime_path()
            .map(str::to_owned);
        Ok(self)
    }

    #[must_use]
    pub const fn with_icon_quantity(mut self, quantity: i32) -> Self {
        self.icon_quantity = quantity;
        self
    }

    #[must_use]
    pub fn with_comparison_icons(
        mut self,
        left_path: impl Into<String>,
        left_combined: bool,
        right_path: impl Into<String>,
        right_combined: bool,
    ) -> Self {
        self.comparison_icon_paths = [Some(left_path.into()), Some(right_path.into())];
        self.comparison_icon_combined = [left_combined, right_combined];
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemMessageUiAction {
    Chosen {
        request_id: u64,
        button_type: SystemMessageButtonType,
        choice: SystemMessageChoice,
    },
}
