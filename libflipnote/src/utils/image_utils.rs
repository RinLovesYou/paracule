use anyhow::Result;
use dithord::{OrderedDither, ThresholdMap};
use image::{
    DynamicImage, ImageBuffer, Rgba, RgbaImage,
    imageops::{self, ColorMap, FilterType, resize},
};
use std::path::PathBuf;

use crate::ppm::constants::{
    PPM_COLOR_BLUE, PPM_COLOR_RED, PPM_PAPER_COLORS, PPM_THUMBNAIL_COLORS,
};

use super::color_utils::{hex_to_rgb, rgb_to_ppm_frame_pixel, single_rgb_to_thumbnail_pixel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum DitherType {
    Bayer4x4,
    #[default]
    Bayer8x8,
}

#[derive(Debug, Clone, Copy, Default)]
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

pub struct PPMThumbnailColorMap;
pub struct PPMFrameColorMap;

impl ColorMap for PPMThumbnailColorMap {
    type Color = Rgba<u8>;

    #[inline(always)]
    fn index_of(&self, color: &Rgba<u8>) -> usize {
        let wrapped = RgbWrapper::new(color[0], color[1], color[2]);

        single_rgb_to_thumbnail_pixel(&wrapped)
    }

    #[inline(always)]
    fn lookup(&self, idx: usize) -> Option<Self::Color> {
        hex_to_rgb(PPM_THUMBNAIL_COLORS[idx])
            .map(|rgb| Rgba([rgb.r, rgb.g, rgb.b, 255]))
            .ok()
    }

    /// Indicate NeuQuant implements `lookup`.
    fn has_lookup(&self) -> bool {
        true
    }

    #[inline(always)]
    fn map_color(&self, color: &mut Rgba<u8>) {
        let new_color = self.lookup(self.index_of(color)).unwrap();
        let luma = &mut color.0;
        *luma = new_color.0;
    }
}

impl ColorMap for PPMFrameColorMap {
    type Color = Rgba<u8>;

    #[inline(always)]
    fn index_of(&self, color: &Rgba<u8>) -> usize {
        let wrapped = RgbWrapper::new(color[0], color[1], color[2]);

        rgb_to_ppm_frame_pixel(&wrapped)
    }

    #[inline(always)]
    fn lookup(&self, idx: usize) -> Option<Self::Color> {
        match idx {
            0 => {
                let col = PPM_PAPER_COLORS[0];
                Some(Rgba([col.r, col.g, col.b, 255]))
            }
            1 => {
                let col = PPM_PAPER_COLORS[1];
                Some(Rgba([col.r, col.g, col.b, 255]))
            }
            2 => {
                let col = PPM_COLOR_RED;
                Some(Rgba([col.r, col.g, col.b, 255]))
            }
            3 => {
                let col = PPM_COLOR_BLUE;
                Some(Rgba([col.r, col.g, col.b, 255]))
            }
            _ => None,
        }
    }

    /// Indicate NeuQuant implements `lookup`.
    fn has_lookup(&self) -> bool {
        true
    }

    #[inline(always)]
    fn map_color(&self, color: &mut Rgba<u8>) {
        let new_color = self.lookup(self.index_of(color)).unwrap();
        let luma = &mut color.0;
        *luma = new_color.0;
    }
}

pub struct ImageWrapper {
    image: RgbaImage,
}

impl ImageWrapper {
    pub fn new(width: u32, height: u32) -> ImageWrapper {
        ImageWrapper {
            image: RgbaImage::new(width, height),
        }
    }

    pub fn load(path: impl Into<PathBuf>) -> Result<ImageWrapper> {
        let path: PathBuf = path.into();
        let image = image::open(path)?;

        Ok(ImageWrapper {
            image: image.to_rgba8(),
        })
    }

    pub fn save_as(&self, path: impl Into<PathBuf>) -> Result<()> {
        let path: PathBuf = path.into();

        self.image.save(path)?;

        Ok(())
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: &RgbWrapper) -> Result<()> {
        let pixel = self.image.get_pixel_mut(x, y);

        *pixel = image::Rgba([color.r, color.g, color.b, 255]);

        Ok(())
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Result<RgbWrapper> {
        let pixel = self.image.get_pixel(x, y);

        Ok(RgbWrapper::new(pixel[0], pixel[1], pixel[2]))
    }

    pub fn get_pixels(&self) -> Result<Vec<RgbWrapper>> {
        let pixels = self
            .image
            .pixels()
            .map(|p| RgbWrapper::new(p[0], p[1], p[2]))
            .collect();

        Ok(pixels)
    }

    pub fn get_raw_pixels(&self) -> Vec<u8> {
        self.image.as_raw().to_vec()
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<ImageWrapper> {
        Ok(ImageWrapper {
            image: resize(&self.image, width + 1, height + 1, FilterType::Nearest),
        })
    }

    pub fn dither(
        &self,
        dither_type: DitherType,
        color_map: impl ColorMap<Color = Rgba<u8>>,
    ) -> Result<ImageWrapper> {
        let map = match dither_type {
            DitherType::Bayer4x4 => ThresholdMap::level(1),
            DitherType::Bayer8x8 => ThresholdMap::level(2),
        };

        let image = self.image.clone();

        let palletized = imageops::index_colors(&image, &color_map);

        let mapped = ImageBuffer::from_fn(self.image.width(), self.image.height(), |x, y| {
            let p = palletized.get_pixel(x, y);
            color_map
                .lookup(p.0[0] as usize)
                .expect("indexed color out-of-range")
        });

        let image = DynamicImage::ImageRgba8(mapped).ordered_dither(&map);

        Ok(ImageWrapper {
            image: image.to_rgba8(),
        })
    }
}
