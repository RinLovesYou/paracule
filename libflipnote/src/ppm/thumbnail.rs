use std::path::PathBuf;

use anyhow::Result;
use binrw::binrw;

use crate::utils::{
    color_utils::{rgb_to_thumbnail_pixel, thumbnail_pixel_to_rgb},
    image_utils::ImageWrapper,
};

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
pub struct PPMThumbnailTile {
    #[br(count = 32)]
    pixels: Vec<u8>,
}

impl PPMThumbnailTile {
    pub fn get_image(&self) -> Result<ImageWrapper> {
        let mut image = ImageWrapper::new(8, 8);

        for (i, pixel) in self.pixels.iter().enumerate() {
            let colors = thumbnail_pixel_to_rgb(*pixel)?;

            let pixel_x = (i % 4) * 2;
            let pixel_y = i / 4;

            image.set_pixel(pixel_x as u32, pixel_y as u32, colors.0)?;
            image.set_pixel(pixel_x as u32 + 1, pixel_y as u32, colors.1)?;
        }

        Ok(image)
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
    pub fn get_image(&self) -> Result<ImageWrapper> {
        let mut thumbnail = ImageWrapper::new(64, 48);

        for (i, tile) in self.tiles.iter().enumerate() {
            let tile_image = tile.get_image()?;
            let tile_x = i % 8;
            let tile_y = i / 8;

            for (i, pixel) in tile_image.get_pixels()?.iter().enumerate() {
                let pixel_x = i % 8;
                let pixel_y = i / 8;

                thumbnail.set_pixel(
                    tile_x as u32 * 8 + pixel_x as u32,
                    tile_y as u32 * 8 + pixel_y as u32,
                    *pixel,
                )?;
            }
        }

        Ok(thumbnail)
    }

    pub fn set_image(&mut self, image: &ImageWrapper) -> Result<()> {
        let image = image.resize(64, 48)?;

        let mut tiles = vec![PPMThumbnailTile::default(); 48];

        for (i, tile) in tiles.iter_mut().enumerate() {
            let tile_x = i % 8;
            let tile_y = i / 8;

            tile.pixels = vec![0; 32];

            for (j, tile_pixel) in tile.pixels.iter_mut().enumerate() {
                let pixel_x = j % 8;
                let pixel_y = j / 8;

                let pixel1 = image.get_pixel(
                    tile_x as u32 * 8 + pixel_x as u32,
                    tile_y as u32 * 8 + pixel_y as u32,
                )?;
                let pixel2 = image.get_pixel(
                    tile_x as u32 * 8 + pixel_x as u32 + 1,
                    tile_y as u32 * 8 + pixel_y as u32,
                )?;

                *tile_pixel = rgb_to_thumbnail_pixel(&pixel1, &pixel2);
            }
        }

        self.tiles = tiles;

        Ok(())
    }

    pub fn set_image_from_path(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let image = ImageWrapper::load(path)?;

        self.set_image(&image)?;

        Ok(())
    }
}
