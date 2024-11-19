use anyhow::{ensure, Result};
use libflipnote::ppm::{audio::audio::PPMSoundTrackType, file::PPMFile};

const FLIPNOTE_FILE: &[u8] = include_bytes!("../flipnotes/mrjohn.ppm");

pub fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    // Load a PPM file
    let ppm_file = PPMFile::from_bytes(FLIPNOTE_FILE)?;

    // Save the thumbnail as a PNG file
    ppm_file
        .thumbnail
        .get_image()?
        .save_as("/home/sarah/Pictures/mrjohn_thumbnail.png")?;

    // Save all audio (including sound effects) as WAV files
    ppm_file
        .audio
        .get_mixed_sound_track_wav(32768)?
        .save_as("/home/sarah/Music/mrjohn.wav")?;

    // Save only a sound effect
    ppm_file
        .audio
        .get_sound_track_wav(PPMSoundTrackType::SE1, 32768)?
        .save_as("/home/sarah/Music/mrjohn_se1.wav")?;

    // Export the video as an MP4 file. Requires ffmpeg to be installed.
    ppm_file.export_video("/home/sarah/Videos/mrjohn.mp4")?;

    // Verify the signature
    ensure!(
        ppm_file.verify_signature()?,
        "The file signature is invalid"
    );

    // Save the PPM file as a new file
    ppm_file.save_as("/home/sarah/Pictures/mrjohn.ppm")?;

    Ok(())
}
