use super::*;

pub const TUTORIAL_VOICE_SUBTITLE_EVENT: i32 = 2;

pub const TUTORIAL_VOICE_SUBTITLE_SPEC_COUNT: usize = 87;

pub const TUTORIAL_VOICE_SUBTITLE_LINE_GAPS: [i32; 4] = [31, 42, 74, 80];

pub const TUTORIAL_VOICE_SUBTITLE_HEIGHT_RATIO: f32 = 0.156_739_82;

pub const TUTORIAL_VOICE_SUBTITLE_HORIZONTAL_BASE: f32 = 250.0;

pub const TUTORIAL_VOICE_SUBTITLE_REFERENCE_HEIGHT: f32 = 768.0;

pub const TUTORIAL_VOICE_SUBTITLE_SCALE_BIAS: f32 = 1.05;

pub const TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH: &str = "fonts/jeffe.otf";

/// `cntutorialscript::OnGUI` draws cinematic bars first, the location title
/// second, voice subtitles third and the skip label last.  Keep this root
/// above the native bar/title layers (1901/1902) so the bottom bar remains the
/// subtitle backdrop instead of covering the text.
pub const TUTORIAL_VOICE_SUBTITLE_Z_INDEX: i32 = 1_903;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialVoiceSubtitleSpec {
    pub cue: &'static str,
    pub event: i32,
    pub line: i32,
    pub max_seconds: f32,
    pub english_literal_override: Option<&'static str>,
}

