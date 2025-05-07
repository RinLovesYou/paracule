use anyhow::Result;
use audio_codec_algorithms::{AdpcmImaState, decode_adpcm_ima, encode_adpcm_ima};

use super::{audio_data::AdpcmImaHeader, wav_container::WavContainer};

pub fn decode_adpcm(data: &[u8], frequency: i32, header: AdpcmImaHeader) -> Result<WavContainer> {
    let mut adpcm_state = AdpcmImaState::new();
    adpcm_state.predictor = header.predictor;
    adpcm_state.step_index = header.step_index;

    let samples = data
        .iter()
        .flat_map(|byte| {
            let s0 = decode_adpcm_ima(*byte & 0xF, &mut adpcm_state);
            let s1 = decode_adpcm_ima(*byte >> 4, &mut adpcm_state);
            vec![s0, s1]
        })
        .collect::<Vec<i16>>();

    let container = WavContainer::from_samples(samples, 1, frequency, 16);

    Ok(container)
}

pub fn encode_adpcm(samples: &[i16]) -> Result<(AdpcmImaHeader, Vec<u8>)> {
    let mut adpcm_state = AdpcmImaState::new();

    let data = samples
        .chunks(2)
        .map(|chunk| {
            let s1 = encode_adpcm_ima(chunk[0], &mut adpcm_state);
            if chunk.len() > 1 {
                let s2 = encode_adpcm_ima(chunk[1], &mut adpcm_state);
                s2 << 4 | s1
            } else {
                s1
            }
        })
        .collect::<Vec<u8>>();

    let header = AdpcmImaHeader {
        predictor: 0,
        step_index: 0,
        unused: 0,
    };

    Ok((header, data))
}
