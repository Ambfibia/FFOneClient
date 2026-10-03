//! Retrobution tutorial voice-subtitle contract.
//!
//! `cntutorialscript.VoiceOut(str, chat, max)` is the authority for cue names,
//! event/line lookups and display lifetimes.  The legacy `OnGUI` rectangle is
//! also projected literally, including its second scale around the source
//! bottom-area pivot.  Audio playback and chat-log insertion remain caller
//! responsibilities so this module can be wired to the authoritative tutorial
//! scheduler without owning or replaying cues.

use std::{error::Error, fmt};

use bevy::{
    prelude::*,
    text::{LineBreak, LineHeight},
    window::PrimaryWindow,
};

use crate::localization::{LocalizationSet, LocalizedText};

use crate::{
    tutorial_logic::TutorialLocaleBranch, tutorial_mission_content::TutorialMissionContent,
};

#[cfg(test)]
mod tests;

mod audio_tutorial_voice_subtitle_state;
mod audio_sync_tutorial_voice_subtitle_ui;
mod constants;
mod operations;
mod localization_is_english_locale;
mod systems;
mod entities;

pub use audio_tutorial_voice_subtitle_state::{
    TUTORIAL_VOICE_SUBTITLE_EVENT, TUTORIAL_VOICE_SUBTITLE_SPEC_COUNT,
    TUTORIAL_VOICE_SUBTITLE_LINE_GAPS, TUTORIAL_VOICE_SUBTITLE_HEIGHT_RATIO,
    TUTORIAL_VOICE_SUBTITLE_HORIZONTAL_BASE, TUTORIAL_VOICE_SUBTITLE_REFERENCE_HEIGHT,
    TUTORIAL_VOICE_SUBTITLE_SCALE_BIAS, TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH,
    TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH, TUTORIAL_VOICE_SUBTITLE_Z_INDEX,
    TutorialVoiceSubtitleSpec, TUTORIAL_VOICE_SUBTITLE_SPECS, tutorial_voice_subtitle_spec,
    ResolvedTutorialVoiceSubtitle, TutorialVoiceSubtitleError, TutorialVoiceSubtitleResult,
    resolve_tutorial_voice_subtitle, resolve_tutorial_voice_subtitle_with,
    ActiveTutorialVoiceSubtitle, TutorialVoiceSubtitleState, TutorialVoiceSubtitleRect,
    TutorialVoiceSubtitleGeometry, TutorialVoiceSubtitleUiContext,
    TutorialVoiceSubtitleVisibility, tutorial_voice_subtitle_visibility,
    TutorialVoiceSubtitleInsets, TutorialVoiceSubtitleTextClipping,
    TutorialVoiceSubtitleImagePosition, TutorialVoiceSubtitleStyleContract,
    TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE, TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE,
    TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE, TutorialVoiceSubtitleUiElement,
    TutorialVoiceSubtitleSet, TutorialVoiceSubtitlePlugin
};
use audio_sync_tutorial_voice_subtitle_ui::sync_tutorial_voice_subtitle_ui;
pub use constants::{
    TUTORIAL_SKIP_LABEL, DEXTER_TUT10_ENGLISH_LITERAL, DEXTER_TUT12_ENGLISH_LITERAL
};
use operations::{spec, override_spec, gui_layout_label_node, gui_text_font, gui_text_layout};
pub use localization_is_english_locale::is_english_locale;
use systems::{apply_gui_layout_label_node, apply_gui_text_style};
use entities::spawn_tutorial_voice_subtitle_ui;
