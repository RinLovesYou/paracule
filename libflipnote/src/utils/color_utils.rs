use anyhow::{ensure, Result};
use photon_rs::Rgb;

use crate::ppm::constants::PPM_THUMBNAIL_COLORS;

pub fn hex_to_rgb(hex: &str) -> Result<Rgb> {
    ensure!(hex.len() == 7, "Invalid hex color length");
    ensure!(hex.starts_with("#"), "Invalid hex color format");

    let hex = &hex[1..];
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok(Rgb::new(r, g, b))
}

pub fn thumbnail_pixel_to_rgb(pixel: u8) -> Result<(Rgb, Rgb)> {
    let color1 = hex_to_rgb(PPM_THUMBNAIL_COLORS[(pixel & 0x0F) as usize])?;
    let color2 = hex_to_rgb(PPM_THUMBNAIL_COLORS[((pixel >> 4) & 0x0F) as usize])?;

    Ok((color1, color2))
}