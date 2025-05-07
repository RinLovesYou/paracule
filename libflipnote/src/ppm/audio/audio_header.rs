use anyhow::Result;
use binrw::binrw;

use crate::ppm::constants::{PPM_AUDIO_SAMPLE_RATE, PPM_FRAMERATE};

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
//args in order: frame_count: u16, sound_header_start: u64
#[brw(import(args: (u16, u64)))]
pub struct PPMAudioHeader {
    #[br(count = args.0)]
    pub sound_effect_flags: Vec<u8>,

    #[brw(pad_before = 4 - (args.1 % 4))]
    //Sound Header
    pub bgm_track_size: u32,
    pub se1_track_size: u32,
    pub se2_track_size: u32,
    pub se3_track_size: u32,
    frame_playback_speed: u8,
    #[brw(pad_after = 14)]
    frame_playback_speed_when_recording: u8,
}

impl PPMAudioHeader {
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
        let frame_count = self.sound_effect_flags.len() - 1;
        let framerate = self.get_framerate()?;

        let duratrion = (((frame_count * 100) as f32) * (1.0 / framerate)) / 100.0;

        Ok(duratrion)
    }

    pub fn get_bgm_sample_rate(&self) -> Result<i32> {
        // (PPM_AUDIO_SAMPLE_RATE as f32)
        //     * ((1.0 / header.get_bgm_framerate()?) / (1.0 / header.get_framerate()?)))
        // .floor() as i32

        let framerate = self.get_framerate()?;
        let bgm_framerate = self.get_bgm_framerate()?;

        let bgm_adjust = (1.0 / bgm_framerate) / (1.0 / framerate);

        Ok((PPM_AUDIO_SAMPLE_RATE as f32 * bgm_adjust).floor() as i32)
    }
}
