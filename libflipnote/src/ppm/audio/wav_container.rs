use std::path::PathBuf;

use anyhow::{ensure, Result};
use hound::WavWriter;

use super::adpcm_decoder;

#[derive(Debug, Clone, Default)]
pub struct WavContainer {
    buffer: Vec<i16>,
    channels: u16,
    sample_rate: i32,
    bits_per_sample: u16,
}

impl WavContainer {
    pub fn from_samples(
        buffer: Vec<i16>,
        channels: u16,
        sample_rate: i32,
        bits_per_sample: u16,
    ) -> Self {
        Self {
            buffer,
            channels,
            sample_rate,
            bits_per_sample,
        }
    }

    pub fn resample(&self, sample_rate: i32) -> Result<Self> {
        let resampled = adpcm_decoder::resample(&self.buffer, self.sample_rate, sample_rate)?;

        Ok(Self {
            buffer: resampled,
            channels: self.channels,
            sample_rate,
            bits_per_sample: self.bits_per_sample,
        })
    }

    pub fn get_samples(&self) -> Vec<i16> {
        self.buffer.to_owned()
    }

    pub fn save_as(&self, path: impl Into<PathBuf>) -> Result<()> {
        let mut path: PathBuf = path.into();

        if path.extension().is_none() {
            path.set_extension("wav");
        }

        ensure!(
            path.extension().unwrap() == "wav",
            "File must have a .wav extension"
        );

        let spec = hound::WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate as u32,
            bits_per_sample: self.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;

        for sample in self.buffer.to_owned() {
            writer.write_sample(sample)?;
        }

        writer.finalize()?;

        Ok(())
    }
}
