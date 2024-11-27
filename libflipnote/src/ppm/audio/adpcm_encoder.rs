use anyhow::Result;

use crate::ppm::constants::{ADPCM_INDEX_TABLE, ADPCM_STEP_TABLE};

pub fn encode_adpcm(data: &[i16]) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(data.len() / 2);

    let mut previous_sample = 0;
    let mut step_index = 0;

    for sample in data.chunks(2) {
        let s1 = encode_sample(sample[0] as i32, &mut previous_sample, &mut step_index);
        let s2 = encode_sample(sample[1] as i32, &mut previous_sample, &mut step_index) << 4;

        output.push((s1 | s2) as u8);
    }

    Ok(output)
}

fn encode_sample(sample: i32, previous_sample: &mut i32, step_index: &mut i32) -> i32 {
    let mut delta = sample - *previous_sample;

    let mut encoded_sample = 0;

    if delta < 0 {
        encoded_sample = 8;
        delta = -delta;
    }

    encoded_sample += 7.min((delta * 4) as i32 / ADPCM_STEP_TABLE[*step_index as usize]);

    *previous_sample = sample;
    *step_index = (*step_index + ADPCM_INDEX_TABLE[encoded_sample as usize] as i32).clamp(0, 88);

    encoded_sample
}