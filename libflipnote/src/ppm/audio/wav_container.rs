use anyhow::{Result, ensure};
use fon::{
    Audio, Frame, Resampler, Sink,
    chan::{Channel, Samp16},
};
use hound::WavWriter;
use rubato::{
    SincFixedIn, SincInterpolationParameters, SincInterpolationType, VecResampler, WindowFunction,
};
use std::num::NonZeroU32;
use std::{
    io::{Cursor, Write},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Mixer<'a, C: Channel, const N: usize> {
    index: usize,
    audio: &'a mut Audio<C, N>,
}

#[allow(single_use_lifetimes)]
impl<'a, C: Channel, const N: usize> Mixer<'a, C, N> {
    #[inline(always)]
    fn new(audio: &'a mut Audio<C, N>, index: usize) -> Self {
        Mixer { index, audio }
    }
}

// Using '_ results in reserved lifetime error.
#[allow(single_use_lifetimes)]
impl<C: Channel, const N: usize> Sink<C, N> for Mixer<'_, C, N> {
    #[inline(always)]
    fn sample_rate(&self) -> NonZeroU32 {
        self.audio.sample_rate()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.audio.len()
    }

    #[inline(always)]
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<C, N>>) {
        let mut this = self;
        Sink::<C, N>::sink_with(&mut this, iter)
    }
}

impl<C: Channel, const N: usize> Sink<C, N> for &mut Mixer<'_, C, N> {
    #[inline(always)]
    fn sample_rate(&self) -> NonZeroU32 {
        self.audio.sample_rate()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.audio.len()
    }

    #[inline(always)]
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<C, N>>) {
        for frame in self.audio.iter_mut().skip(self.index) {
            if let Some(other) = iter.next() {
                for (sample, samp) in frame.samples_mut().iter_mut().zip(other.samples()) {
                    *sample += *samp;
                }
            } else {
                break;
            }
            self.index += 1;
        }
    }
}

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
        if sample_rate == self.sample_rate {
            return Ok(self.clone());
        }

        let params = SincInterpolationParameters {
            sinc_len: 32,
            f_cutoff: 1.0,
            interpolation: SincInterpolationType::Nearest,
            oversampling_factor: 1,
            window: WindowFunction::BlackmanHarris,
        };

        let mut resampler = SincFixedIn::<f64>::new(
            sample_rate as f64 / self.sample_rate as f64,
            4.0,
            params,
            self.buffer.len(),
            self.channels as usize,
        )?;

        let samples = self.buffer.iter().map(|&s| s as f64).collect();

        let waves_in = vec![samples];

        let waves_out = resampler.process(&waves_in, None)?;

        Ok(Self::from_samples(
            waves_out[0].iter().map(|&s| s as i16).collect(),
            self.channels,
            sample_rate,
            self.bits_per_sample,
        ))
    }

    pub fn mix(&self, other: &Self, index: usize) -> Result<Self> {
        ensure!(
            self.channels == other.channels,
            "Channels must be the same for mixing"
        );

        let mut original_audio =
            Audio::<Samp16, 1>::with_i16_buffer(self.sample_rate as u32, self.get_samples());
        let other_audio =
            Audio::<Samp16, 1>::with_i16_buffer(other.sample_rate as u32, other.get_samples());

        let mut mixer = Mixer::new(&mut original_audio, index);

        let mut stream: Resampler<1> = Resampler::new(self.sample_rate as u32);
        stream.pipe(&other_audio, &mut mixer);
        stream.flush(&mut mixer);

        Ok(Self::from_samples(
            original_audio.as_i16_slice().to_vec(),
            self.channels,
            self.sample_rate,
            self.bits_per_sample,
        ))
    }

    pub fn get_samples(&self) -> Vec<i16> {
        self.buffer.to_owned()
    }

    pub fn get_sample_rate(&self) -> i32 {
        self.sample_rate
    }

    pub fn get_wav_buffer(&self) -> Result<Vec<u8>> {
        let spec = hound::WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate as u32,
            bits_per_sample: self.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = WavWriter::new(cursor, spec)?;

        for sample in self.buffer.iter() {
            writer.write_sample(*sample)?;
        }

        writer.finalize()?;

        Ok(buffer)
    }

    pub fn from_wav_buffer(buffer: Vec<u8>) -> Result<Self> {
        let cursor = Cursor::new(buffer);
        let reader = hound::WavReader::new(cursor)?;

        let spec = reader.spec();
        let samples: Vec<i16> = reader.into_samples::<i16>().map(|s| s.unwrap()).collect();

        Ok(Self {
            buffer: samples,
            channels: spec.channels,
            sample_rate: spec.sample_rate as i32,
            bits_per_sample: spec.bits_per_sample,
        })
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

        let wav_buffer = self.get_wav_buffer()?;

        let mut file = std::fs::File::create(path)?;
        file.write_all(&wav_buffer)?;

        Ok(())
    }
}
