use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{Result, bail, ensure};
use binrw::{BinRead, BinWrite, binrw};
use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey, pkcs8::DecodePublicKey, rand_core};
use sha1_checked::Sha1;

use crate::utils::crypto::hash_data;

use super::{
    audio::audio_data::PPMAudio,
    constants::{FLIPNOTE_STUDIO_PUBLIC_KEY, PPM_FORMAT_VERSION},
    frames::animation_data::PPMAnimationData,
    parsers::{audio_parser, ppm_parser::ppm_parser},
    thumbnail::PPMThumbnail,
    writers::audio_writer,
};

#[binrw]
#[brw(little)]
#[brw(magic = b"PARA")]
#[derive(Debug, Clone, Default)]
pub struct PPMFile {
    #[bw(ignore)]
    #[br(parse_with = ppm_parser)]
    pub original_data: Vec<u8>,
    //File Header
    animation_data_size: u32,
    sound_data_size: u32,
    frame_count: u16,
    format_version: u16, //always 0x24

    //Metadata
    locked_buf: u16, // always 0 or 1, i.e. true or false.
    thumbnail_frame_index: u16,
    root_name_buf: [u8; 22],
    parent_name_buf: [u8; 22],
    child_name_buf: [u8; 22],
    parent_id: u64,
    current_id: u64,
    parent_file_name_buf: [u8; 18],
    current_file_name_buf: [u8; 18],
    root_id: u64,
    root_file_fragment_buf: [u8; 8],
    time_stamp_buf: u32,

    //Thumbnail
    #[brw(pad_before = 2)]
    pub thumbnail: PPMThumbnail,

    // Animation Data
    #[brw(seek_before = std::io::SeekFrom::Start(0x6A0))]
    #[brw(args((frame_count + 1, animation_data_size.to_owned())))]
    pub animation_data: PPMAnimationData,

    //not part of the spec, just a bit more readable to calculate this here instead. Used to calc padding before sound header.
    #[br(calc((0x6A0 + animation_data_size + ((frame_count + 1) as u32)) as u64))]
    #[bw(ignore)]
    _sound_header_start: u64,

    //Sound Data
    #[brw(seek_before = std::io::SeekFrom::Start((0x6A0 + animation_data_size) as u64))]
    //#[brw(args((frame_count + 1, _sound_header_start.to_owned())))]
    #[br(parse_with = audio_parser::audio_parser, args(frame_count + 1, _sound_header_start.to_owned()))]
    #[bw(write_with = audio_writer::audio_writer, args(frame_count + 1, _sound_header_start.to_owned()))]
    pub audio: PPMAudio,

    //Signature
    #[br(count = 0x80)]
    #[brw(pad_after = 0x10)]
    signature: Vec<u8>,
}

impl PPMFile {
    pub fn new() -> Self {
        Self {
            format_version: PPM_FORMAT_VERSION,
            ..Default::default()
        }
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let mut file = File::open(path.into())?;

        let parsed = PPMFile::read(&mut file)?;

        Ok(parsed)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = std::io::Cursor::new(bytes);

        let parsed = PPMFile::read(&mut cursor)?;

        Ok(parsed)
    }

    pub fn save_as(&self, path: impl Into<PathBuf>) -> Result<()> {
        let mut path: PathBuf = path.into();

        if path.extension().is_none() {
            path.set_extension("ppm");
        }

        ensure!(
            path.extension().unwrap() == "ppm",
            "File must have a .ppm extension"
        );

        let mut file = File::create(path)?;

        self.write(&mut file)?;

        Ok(())
    }

    fn get_body(&self) -> Result<Vec<u8>> {
        let mut body = vec![];

        let mut cursor = std::io::Cursor::new(&mut body);

        self.write(&mut cursor)?;

        body.truncate(body.len() - 0x90); //cut off the signature & padding

        Ok(body)
    }

