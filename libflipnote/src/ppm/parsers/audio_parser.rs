use crate::ppm::{
    audio::{
        adpcm_ima::decode_adpcm,
        audio_data::{AdpcmImaHeader, PPMAudio},
        audio_header::PPMAudioHeader,
        wav_container::WavContainer,
    },
    constants::{ADPCM_STATE_HEADER_SIZE, PPM_AUDIO_PLAYBACK_SAMPLE_RATE, PPM_AUDIO_SAMPLE_RATE},
};
use anyhow::Result;
use binrw::{BinRead, BinResult};
use fon::{Audio, chan::Samp16};
use std::vec;

#[binrw::parser(reader)]
pub fn audio_parser(frame_count: u16, sound_header_start: u64) -> BinResult<PPMAudio> {
    let audio = inner_reader(reader, frame_count, sound_header_start).map_err(|e| {
        binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap(),
            message: e.to_string(),
        }
    })?;

    Ok(audio)
}

fn inner_reader<T: binrw::io::Read + binrw::io::Seek>(
    reader: &mut T,
    frame_count: u16,
    sound_header_start: u64,
) -> Result<PPMAudio> {
    let header = PPMAudioHeader::read_args(reader, ((frame_count, sound_header_start),))?;

    let backgroung_track = read_audio_data(reader, header.bgm_track_size, &header, true)?;

    let se1_track = read_audio_data(reader, header.se1_track_size, &header, false)?;
    let se2_track = read_audio_data(reader, header.se2_track_size, &header, false)?;
    let se3_track = read_audio_data(reader, header.se3_track_size, &header, false)?;

    let mut audio = PPMAudio {
        audio_header: header,
        background_track: backgroung_track,
        sound_effect_1_track: se1_track,
        sound_effect_2_track: se2_track,
        sound_effect_3_track: se3_track,
        mixed_tracks: None,
    };

    audio.mixed_tracks = mix_audio(&audio)?;

    Ok(audio)
}

fn read_audio_data<T: binrw::io::Read + binrw::io::Seek>(
    reader: &mut T,
    size: u32,
    header: &PPMAudioHeader,
    is_bgm: bool,
) -> Result<Option<WavContainer>> {
    if size == 0 {
        return Ok(None);
    }

    let adpcm_header = AdpcmImaHeader::read(reader)?;
    let mut data = vec![0u8; size as usize - ADPCM_STATE_HEADER_SIZE];
    reader.read_exact(&mut data)?;

    let source_frequency = match is_bgm {
        true => header.get_bgm_sample_rate()?,
        false => PPM_AUDIO_SAMPLE_RATE,
    };

    let container = decode_adpcm(&data, source_frequency, adpcm_header)?
        .resample(PPM_AUDIO_PLAYBACK_SAMPLE_RATE)?;

    Ok(Some(container))
}

fn mix_audio(audio: &PPMAudio) -> Result<Option<WavContainer>> {
    if audio.background_track.is_none() {
        return Ok(None);
    }

    let mut master_audio = WavContainer::from_samples(
        Audio::<Samp16, 1>::with_silence(
            PPM_AUDIO_PLAYBACK_SAMPLE_RATE as u32,
            (audio.audio_header.get_duration()?.ceil() * PPM_AUDIO_PLAYBACK_SAMPLE_RATE as f32)
                as usize,
        )
        .as_i16_slice()
        .to_vec(),
        1,
        PPM_AUDIO_PLAYBACK_SAMPLE_RATE,
        16,
    );

    if let Some(bgm) = &audio.background_track {
        master_audio = master_audio.mix(bgm, 0)?;
    }

    let samples_per_frame =
        PPM_AUDIO_PLAYBACK_SAMPLE_RATE as f32 / audio.audio_header.get_framerate()?;

    for (i, flag) in audio.audio_header.sound_effect_flags.iter().enumerate() {
        let offset = (i as f64 * samples_per_frame as f64).ceil() as usize;

        if let Some(se1) = &audio.sound_effect_1_track {
            if flag & 0x1 != 0 {
                master_audio = master_audio.mix(se1, offset)?;
            }
        }

        if let Some(se2) = &audio.sound_effect_2_track {
            if flag & 0x2 != 0 {
                master_audio = master_audio.mix(se2, offset)?;
            }
        }

        if let Some(se3) = &audio.sound_effect_3_track {
            if flag & 0x4 != 0 {
                master_audio = master_audio.mix(se3, offset)?;
            }
        }
    }

    Ok(Some(master_audio))

    // let duration = audio.audio_header.get_duration()?.ceil();

    // let mut master_samples = vec![0; (duration * PPM_AUDIO_PLAYBACK_SAMPLE_RATE as f32).ceil() as usize];

    // let samples_per_frame = PPM_AUDIO_PLAYBACK_SAMPLE_RATE as f32 / audio.audio_header.get_framerate()?;

    // if let Some(bgm) = &audio.background_track {
    //     let bgm_samples = bgm.get_samples();
    //     master_samples = adpcm_decoder::mix_pcm_audio(&bgm_samples, &master_samples, 0)?;
    // }

    // for (i, flag) in audio.audio_header.sound_effect_flags.iter().enumerate() {
    //     let offset = (i as f64 * samples_per_frame as f64).ceil() as usize;

    //     if let Some(se1) = &audio.sound_effect_1_track {
    //         let se1_samples = se1.get_samples();
    //         if flag & 0x1 != 0 {
    //             master_samples =
    //                 adpcm_decoder::mix_pcm_audio(&se1_samples, &master_samples, offset)?;
    //         }
    //     }

    //     if let Some(se2) = &audio.sound_effect_2_track {
    //         let se2_samples = se2.get_samples();
    //         if flag & 0x2 != 0 {
    //             master_samples =
    //                 adpcm_decoder::mix_pcm_audio(&se2_samples, &master_samples, offset)?;
    //         }
    //     }

    //     if let Some(se3) = &audio.sound_effect_3_track {
    //         let se3_samples = se3.get_samples();
    //         if flag & 0x4 != 0 {
    //             master_samples =
    //                 adpcm_decoder::mix_pcm_audio(&se3_samples, &master_samples, offset)?;
    //         }
    //     }
    // }

    // let container = WavContainer::from_samples(master_samples, 1, PPM_AUDIO_PLAYBACK_SAMPLE_RATE, 16);

    // Ok(Some(container))
}