/// All unique `VoiceOut(cue, TextManager.GetSceneText(...), max)` identities
/// used by `cntutorialscript.cs`. Line 91 is invoked twice in the legacy
/// coroutine, but intentionally has one cue identity here.
pub const TUTORIAL_VOICE_SUBTITLE_SPECS: [TutorialVoiceSubtitleSpec;
    TUTORIAL_VOICE_SUBTITLE_SPEC_COUNT] = [
    spec("Computress_tut01", 1, 5.0),
    spec("Btrcup_Tut01", 2, 2.0),
    spec("NumFive_TutFollowme_Rev", 3, 3.0),
    spec("Ben_Tut01", 4, 3.0),
    spec("Computress_Tut07", 5, 7.0),
    spec("Computress_Tut08", 6, 5.0),
    spec("Computress_Tut02", 7, 2.0),
    spec("Computress_Tut03", 8, 3.0),
    spec("Computress_Tut04", 9, 3.0),
    spec("Computress_Tut05", 10, 3.0),
    spec("Computress_Tut06", 11, 3.0),
    spec("Computress_Tut09", 12, 3.0),
    spec("Computress_Tut10", 13, 5.0),
    spec("Computress_Tut11", 14, 5.0),
    spec("Computress_Tut12", 15, 10.0),
    spec("Ben_Tut02", 16, 5.0),
    spec("NumFive_TutHoldthem_Rev", 17, 2.0),
    spec("Ben_Tut03", 18, 3.0),
    spec("Ben_Tut04", 19, 7.0),
    spec("Computress_Tut16", 20, 14.0),
    spec("Computress_Tut17", 21, 3.0),
    spec("Ben_Tut05", 22, 2.0),
    spec("Ben_Tut06", 23, 3.0),
    spec("Ben_Tut08", 24, 2.0),
    spec("NumFive_TutIKnowYou", 25, 10.0),
    spec("NumFive_TutFuseExpl", 26, 12.0),
    spec("Ben_Tut09", 27, 13.0),
    spec("Ben_Tut10", 28, 4.0),
    spec("Computress_Tut13", 29, 10.0),
    spec("Computress_Tut14", 30, 10.0),
    spec("Computress_Tut15", 32, 10.0),
    spec("Computress_Tut18", 33, 10.0),
    spec("Computress_Tut19", 34, 5.0),
    spec("Computress_Tut20", 35, 3.0),
    spec("Computress_Tut21", 36, 3.0),
    spec("Computress_Tut22", 37, 10.0),
    spec("Computress_Tut23", 38, 5.0),
    spec("Computress_Tut25", 39, 10.0),
    spec("Computress_Tut26", 40, 10.0),
    spec("Computress_Tut38", 41, 10.0),
    spec("Btrcup_Tut03", 43, 3.0),
    spec("Computress_Tut41", 44, 10.0),
    spec("Computress_Tut24", 45, 10.0),
    spec("NumTwo_Tut02", 46, 3.0),
    spec("Computress_Tut27", 47, 10.0),
    spec("Computress_Tut28", 48, 5.0),
    spec("Computress_Tut29", 49, 10.0),
    spec("NumTwo_Tut03", 50, 3.0),
    spec("Computress_Tut30", 51, 10.0),
    spec("Computress_Tut31", 52, 5.0),
    spec("Computress_Tut32", 53, 5.0),
    spec("Computress_Tut33", 54, 5.0),
    spec("Computress_Tut34", 55, 5.0),
    spec("Computress_Tut35", 56, 10.0),
    spec("Computress_Tut36", 57, 10.0),
    spec("Computress_Tut37", 58, 5.0),
    spec("Computress_Tut39", 59, 10.0),
    spec("Computress_Tut40", 60, 5.0),
    spec("Btrcup_Tut08", 61, 12.0),
    spec("Dexter_Tut03", 62, 11.0),
    spec("Dexter_Tut04", 63, 5.0),
    spec("Dexter_Tut08", 64, 5.0),
    spec("Dexter_Tut09", 65, 6.0),
    override_spec("Dexter_Tut10", 66, 17.0, DEXTER_TUT10_ENGLISH_LITERAL),
    spec("Computress_Tut55", 67, 6.0),
    spec("Dexter_Tut11", 68, 10.0),
    spec("Dexter_Tut01", 69, 5.0),
    spec("Computress_Tut49", 70, 10.0),
    spec("Dexter_Tut02", 71, 5.0),
    spec("Computress_Tut56", 72, 3.0),
    spec("Computress_Tut57", 73, 5.0),
    spec("Computress_Tut44", 75, 5.0),
    spec("Btrcup_Tut07", 76, 10.0),
    spec("Computress_Tut45", 77, 5.0),
    spec("Computress_Tut46", 78, 10.0),
    spec("Computress_Tut47", 79, 5.0),
    spec("Computress_Tut50", 81, 10.0),
    spec("Computress_Tut51", 82, 5.0),
    spec("Computress_Tut53", 83, 3.0),
    spec("NumTwo_tut05", 84, 4.0),
    spec("NumTwo_Tut06", 85, 11.0),
    override_spec("Dexter_Tut12", 86, 24.0, DEXTER_TUT12_ENGLISH_LITERAL),
    spec("NumTwo_Tut07", 87, 4.0),
    spec("Computress_Tut60", 88, 8.0),
    spec("Computress_Tut61", 89, 10.0),
    spec("Computress_Tut59", 90, 10.0),
    spec("Computress_Tut62", 91, 3.0),
];

