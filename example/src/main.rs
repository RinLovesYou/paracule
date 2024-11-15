use anyhow::{ensure, Result};
use libflipnote::ppm::file::PPMFile;

const FLIPNOTE_FILE: &[u8] = include_bytes!("../flipnotes/bokeh.ppm");

fn main() -> Result<()> {
    // Load a PPM file
    let ppm_file = PPMFile::from_bytes(FLIPNOTE_FILE)?;

    // Save the thumbnail as a PNG file
    ppm_file
        .thumbnail
        .get_image()?
        .save_as("/home/sarah/Pictures/bokeh_thumbnail.png")?;

    // Verify the signature
    ensure!(
        ppm_file.verify_signature()?,
        "The file signature is invalid"
    );

    // Save the PPM file as a new file
    ppm_file.save_as("/home/sarah/Pictures/bokeh.ppm")?;

    Ok(())
}
