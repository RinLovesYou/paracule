use anyhow::{ensure, Result};

use crate::ppm::constants::PPM_THUMBNAIL_COLORS;

use super::image_utils::RgbWrapper;

pub fn hex_to_rgb(hex: &str) -> Result<RgbWrapper> {
    ensure!(hex.len() == 7, "Invalid hex color length");
    ensure!(hex.starts_with("#"), "Invalid hex color format");

    let hex = &hex[1..];
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok(RgbWrapper::new(r, g, b))
}

pub fn thumbnail_pixel_to_rgb(pixel: u8) -> Result<(RgbWrapper, RgbWrapper)> {
    let color1 = hex_to_rgb(PPM_THUMBNAIL_COLORS[(pixel & 0x0F) as usize])?;
    let color2 = hex_to_rgb(PPM_THUMBNAIL_COLORS[((pixel >> 4) & 0x0F) as usize])?;

    Ok((color1, color2))
}

pub fn rgb_to_thumbnail_pixel(color1: &RgbWrapper, color2: &RgbWrapper) -> u8 {
    let mut indexes = [0; 2];

    for (i, color) in [color1, color2].iter().enumerate() {
        let mut closest_distance = f32::MAX;
        let mut closest_index = 0;

        for (j, thumbnail_color) in PPM_THUMBNAIL_COLORS.iter().enumerate() {
            let thumbnail_color = hex_to_rgb(thumbnail_color).unwrap();
            let distance = color.distance(&thumbnail_color);

            if distance < closest_distance {
                closest_distance = distance;
                closest_index = j;
            }
        }

        indexes[i] = closest_index;
    }

    ((0xF0 & (indexes[1] << 4)) | (0x0F & indexes[0])) as u8
}
