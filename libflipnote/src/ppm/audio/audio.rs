//! Parser for the audio section of Flipnotes. Requires arguments for frame count and sound header start position, to calculate padding.

use super::{audio_header::PPMAudioHeader, wav_container::WavContainer};

#[derive(Debug, Clone, Default)]
pub struct PPMAudio {
    pub audio_header: PPMAudioHeader,

    pub background_track: Option<WavContainer>,
    pub sound_effect_1_track: Option<WavContainer>,
    pub sound_effect_2_track: Option<WavContainer>,
    pub sound_effect_3_track: Option<WavContainer>,

    pub mixed_tracks: Option<WavContainer>,
}