    /// Verifies if the signature is valid, if true, the file can be played back on the official Flipnote Studio app.
    /// This is verified by *writing* the file to a buffer, then hashing the buffer and verifying the signature.
    /// Due to Adpcm being a lossy format, the hash will not match the original file. use [`PPMFile::verify_read_signature`] to verify the signature of the original data.
    pub fn verify_signature(&self) -> Result<bool> {
        let public_key = RsaPublicKey::from_public_key_pem(FLIPNOTE_STUDIO_PUBLIC_KEY)?;

        let hash = hash_data(&self.get_body()?);

        Ok(public_key
            .verify(
                Pkcs1v15Sign::new::<Sha1>(),
                hash.as_slice(),
                self.signature.as_slice(),
            )
            .is_ok())
    }

    /// Verifies the signature of the original data. This is what you should use if you want to verify the signature of the original parsed data.
    /// If you want to verify the signature of the data you have written, use [`PPMFile::verify_signature`]
    pub fn verify_read_signature(&self) -> Result<bool> {
        let mut body = self.original_data.clone();

        body.truncate(body.len() - 0x90); //cut off the signature & padding

        let public_key = RsaPublicKey::from_public_key_pem(FLIPNOTE_STUDIO_PUBLIC_KEY)?;

        let hash = hash_data(&body);

        Ok(public_key
            .verify(
                Pkcs1v15Sign::new::<Sha1>(),
                hash.as_slice(),
                self.signature.as_slice(),
            )
            .is_ok())
    }

    /// Signs the file with the provided private key. Takes a `RsaPrivateKey` from the [`rsa`](https://crates.io/crates/rsa) crate.
    /// The key is not provided for legal reasons. If you have the file, you know what to do with it.
    pub fn sign(&mut self, private_key: &RsaPrivateKey) -> Result<()> {
        let hash = hash_data(&self.get_body()?);

        let signature =
            private_key.sign_with_rng(&mut rand_core::OsRng, Pkcs1v15Sign::new::<Sha1>(), &hash)?;

        ensure!(signature.len() == 0x80, "Signature is not 0x80 bytes long");

        self.signature = signature;

        ensure!(self.verify_signature()?, "Signature is invalid");

        Ok(())
    }

    pub fn export_video(&self, path: impl Into<PathBuf>, audio_sample_rate: i32) -> Result<()> {
        // use the ffmpeg crate to encode the video. for now we only want to render the frames.

        let frames = self.animation_data.get_frames()?;

        let framerate = self.audio.audio_header.get_framerate()?;

        let path: PathBuf = path.into();

        unsafe {
            libc::mkfifo("audio".as_ptr() as *const i8, 0o777);
            libc::mkfifo("video".as_ptr() as *const i8, 0o777);
        };

        let mut audio_file = File::create("audio")?;
        let mut stdin = File::create("video")?;

        let audio = self
            .audio
            .mixed_tracks
            .to_owned()
            .unwrap_or_default()
            .resample(audio_sample_rate)?;

        let wav_data = audio
            .get_samples()
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect::<Vec<u8>>();

        audio_file.write_all(&wav_data)?;

        frames.iter().try_for_each(|f| {
            let image = f.get_image()?;

            let buffer = image.get_raw_pixels();

            stdin.write_all(&buffer)?;

            Result::<_, anyhow::Error>::Ok(())
        })?;

        //ffmpeg command preparation
        let ffmpeg = Command::new("ffmpeg")
            .arg("-y")
            .args(["-f", "rawvideo"])
            .args(["-pix_fmt", "rgba"])
            .args(["-video_size", "256x192"])
            .arg("-framerate")
            .arg((framerate as i32).to_string())
            .args(["-i", "video"])
            .args([
                "-f",
                "s16le",
                "-sample_rate",
                &audio_sample_rate.to_string(),
            ])
            .args(["-i", "audio"])
            .args(["-c:v", "libx264"])
            .arg(path.to_string_lossy().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let output = ffmpeg.wait_with_output()?;

        Command::new("rm").arg("audio").arg("video").spawn()?;

        if !output.status.success() {
            bail!("ffmpeg failed: {:?}", output.status);
        }

        Ok(())
    }
}
