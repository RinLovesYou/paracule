use binrw::{BinResult, BinWrite};

use crate::ppm::{audio::{adpcm_encoder::encode_adpcm, audio::PPMAudio}, constants::PPM_AUDIO_SAMPLE_RATE};

#[binrw::writer(writer)]
pub fn audio_writer(obj: &PPMAudio, frame_count: u16, sound_header_start: u64) -> BinResult<()> {
    inner_writer(writer, obj, frame_count, sound_header_start).map_err(|e| {
        binrw::Error::AssertFail {
            pos: writer.stream_position().unwrap(),
            message: e.to_string(),
        }
    })?;

    Ok(())
}

fn inner_writer<T: binrw::io::Write + binrw::io::Seek>(
    writer: &mut T,
    audio: &PPMAudio,
    frame_count: u16,
    sound_header_start: u64,
) -> anyhow::Result<()> {
    let bgm = audio.background_track.to_owned();
    let se1 = audio.sound_effect_1_track.to_owned();
    let se2 = audio.sound_effect_2_track.to_owned();
    let se3 = audio.sound_effect_3_track.to_owned();

    let mut bgm_samples = Vec::new();
    let mut se1_samples = Vec::new();
    let mut se2_samples = Vec::new();
    let mut se3_samples = Vec::new();

    let mut owned_audio = audio.to_owned();

    if let Some(bgm) = &bgm {
        bgm_samples = encode_adpcm(&bgm.get_samples())?;
        owned_audio.audio_header.bgm_track_size = bgm_samples.len() as u32;
    }

    if let Some(se1) = &se1 {
        se1_samples = encode_adpcm(&se1.get_samples())?;
        owned_audio.audio_header.se1_track_size = se1_samples.len() as u32;
    }

    if let Some(se2) = &se2 {
        se2_samples = encode_adpcm(&se2.get_samples())?;
        owned_audio.audio_header.se2_track_size = se2_samples.len() as u32;
    }

    if let Some(se3) = &se3 {
        se3_samples = encode_adpcm(&se3.get_samples())?;
        owned_audio.audio_header.se3_track_size = se3_samples.len() as u32;
    }

    owned_audio
        .audio_header
        .write_args(writer, ((frame_count, sound_header_start),))?;

    if !bgm_samples.is_empty() {
        let frequency = PPM_AUDIO_SAMPLE_RATE;
        let mut source_frequency = PPM_AUDIO_SAMPLE_RATE;

        let frame_rate = owned_audio.audio_header.get_framerate()?;
        let bgm_framerate = owned_audio.audio_header.get_bgm_framerate()?;

        let bgm_adjust = (1.0 / bgm_framerate) / (1.0 / frame_rate);
        source_frequency = (source_frequency as f32 / bgm_adjust) as i32;

        if source_frequency != frequency {
            let s = owned_audio.background_track.unwrap().resample(source_frequency)?;
            bgm_samples = encode_adpcm(&s.get_samples())?;
        }

        writer.write_all(&bgm_samples)?;
    }

    if !se1_samples.is_empty() {
        writer.write_all(&se1_samples)?;
    }

    if !se2_samples.is_empty() {
        writer.write_all(&se2_samples)?;
    }

    if !se3_samples.is_empty() {
        writer.write_all(&se3_samples)?;
    }

    Ok(())
}
