//! Audio parsing adapted from [flipnote.js](https://github.com/jaames/flipnote.js/blob/master/src/parsers/PpmParser.ts)

use anyhow::{ensure, Result};

use crate::ppm::constants::{ADPCM_INDEX_TABLE, ADPCM_STEP_TABLE};

pub fn decode_adpcm(data: &[u8]) -> Result<Vec<i16>> {
    let mut output_samples = vec![];
    output_samples.resize(data.len() * 2, 0);

    let mut step_index = 0;
    let mut predictor = 0;

    let mut low_nibble = true;

    let mut source_pointer = 0;
    let mut destination_pointer = 0;

    while source_pointer < data.len() {
        let mut sample = data[source_pointer] & 0x0F;

        if !low_nibble {
            sample = data[source_pointer] >> 4;
            source_pointer += 1;
        }

        low_nibble = !low_nibble;

        let step = ADPCM_STEP_TABLE[step_index];
        let mut diff = step >> 3;

        diff += if (sample & 1) != 0 { step >> 2 } else { 0 };
        diff += if (sample & 2) != 0 { step >> 1 } else { 0 };
        diff += if (sample & 4) != 0 { step } else { 0 };
        if (sample & 8) != 0 {
            diff = -diff;
        }

        predictor += diff;
        predictor = predictor.clamp(-32768, 32767);

        step_index += ADPCM_INDEX_TABLE[sample as usize] as usize;
        step_index = step_index.clamp(0, 88);

        output_samples[destination_pointer] = predictor as i16;
        destination_pointer += 1;
    }

    Ok(output_samples)
}

pub fn resample(
    samples: &[i16],
    source_frequency: u32,
    destination_frequency: u32,
) -> Result<Vec<i16>> {
    let source_duration = samples.len() / source_frequency as usize;
    let destination_length = source_duration * destination_frequency as usize;

    let mut output_samples = vec![0; destination_length + 1];

    let adjusted_frequency = source_frequency as f32 / destination_frequency as f32;

    for i in 0..destination_length {
        let value = (i as f32 * adjusted_frequency).floor() as usize;
        output_samples[i] = try_get_sample(samples, value)?;
    }

    Ok(output_samples)
}

pub fn try_get_sample(samples: &[i16], index: usize) -> Result<i16> {
    ensure!(index < samples.len(), "Index out of bounds");

    Ok(samples[index])
}

pub fn mix_pcm_audio(source: &[i16], destination: &[i16], offset: usize) -> Result<Vec<i16>> {
    let mut result = vec![0; destination.len()];

    for i in 0..source.len() {
        if offset + i >= destination.len() {
            break;
        }

        let sample = destination[offset + i] + (source[i] / 2);
        result[offset + i] = sample.clamp(-32768, 32767);
    }

    Ok(result)
}
