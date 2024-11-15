use std::path::PathBuf;

use anyhow::{ensure, Result};
use photon_rs::{native::open_image, transform::{resize, SamplingFilter}, PhotonImage};

#[derive(Debug, Clone, Copy)]
pub struct RgbWrapper {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbWrapper {
    pub fn new(r: u8, g: u8, b: u8) -> RgbWrapper {
        RgbWrapper { r, g, b }
    }

    pub fn distance(&self, other: &RgbWrapper) -> f32 {
        let r = self.r as f32 - other.r as f32;
        let g = self.g as f32 - other.g as f32;
        let b = self.b as f32 - other.b as f32;

        (r * r + g * g + b * b).sqrt()
    }
}

pub struct ImageWrapper {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
}

impl ImageWrapper {
    pub fn new(width: u32, height: u32) -> ImageWrapper {
        let buffer = vec![0; (width * height * 4) as usize];

        ImageWrapper {
            buffer,
            width,
            height,
        }
    }

    pub fn load(path: impl Into<PathBuf>) -> Result<ImageWrapper> {
        let path: PathBuf = path.into();
        let image = open_image(&path.to_string_lossy().to_string())?;

        let buffer = image.get_raw_pixels().to_vec();

        Ok(ImageWrapper {
            buffer,
            width: image.get_width(),
            height: image.get_height(),
        })
    }

    pub fn save_as(&self, path: impl Into<PathBuf>) -> Result<()> {
        let path: PathBuf = path.into();
        let image = PhotonImage::new(self.buffer.clone(), self.width, self.height);

        photon_rs::native::save_image(image, &path.to_string_lossy().to_string())?;

        Ok(())
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: RgbWrapper) -> Result<()> {
        let index = ((y * self.width + x) * 4) as usize;

        self.buffer[index..index + 4].copy_from_slice(&[color.r, color.g, color.b, 255]);

        Ok(())
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Result<RgbWrapper> {
        let index = ((y * self.width + x) * 4) as usize;

        ensure!(index + 3 <= self.buffer.len(), "Pixel out of bounds");

        Ok(RgbWrapper::new(
            self.buffer[index],
            self.buffer[index + 1],
            self.buffer[index + 2],
        ))
    }

    pub fn get_pixels(&self) -> Result<Vec<RgbWrapper>> {
        let pixels = self.buffer.chunks(4).map(|pixel| {
            RgbWrapper::new(pixel[0], pixel[1], pixel[2])
        }).collect();

        Ok(pixels)
    }

    fn get_photon_image(&self) -> PhotonImage {
        PhotonImage::new(self.buffer.clone(), self.width, self.height)
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<ImageWrapper> {
        let image = self.get_photon_image();
        let resized = resize(&image, width, height, SamplingFilter::Nearest);

        Ok(ImageWrapper {
            buffer: resized.get_raw_pixels().to_vec(),
            width,
            height,
        })
    }
}
