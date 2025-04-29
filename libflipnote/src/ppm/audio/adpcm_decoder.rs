//! Audio parsing adapted from [flipnote.js](https://github.com/jaames/flipnote.js/blob/master/src/parsers/PpmParser.ts)

use anyhow::{ensure, Result};

use crate::ppm::constants::{ADPCM_INDEX_TABLE, ADPCM_STEP_TABLE};

pub fn decode_adpcm(data: &[u8], predictor: i16, step_index: u8) -> Result<Vec<i16>> {
    let mut output_samples = Vec::with_capacity(data.len() * 2);

    let mut step_index = step_index as i32;
    let mut predictor = predictor as i32;

    for &byte in data {
        for i in 0..2 {
            let sample = ((byte >> (i * 4)) & 0xF) as i32;

            let step = ADPCM_STEP_TABLE[step_index as usize];
            let mut diff = step >> 3;

            diff += if sample & 1 != 0 { step >> 2 } else { 0 };
            diff += if sample & 2 != 0 { step >> 1 } else { 0 };
            diff += if sample & 4 != 0 { step } else { 0 };
            if sample & 8 != 0 {
                diff = -diff;
            }

            predictor = (predictor + diff).clamp(-32768, 32767);
            step_index = (step_index + ADPCM_INDEX_TABLE[sample as usize]).clamp(0, 88);

            output_samples.push(predictor as i16);
        }
    }

    Ok(output_samples)
}

pub fn resample(
    samples: &[i16],
    source_frequency: i32,
    destination_frequency: i32,
) -> Result<Vec<i16>> {
    let source_duration = samples.len() as f32 / source_frequency as f32;
    let destination_length = (source_duration * destination_frequency as f32).ceil() as usize;

    let mut output_samples = Vec::with_capacity(destination_length);

    let adjusted_frequency = source_frequency as f32 / destination_frequency as f32;

    for i in 0..destination_length {
        let value = (i as f32 * adjusted_frequency).floor() as usize;
        output_samples.push(try_get_sample(samples, value)?);
    }

    Ok(output_samples)
}

pub fn try_get_sample(samples: &[i16], index: usize) -> Result<i16> {
    ensure!(index < samples.len(), "Index out of bounds");

    Ok(samples[index])
}

pub fn mix_pcm_audio(source: &[i16], destination: &[i16], offset: usize) -> Result<Vec<i16>> {
    let mut result = destination.to_owned();

    for i in 0..source.len() {
        if offset + i > destination.len() {
            break;
        }

        let sample = destination[offset + i] + (source[i] / 2);
        result[offset + i] = sample.clamp(-32768, 32767);
    }

    Ok(result)
}
