//! Parser for the audio section of Flipnotes. Requires arguments for frame count and sound header start position, to calculate padding.

use anyhow::Result;
use binrw::binrw;

use crate::ppm::constants::PPM_FRAMERATE;

use super::{
    adpcm_decoder::{self, decode_adpcm, mix_pcm_audio},
    wav_container::WavContainer,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PPMSoundTrackType {
    BGM,
    SE1,
    SE2,
    SE3,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
//args in order: frame_count: u16, sound_header_start: u64
#[brw(import(args: (u16, u64)))]
pub struct PPMAudio {
    #[br(count = args.0)]
    sound_effect_flags: Vec<u8>,

    #[brw(pad_before = (4 - args.1 % 4) % 4)]
    //Sound Header
    bgm_track_size: u32,
    se1_track_size: u32,
    se2_track_size: u32,
    se3_track_size: u32,
    frame_playback_speed: u8,
    frame_playback_speed_when_recording: u8,

    //Sound Data
    #[brw(pad_before = 14)]
    #[br(count = bgm_track_size)]
    raw_bgm_track: Vec<u8>,
    #[br(count = se1_track_size)]
    raw_se1_track: Vec<u8>,
    #[br(count = se2_track_size)]
    raw_se2_track: Vec<u8>,
    #[br(count = se3_track_size)]
    raw_se3_track: Vec<u8>,
}

impl PPMAudio {
    /// Gets raw PCM samples for the selected soundtrack at the desired frequency (resampled if necessary using nearest neighbor)
    pub fn get_sound_track_samples(
        &self,
        track_type: PPMSoundTrackType,
        frequency: u32,
    ) -> Result<Vec<i16>> {
        let track = match track_type {
            PPMSoundTrackType::BGM => &self.raw_bgm_track,
            PPMSoundTrackType::SE1 => &self.raw_se1_track,
            PPMSoundTrackType::SE2 => &self.raw_se2_track,
            PPMSoundTrackType::SE3 => &self.raw_se3_track,
        };

        if track.is_empty() {
            return Ok(vec![]);
        }

        let mut samples = decode_adpcm(track)?;

        let speed = self.get_bgm_framerate()?;
        let frame_rate = self.get_framerate()?;

        let mut source_frequency = 8192;

        if track_type == PPMSoundTrackType::BGM {
            let bgm_adjust = (1.0 / speed) / (1.0 / frame_rate);
            source_frequency = (source_frequency as f32 * bgm_adjust) as u32;
        }

        if source_frequency != frequency {
            samples = adpcm_decoder::resample(&samples, source_frequency, frequency)?;
        }

        Ok(samples)
    }

    /// Gets raw PCM samples for the mixed soundtrack at the desired frequency (resampled if necessary using nearest neighbor)
    /// This includes all sound effects mixed with the BGM according to sound effect flags.
    pub fn get_mixed_sound_track_samples(&self, frequency: u32) -> Result<Vec<i16>> {
        let duration = self.get_duration()?.ceil();

        let mut master_samples = vec![0; (duration * frequency as f32).ceil() as usize + 2];

        if !self.raw_bgm_track.is_empty() {
            let bgm_samples = self.get_sound_track_samples(PPMSoundTrackType::BGM, frequency)?;
            master_samples = mix_pcm_audio(&bgm_samples, &master_samples, 0)?;
        }

        let samples_per_frame = frequency as f32 / self.get_framerate()?;

        let se1_samples = self.get_sound_track_samples(PPMSoundTrackType::SE1, frequency)?;
        let se2_samples = self.get_sound_track_samples(PPMSoundTrackType::SE2, frequency)?;
        let se3_samples = self.get_sound_track_samples(PPMSoundTrackType::SE3, frequency)?;

        for (i, flag) in self.sound_effect_flags.iter().enumerate() {
            let offset = i * samples_per_frame as usize;

            if !se1_samples.is_empty() && *flag == 1 {
                master_samples = mix_pcm_audio(&se1_samples, &master_samples, offset)?;
            }

            if !se2_samples.is_empty() && *flag == 2 {
                master_samples = mix_pcm_audio(&se2_samples, &master_samples, offset)?;
            }

            if !se3_samples.is_empty() && *flag == 4 {
                master_samples = mix_pcm_audio(&se3_samples, &master_samples, offset)?;
            }
        }

        Ok(master_samples)
    }

    /// Returns a WavContainer for the mixed soundtrack at the desired frequency (resampled if necessary using nearest neighbor)
    /// This includes all sound effects mixed with the BGM according to sound effect flags.
    pub fn get_mixed_sound_track_wav(&self, frequency: u32) -> Result<WavContainer> {
        let samples = self.get_mixed_sound_track_samples(frequency)?;

        let wav = WavContainer::from_samples(samples, 1, frequency, 16);

        Ok(wav)
    }

    /// Returns the actual FPS of the animation
    pub fn get_framerate(&self) -> Result<f32> {
        let speed = 8 - self.frame_playback_speed;

        if speed as usize >= PPM_FRAMERATE.len() {
            return Err(anyhow::anyhow!("Invalid frame playback speed"));
        }

        Ok(PPM_FRAMERATE[speed as usize])
    }

    /// Returns the actual FPS of the BGM when it was recorded
    pub fn get_bgm_framerate(&self) -> Result<f32> {
        let speed = 8 - self.frame_playback_speed_when_recording;

        if speed as usize >= PPM_FRAMERATE.len() {
            return Err(anyhow::anyhow!("Invalid frame playback speed"));
        }

        Ok(PPM_FRAMERATE[speed as usize])
    }

    /// Returns the duration of the animation
    pub fn get_duration(&self) -> Result<f32> {
        let frame_count = self.sound_effect_flags.len() as f32;
        let framerate = self.get_framerate()?;

        let duratrion = ((frame_count * 100.0) * (1.0 / framerate)) / 100.0;

        Ok(duratrion)
    }

    /// Returns a WavContainer for the selected soundtrack at the desired frequency (resampled if necessary using nearest neighbor)
    pub fn get_sound_track_wav(
        &self,
        track_type: PPMSoundTrackType,
        frequency: u32,
    ) -> Result<WavContainer> {
        let samples = self.get_sound_track_samples(track_type, frequency)?;

        let wav = WavContainer::from_samples(samples, 1, frequency, 16);

        Ok(wav)
    }
}
