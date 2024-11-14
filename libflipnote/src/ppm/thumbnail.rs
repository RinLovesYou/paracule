use std::path::PathBuf;

use anyhow::Result;
use binrw::binrw;
use photon_rs::{native::save_image, PhotonImage};

use crate::utils::color_utils::thumbnail_pixel_to_rgb;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
pub struct PPMThumbnailTile {
    #[br(count = 32)]
    pixels: Vec<u8>,
}

impl PPMThumbnailTile {
    pub fn get_raw_pixels(&self) -> Result<Vec<u8>> {

        // One tile represents an 8x8 pixel area. Photon, our image library, expects an array of RGBA values, i.e. 4 bytes per pixel.
        let mut raw_pixels = vec![0; 8 * 8 * 4];

        for (i, pixel) in self.pixels.iter().enumerate() {
            // Thumbnails have a preset color palette, so instead of storing actual color information, PPM files store an index to the color palette.
            // Therefore, two pixel colors can be compressed into a single byte.
            let colors = thumbnail_pixel_to_rgb(*pixel)?;

            let pixel_index = i * 8;

            raw_pixels[pixel_index..pixel_index + 8].copy_from_slice(&[
                colors.0.get_red(), colors.0.get_green(), colors.0.get_blue(), 255,
                colors.1.get_red(), colors.1.get_green(), colors.1.get_blue(), 255,
            ]);
        }

        Ok(raw_pixels)
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
pub struct PPMThumbnail {
    #[br(count = 48)]
    tiles: Vec<PPMThumbnailTile>,
}

impl PPMThumbnail {
    pub fn get_image(&self) -> Result<PhotonImage> {
        let mut raw_pixels = vec![0; 64 * 48 * 4];

        for (i, tile) in self.tiles.iter().enumerate() {
            let tile_raw_pixels = tile.get_raw_pixels()?;
            let tile_x = i % 8;
            let tile_y = i / 8;

            for (j, pixel) in tile_raw_pixels.chunks(4).enumerate() {
                let pixel_x = j % 8;
                let pixel_y = j / 8;

                let pixel_index = ((tile_y * 8 + pixel_y) * 64 + tile_x * 8 + pixel_x) * 4;
                raw_pixels[pixel_index..pixel_index + 4].copy_from_slice(pixel);
            }
        }

        let image = PhotonImage::new(raw_pixels, 64, 48);

        Ok(image)
    }

    pub fn save_image_as(&self, path: impl Into<std::path::PathBuf>) -> Result<()> {
        let path: PathBuf = path.into();
        let image = self.get_image()?;
        
        save_image(image, &path.to_string_lossy().to_string())?;

        Ok(())
    }
}