pub fn tutorial_voice_subtitle_spec(cue: &str) -> Option<&'static TutorialVoiceSubtitleSpec> {
    TUTORIAL_VOICE_SUBTITLE_SPECS
        .iter()
        .find(|spec| spec.cue.eq_ignore_ascii_case(cue))
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedTutorialVoiceSubtitle {
    pub cue: &'static str,
    pub event: i32,
    pub line: i32,
    pub max_seconds: f32,
    pub full_text: String,
    /// Exact legacy `array[0] + ":"` projection.
    pub speaker: String,
    /// Everything after the first `:`. No whitespace is trimmed.
    pub dialogue: String,
    pub used_english_literal_override: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TutorialVoiceSubtitleError {
    MissingMappedSceneText {
        cue: &'static str,
        event: i32,
        line: i32,
    },
    MissingSpeakerSeparator {
        cue: &'static str,
        event: i32,
        line: i32,
        text: String,
    },
    InvalidStartTime(f64),
    InvalidDuration(f32),
    NonMonotonicTime {
        started_at_seconds: f64,
        now_seconds: f64,
    },
    InvalidViewport {
        width: f32,
        height: f32,
    },
}

impl fmt::Display for TutorialVoiceSubtitleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMappedSceneText { cue, event, line } => write!(
                formatter,
                "tutorial voice cue {cue:?} maps to missing scene text event {event}, line {line}"
            ),
            Self::MissingSpeakerSeparator {
                cue,
                event,
                line,
                text,
            } => write!(
                formatter,
                "tutorial voice cue {cue:?} scene text {event}:{line} has no speaker separator ':': {text:?}"
            ),
            Self::InvalidStartTime(value) => {
                write!(formatter, "invalid tutorial subtitle start time {value:?}")
            }
            Self::InvalidDuration(value) => {
                write!(formatter, "invalid tutorial subtitle duration {value:?}")
            }
            Self::NonMonotonicTime {
                started_at_seconds,
                now_seconds,
            } => write!(
                formatter,
                "tutorial subtitle clock moved backwards: start={started_at_seconds}, now={now_seconds}"
            ),
            Self::InvalidViewport { width, height } => {
                write!(
                    formatter,
                    "invalid tutorial subtitle viewport {width}x{height}"
                )
            }
        }
    }
}

impl Error for TutorialVoiceSubtitleError {}

pub type TutorialVoiceSubtitleResult<T> = Result<T, TutorialVoiceSubtitleError>;

/// Resolves through manifest-verified XDT scene text. Unknown audio cues are
/// deliberately not an error because `VoiceOut(str)` also has audio-only uses.
pub fn resolve_tutorial_voice_subtitle(
    content: &TutorialMissionContent,
    cue: &str,
    locale: TutorialLocaleBranch,
) -> TutorialVoiceSubtitleResult<Option<ResolvedTutorialVoiceSubtitle>> {
    resolve_tutorial_voice_subtitle_with(cue, locale, |event, line| {
        content.scene_text(event, line).ok().map(str::to_owned)
    })
}

