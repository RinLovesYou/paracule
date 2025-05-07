use anyhow::{ensure, Result};
use libflipnote::{
    ppm::{constants::PPM_AUDIO_PLAYBACK_SAMPLE_RATE, file::PPMFile},
    utils::image_utils::ImageWrapper,
};

const FLIPNOTE_FILE: &[u8] = include_bytes!("../flipnotes/bokeh.ppm");

pub fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    // Load a PPM file
    let mut ppm_file = PPMFile::from_bytes(FLIPNOTE_FILE)?;

    // Replace the thumbnail with another one
    ppm_file
        .thumbnail
        .set_image(&ImageWrapper::load("/home/sarah/Pictures/thumbnail.jpg")?)?;

    // Save the thumbnail as a PNG file
    ppm_file
        .thumbnail
        .get_image()?
        .save_as("/home/sarah/Pictures/bokeh.png")?;

    // Save all audio (including sound effects) as a WAV file
    if let Some(mixed) = ppm_file.audio.mixed_tracks.as_ref() {
        mixed.save_as("/home/sarah/Music/bokeh.wav")?;
    }

    // Save only a sound effect
    if let Some(se1) = ppm_file.audio.sound_effect_1_track.as_ref() {
        se1.save_as("/home/sarah/Music/bokeh_se1.wav")?;
    }

    // Export the video as an MP4 file. Requires ffmpeg to be installed.
    ppm_file.export_video(
        "/home/sarah/Videos/bokeh.mp4",
        PPM_AUDIO_PLAYBACK_SAMPLE_RATE,
    )?;

    // Verify the signature
    ensure!(
        ppm_file.verify_read_signature()?,
        "The file signature is invalid"
    );

    // Save the PPM file as a new file
    ppm_file.save_as("/home/sarah/Pictures/bokeh.ppm")?;

    Ok(())
}
