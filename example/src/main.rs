use anyhow::{ensure, Result};
use libflipnote::{ppm::file::PPMFile, utils::image_utils::ImageWrapper};

const FLIPNOTE_FILE: &[u8] = include_bytes!("../flipnotes/mrjohn.ppm");

pub fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    // Load a PPM file
    let mut ppm_file = PPMFile::from_bytes(FLIPNOTE_FILE)?;
    
    // Replace the thumbnail with another one
    ppm_file.thumbnail.set_image(&ImageWrapper::load(
        "/home/sarah/Pictures/73-1614629524-1044199087.webp",
    )?)?;
    
    // Save the thumbnail as a PNG file
    ppm_file
        .thumbnail
        .get_image()?
        .save_as("/home/sarah/Pictures/mrjohn_thumbnail.png")?;


    // Save all audio (including sound effects) as a WAV file
    if let Some(mixed) = ppm_file.audio.mixed_tracks.as_ref() {
        mixed
            .resample(32768)?
            .save_as("/home/sarah/Music/mrjohn_mixed.wav")?;
    }

    // Save only a sound effect
    if let Some(se1) = ppm_file.audio.sound_effect_1_track.as_ref() {
        se1.resample(32768)?
            .save_as("/home/sarah/Music/mrjohn_se1.wav")?;
    }

    // Export the video as an MP4 file. Requires ffmpeg to be installed.
    ppm_file.export_video("/home/sarah/Videos/mrjohn.mp4", 32768)?;

    // Verify the signature
    ensure!(
        ppm_file.verify_read_signature()?,
        "The file signature is invalid"
    );

    // Save the PPM file as a new file
    ppm_file.save_as("/home/sarah/Pictures/mrjohn.ppm")?;

    Ok(())
}