/// Lookup-injected form used by deterministic tests and loaders that already
/// own a verified cut-scene projection.
pub fn resolve_tutorial_voice_subtitle_with(
    cue: &str,
    locale: TutorialLocaleBranch,
    mut scene_text: impl FnMut(i32, i32) -> Option<String>,
) -> TutorialVoiceSubtitleResult<Option<ResolvedTutorialVoiceSubtitle>> {
    let Some(spec) = tutorial_voice_subtitle_spec(cue) else {
        return Ok(None);
    };
    let source_text = scene_text(spec.event, spec.line).ok_or(
        TutorialVoiceSubtitleError::MissingMappedSceneText {
            cue: spec.cue,
            event: spec.event,
            line: spec.line,
        },
    )?;
    let (full_text, used_english_literal_override) = if locale
        == TutorialLocaleBranch::OriginalEnglish
        && let Some(literal) = spec.english_literal_override
    {
        (literal.to_owned(), true)
    } else {
        (source_text, false)
    };
    let Some((name, dialogue)) = full_text.split_once(':') else {
        return Err(TutorialVoiceSubtitleError::MissingSpeakerSeparator {
            cue: spec.cue,
            event: spec.event,
            line: spec.line,
            text: full_text,
        });
    };
    Ok(Some(ResolvedTutorialVoiceSubtitle {
        cue: spec.cue,
        event: spec.event,
        line: spec.line,
        max_seconds: spec.max_seconds,
        full_text: full_text.clone(),
        speaker: format!("{name}:"),
        dialogue: dialogue.to_owned(),
        used_english_literal_override,
    }))
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveTutorialVoiceSubtitle {
    pub resolved: ResolvedTutorialVoiceSubtitle,
    pub started_at_seconds: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct TutorialVoiceSubtitleState {
    pub(super) active: Option<ActiveTutorialVoiceSubtitle>,
}

impl TutorialVoiceSubtitleState {
    pub fn active(&self) -> Option<&ActiveTutorialVoiceSubtitle> {
        self.active.as_ref()
    }

    /// Localized recordings may exceed the source subtitle timeout.
    pub fn retain_until(&mut self, until_seconds: f64) {
        if let Some(active) = self.active.as_mut()
            && until_seconds.is_finite()
        {
            active.resolved.max_seconds = active
                .resolved
                .max_seconds
                .max((until_seconds - active.started_at_seconds).max(0.0) as f32);
        }
    }

    pub fn start_from_cue(
        &mut self,
        content: &TutorialMissionContent,
        cue: &str,
        locale: TutorialLocaleBranch,
        now_seconds: f64,
    ) -> TutorialVoiceSubtitleResult<Option<&ActiveTutorialVoiceSubtitle>> {
        self.clear();
        let resolved = resolve_tutorial_voice_subtitle(content, cue, locale)?;
        self.start_resolved_option(resolved, now_seconds)
    }

    pub fn start_from_cue_with(
        &mut self,
        cue: &str,
        locale: TutorialLocaleBranch,
        now_seconds: f64,
        scene_text: impl FnMut(i32, i32) -> Option<String>,
    ) -> TutorialVoiceSubtitleResult<Option<&ActiveTutorialVoiceSubtitle>> {
        self.clear();
        let resolved = resolve_tutorial_voice_subtitle_with(cue, locale, scene_text)?;
        self.start_resolved_option(resolved, now_seconds)
    }

    pub fn start_resolved(
        &mut self,
        resolved: ResolvedTutorialVoiceSubtitle,
        now_seconds: f64,
    ) -> TutorialVoiceSubtitleResult<&ActiveTutorialVoiceSubtitle> {
        if !now_seconds.is_finite() || now_seconds < 0.0 {
            self.clear();
            return Err(TutorialVoiceSubtitleError::InvalidStartTime(now_seconds));
        }
        if !resolved.max_seconds.is_finite() || resolved.max_seconds <= 0.0 {
            self.clear();
            return Err(TutorialVoiceSubtitleError::InvalidDuration(
                resolved.max_seconds,
            ));
        }
        self.active = Some(ActiveTutorialVoiceSubtitle {
            resolved,
            started_at_seconds: now_seconds,
        });
        Ok(self.active.as_ref().expect("active subtitle was just set"))
    }

    pub(super) fn start_resolved_option(
        &mut self,
        resolved: Option<ResolvedTutorialVoiceSubtitle>,
        now_seconds: f64,
    ) -> TutorialVoiceSubtitleResult<Option<&ActiveTutorialVoiceSubtitle>> {
        // Both legacy VoiceOut overloads begin with VoiceOff(), which clears
        // VOSubTitle even for an audio-only cue.
        self.clear();
        let Some(resolved) = resolved else {
            return Ok(None);
        };
        self.start_resolved(resolved, now_seconds).map(Some)
    }

    pub fn clear(&mut self) -> Option<ActiveTutorialVoiceSubtitle> {
        self.active.take()
    }

    /// Mirrors `Time.time - VOSubTitleStartTime > SubTitleMaxTime`: equality
    /// remains visible for that frame.
    pub fn tick(&mut self, now_seconds: f64) -> TutorialVoiceSubtitleResult<bool> {
        let Some(active) = self.active.as_ref() else {
            return Ok(false);
        };
        if !now_seconds.is_finite() {
            return Err(TutorialVoiceSubtitleError::InvalidStartTime(now_seconds));
        }
        if now_seconds < active.started_at_seconds {
            return Err(TutorialVoiceSubtitleError::NonMonotonicTime {
                started_at_seconds: active.started_at_seconds,
                now_seconds,
            });
        }
        if now_seconds - active.started_at_seconds > f64::from(active.resolved.max_seconds) {
            self.clear();
            return Ok(true);
        }
        Ok(false)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TutorialVoiceSubtitleRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TutorialVoiceSubtitleRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn center(self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn scale_about(self, pivot: Vec2, scale: f32) -> Self {
        let center = pivot + (self.center() - pivot) * scale;
        Self::new(
            center.x - self.width * scale * 0.5,
            center.y - self.height * scale * 0.5,
            self.width * scale,
            self.height * scale,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialVoiceSubtitleGeometry {
    pub viewport: Vec2,
    pub ui_scale: f32,
    /// Literal rectangle passed to `GUILayout.BeginArea`.
    pub raw_area: TutorialVoiceSubtitleRect,
    /// Literal pivot passed to `FFGUIUtility.ScaleAroundPivot`.
    pub legacy_pivot: Vec2,
    /// Screen-space result after applying that GUI matrix.
    pub scaled_area: TutorialVoiceSubtitleRect,
}

impl TutorialVoiceSubtitleGeometry {
    pub fn for_viewport(
        width: f32,
        height: f32,
    ) -> TutorialVoiceSubtitleResult<TutorialVoiceSubtitleGeometry> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return Err(TutorialVoiceSubtitleError::InvalidViewport { width, height });
        }
        let ui_scale = (height / TUTORIAL_VOICE_SUBTITLE_REFERENCE_HEIGHT
            * TUTORIAL_VOICE_SUBTITLE_SCALE_BIAS)
            .max(1.0);
        let area_height = TUTORIAL_VOICE_SUBTITLE_HEIGHT_RATIO * height;
        let margin = TUTORIAL_VOICE_SUBTITLE_HORIZONTAL_BASE * ui_scale * ui_scale;
        let raw_area = TutorialVoiceSubtitleRect::new(
            margin,
            height - area_height,
            width - margin * 2.0,
            area_height,
        );
        // Source lines 1202-1204 use (Screen.width / 2, Screen.height -
        // area_height), i.e. the horizontal center of the bottom subtitle
        // area's top edge. Applying the GUI matrix is essential: using only
        // the already-squared margin misses the second legacy scale.
        let legacy_pivot = Vec2::new(width * 0.5, height - area_height);
        let scaled_area = raw_area.scale_about(legacy_pivot, ui_scale);
        Ok(Self {
            viewport: Vec2::new(width, height),
            ui_scale,
            raw_area,
            legacy_pivot,
            scaled_area,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct TutorialVoiceSubtitleUiContext {
    pub event_scene: bool,
    pub scene: Option<i32>,
    pub cinematic: bool,
    pub cinematic_alpha: f32,
    pub locale: TutorialLocaleBranch,
}

impl Default for TutorialVoiceSubtitleUiContext {
    fn default() -> Self {
        Self {
            event_scene: false,
            scene: None,
            cinematic: false,
            cinematic_alpha: 0.0,
            locale: TutorialLocaleBranch::OriginalEnglish,
        }
    }
}

impl TutorialVoiceSubtitleUiContext {
    pub fn set_locale(&mut self, locale: &str) {
        self.locale = if is_english_locale(locale) {
            TutorialLocaleBranch::OriginalEnglish
        } else {
            TutorialLocaleBranch::Localized
        };
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TutorialVoiceSubtitleVisibility {
    pub subtitle: bool,
    pub skip_label: bool,
}

pub fn tutorial_voice_subtitle_visibility(
    has_subtitle: bool,
    context: &TutorialVoiceSubtitleUiContext,
) -> TutorialVoiceSubtitleVisibility {
    let scene_is_some = context.scene.is_some();
    // The clean client kept localized VOSubTitle text visible outside an
    // event scene. The native presentation deliberately narrows that branch:
    // dialogue subtitles belong to cutscenes and must not overlap interactive
    // tutorial instructions after control returns to the player.
    TutorialVoiceSubtitleVisibility {
        subtitle: has_subtitle
            && context.event_scene
            && scene_is_some
            && context.cinematic
            && context.cinematic_alpha > 0.9,
        skip_label: context.event_scene && scene_is_some && context.cinematic,
    }
}

/// Serialized `RectOffset` used by Unity 2.5.5 `GUIStyle`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TutorialVoiceSubtitleInsets {
    pub left: i16,
    pub right: i16,
    pub top: i16,
    pub bottom: i16,
}

impl TutorialVoiceSubtitleInsets {
    pub const fn new(left: i16, right: i16, top: i16, bottom: i16) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub(super) fn scaled_ui_rect(self, scale: f32) -> UiRect {
        UiRect {
            left: px(f32::from(self.left) * scale),
            right: px(f32::from(self.right) * scale),
            top: px(f32::from(self.top) * scale),
            bottom: px(f32::from(self.bottom) * scale),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TutorialVoiceSubtitleTextClipping {
    Overflow,
    Clip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TutorialVoiceSubtitleImagePosition {
    ImageAbove,
    TextOnly,
}

/// Exact source contract recovered from `sharedassets0.assets`:
/// `cntutorialscript` pathId 1621 -> `pTextSkin` pathId 1380
/// (`FusionFallSysMessageSkin`) -> `customStyles`.
///
/// Unity 2.5.5 does not serialize a separate `GUIStyle.m_FontSize` here: each
/// style points at a fixed raster `Font` asset. `source_line_spacing` records
/// the exact serialized raster metric. `semantic_font_size` is the established
/// logical-pixel calibration for the already-published vector counterpart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialVoiceSubtitleStyleContract {
    pub legacy_name: &'static str,
    pub source_font_name: &'static str,
    pub source_font_path_id: i64,
    pub source_nominal_raster_size: f32,
    pub source_line_spacing: f32,
    pub semantic_font_path: &'static str,
    pub semantic_font_size: f32,
    /// Serialized `m_Normal.m_TextColor`, in RGBA order.
    pub normal_text_color: [f32; 4],
    pub normal_background_path_id: i64,
    /// All three source styles serialize `TextAnchor.MiddleCenter` (`4`).
    pub alignment: i32,
    pub border: TutorialVoiceSubtitleInsets,
    pub margin: TutorialVoiceSubtitleInsets,
    pub padding: TutorialVoiceSubtitleInsets,
    pub image_position: TutorialVoiceSubtitleImagePosition,
    pub word_wrap: bool,
    pub clipping: TutorialVoiceSubtitleTextClipping,
    pub stretch_width: bool,
    pub stretch_height: bool,
    pub fixed_width: f32,
    pub fixed_height: f32,
}

impl TutorialVoiceSubtitleStyleContract {
    pub(super) fn color(self) -> Color {
        Color::srgba(
            self.normal_text_color[0],
            self.normal_text_color[1],
            self.normal_text_color[2],
            self.normal_text_color[3],
        )
    }

    pub(super) fn overflow(self) -> Overflow {
        match self.clipping {
            TutorialVoiceSubtitleTextClipping::Overflow => Overflow::visible(),
            TutorialVoiceSubtitleTextClipping::Clip => Overflow::clip(),
        }
    }
}

pub const TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE: TutorialVoiceSubtitleStyleContract =
    TutorialVoiceSubtitleStyleContract {
        legacy_name: "centerbox",
        source_font_name: "JEFFE___14",
        source_font_path_id: 903,
        source_nominal_raster_size: 14.0,
        source_line_spacing: 13.710_000_038_146_973,
        semantic_font_path: TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH,
        // JEFFE___14's approximately 8 px cap height matches JEFFE.otf at 12
        // logical pixels (the same calibration used by character creation).
        semantic_font_size: 12.0,
        normal_text_color: [1.0, 1.0, 1.0, 1.0],
        normal_background_path_id: 0,
        alignment: 4,
        border: TutorialVoiceSubtitleInsets::new(1, 0, 0, 0),
        margin: TutorialVoiceSubtitleInsets::new(0, 0, 0, 0),
        padding: TutorialVoiceSubtitleInsets::new(0, 0, 0, 0),
        image_position: TutorialVoiceSubtitleImagePosition::ImageAbove,
        word_wrap: true,
        clipping: TutorialVoiceSubtitleTextClipping::Clip,
        stretch_width: true,
        stretch_height: false,
        fixed_width: 0.0,
        fixed_height: 0.0,
    };

pub const TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE: TutorialVoiceSubtitleStyleContract =
    TutorialVoiceSubtitleStyleContract {
        legacy_name: "smallfont2",
        source_font_name: "ChaletBook-Regular",
        source_font_path_id: 1115,
        source_nominal_raster_size: 14.0,
        source_line_spacing: 14.083_999_633_789_062,
        semantic_font_path: TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH,
        semantic_font_size: 14.0,
        normal_text_color: [1.0, 0.995_967_75, 1.0, 1.0],
        normal_background_path_id: 0,
        alignment: 4,
        border: TutorialVoiceSubtitleInsets::new(5, 5, 5, 5),
        margin: TutorialVoiceSubtitleInsets::new(4, 4, 4, 4),
        padding: TutorialVoiceSubtitleInsets::new(10, 6, 4, 6),
        image_position: TutorialVoiceSubtitleImagePosition::TextOnly,
        word_wrap: true,
        clipping: TutorialVoiceSubtitleTextClipping::Overflow,
        stretch_width: true,
        stretch_height: false,
        fixed_width: 0.0,
        fixed_height: 0.0,
    };

pub const TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE: TutorialVoiceSubtitleStyleContract =
    TutorialVoiceSubtitleStyleContract {
        legacy_name: "smallfont",
        source_font_name: "ChaletBook-Regular Small",
        source_font_path_id: 1018,
        source_nominal_raster_size: 12.0,
        source_line_spacing: 12.071_999_549_865_723,
        semantic_font_path: TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH,
        semantic_font_size: 12.0,
        normal_text_color: [1.0, 0.995_967_75, 1.0, 1.0],
        normal_background_path_id: 0,
        alignment: 4,
        border: TutorialVoiceSubtitleInsets::new(5, 5, 5, 5),
        margin: TutorialVoiceSubtitleInsets::new(4, 4, 4, 4),
        padding: TutorialVoiceSubtitleInsets::new(10, 6, 4, 6),
        image_position: TutorialVoiceSubtitleImagePosition::TextOnly,
        word_wrap: true,
        clipping: TutorialVoiceSubtitleTextClipping::Clip,
        stretch_width: true,
        stretch_height: false,
        fixed_width: 0.0,
        fixed_height: 0.0,
    };

#[derive(Clone, Copy, Debug, Component, Eq, PartialEq)]
pub enum TutorialVoiceSubtitleUiElement {
    Root,
    Area,
    Speaker,
    Dialogue,
    SkipLabel,
    SkipLabelText,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum TutorialVoiceSubtitleSet {
    Tick,
    Context,
    Bind,
}

#[derive(Default)]
pub struct TutorialVoiceSubtitlePlugin;

impl Plugin for TutorialVoiceSubtitlePlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<TutorialVoiceSubtitleState>()
            .init_resource::<TutorialVoiceSubtitleUiContext>()
            .configure_sets(
                Update,
                (
                    TutorialVoiceSubtitleSet::Tick,
                    TutorialVoiceSubtitleSet::Context,
                    TutorialVoiceSubtitleSet::Bind,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_tutorial_voice_subtitle_ui,
            )
            .add_systems(
                Update,
                (tick_tutorial_voice_subtitle.in_set(TutorialVoiceSubtitleSet::Tick))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (sync_tutorial_voice_subtitle_ui
                    .in_set(TutorialVoiceSubtitleSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

pub(super) fn tick_tutorial_voice_subtitle(time: Res<Time>, mut state: ResMut<TutorialVoiceSubtitleState>) {
    // Bevy's monotonic clock satisfies the checked state contract.
    let _ = state.tick(time.elapsed_secs_f64());
}
