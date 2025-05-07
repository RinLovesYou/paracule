use anyhow::Result;
use binrw::{BinResult, BinWrite};

use crate::ppm::{
    audio::{
        adpcm_ima::encode_adpcm,
        audio_data::{AdpcmImaHeader, PPMAudio},
    },
    constants::{ADPCM_STATE_HEADER_SIZE, PPM_AUDIO_SAMPLE_RATE, PPM_OFFSET_AUDIO_DATA_SIZE},
};

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
) -> Result<()> {
    let bgm = audio.background_track.to_owned();
    let se1 = audio.sound_effect_1_track.to_owned();
    let se2 = audio.sound_effect_2_track.to_owned();
    let se3 = audio.sound_effect_3_track.to_owned();

    let mut bgm_samples = Vec::new();
    let mut bgm_header = AdpcmImaHeader::default();

    let mut se1_samples = Vec::new();
    let mut se1_header = AdpcmImaHeader::default();

    let mut se2_samples = Vec::new();
    let mut se2_header = AdpcmImaHeader::default();

    let mut se3_samples = Vec::new();
    let mut se3_header = AdpcmImaHeader::default();

    let mut owned_audio = audio.to_owned();

    let mut audio_data_size = 0;

    if let Some(bgm) = &bgm {
        (bgm_header, bgm_samples) = encode_adpcm(
            &bgm.resample(audio.audio_header.get_bgm_sample_rate()?)?
                .get_samples(),
        )?;

        owned_audio.audio_header.bgm_track_size =
            (bgm_samples.len() + ADPCM_STATE_HEADER_SIZE) as u32;

        audio_data_size += owned_audio.audio_header.bgm_track_size;
    }

    if let Some(se1) = &se1 {
        (se1_header, se1_samples) =
            encode_adpcm(&se1.resample(PPM_AUDIO_SAMPLE_RATE)?.get_samples())?;
        owned_audio.audio_header.se1_track_size =
            (se1_samples.len() + ADPCM_STATE_HEADER_SIZE) as u32;

        audio_data_size += owned_audio.audio_header.se1_track_size;
    }

    if let Some(se2) = &se2 {
        (se2_header, se2_samples) =
            encode_adpcm(&se2.resample(PPM_AUDIO_SAMPLE_RATE)?.get_samples())?;
        owned_audio.audio_header.se2_track_size =
            (se2_samples.len() + ADPCM_STATE_HEADER_SIZE) as u32;

        audio_data_size += owned_audio.audio_header.se2_track_size;
    }

    if let Some(se3) = &se3 {
        (se3_header, se3_samples) =
            encode_adpcm(&se3.resample(PPM_AUDIO_SAMPLE_RATE)?.get_samples())?;
        owned_audio.audio_header.se3_track_size =
            (se3_samples.len() + ADPCM_STATE_HEADER_SIZE) as u32;

        audio_data_size += owned_audio.audio_header.se3_track_size;
    }

    owned_audio
        .audio_header
        .write_args(writer, ((frame_count, sound_header_start),))?;

    if !bgm_samples.is_empty() {
        bgm_header.write(writer)?;
        writer.write_all(&bgm_samples)?;
    }

    if !se1_samples.is_empty() {
        se1_header.write(writer)?;
        writer.write_all(&se1_samples)?;
    }

    if !se2_samples.is_empty() {
        se2_header.write(writer)?;
        writer.write_all(&se2_samples)?;
    }

    if !se3_samples.is_empty() {
        se3_header.write(writer)?;
        writer.write_all(&se3_samples)?;
    }

    let current_pos = writer.stream_position()?;
    writer.seek(std::io::SeekFrom::Start(PPM_OFFSET_AUDIO_DATA_SIZE))?;
    writer.write_all(&audio_data_size.to_le_bytes())?;
    writer.seek(std::io::SeekFrom::Start(current_pos))?;

    Ok(())
}
