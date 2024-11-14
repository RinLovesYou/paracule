use std::{fs::File, path::PathBuf};

use anyhow::{ensure, Result};
use binrw::{binrw, BinRead, BinWrite};

use super::{constants::PPM_FORMAT_VERSION, thumbnail::PPMThumbnail};

#[binrw]
#[brw(little)]
#[brw(magic = b"PARA")]
#[derive(Debug, Clone, Default)]
pub struct PPMFile {
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

    #[brw(seek_before = std::io::SeekFrom::Start(0x6A0))]
    frame_offset_table_size: u16,
    #[brw(pad_before = 4)] //unknown, always seen as 0 so we just pad instead.
    animation_flags: u16,

    //Frame Data
    #[br(count = frame_count + 1)]
    animation_offsets: Vec<u32>,
    #[brw(seek_before = std::io::SeekFrom::Start((0x6A8 + frame_offset_table_size) as u64))]
    #[br(count = animation_data_size)]
    animation_data: Vec<u8>,

    //Sound Effect Flags
    #[brw(seek_before = std::io::SeekFrom::Start((0x6A0 + animation_data_size) as u64))]
    #[br(count = frame_count + 1)]
    sound_effect_flags: Vec<u8>,

    //not part of the spec, just a bit more readable to calculate this here instead. Used to calc padding before sound header.
    #[br(calc((0x6A0 + animation_data_size + ((frame_count + 1) as u32)) as u64))]
    #[bw(ignore)]
    _sound_header_start: u64,

    //Sound Header
    #[brw(pad_before = (4 - _sound_header_start % 4) % 4)]
    bgm_track_size: u32,
    se1_track_size: u32,
    se2_track_size: u32,
    se3_track_size: u32,
    frame_playback_speed: u8,
    frame_playback_speed_when_recording: u8,

    //Sound Data
    #[brw(pad_before = 14)]
    #[br(count = bgm_track_size)]
    raw_bgm_track: Vec<u8>,
    #[br(count = se1_track_size)]
    raw_se1_track: Vec<u8>,
    #[br(count = se2_track_size)]
    raw_se2_track: Vec<u8>,
    #[br(count = se3_track_size)]
    raw_se3_track: Vec<u8>,

    //Signature
    #[br(count = 0x80)]
    #[brw(pad_after = 0x10)]
    signature: Vec<u8>,
}

impl PPMFile {
    pub fn new() -> Self {
        let mut file = Self::default();
        file.format_version = PPM_FORMAT_VERSION;

        file
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let mut file = File::open(path.into())?;

        let parsed = PPMFile::read(&mut file)?;

        Ok(parsed)
    }

    pub fn save_as(&self, path: impl Into<PathBuf>) -> Result<()> {
        let mut path: PathBuf = path.into();

        if path.extension().is_none() {
            path.set_extension("ppm");
        }

        ensure!(path.extension().unwrap() == "ppm", "File must have a .ppm extension");

        let mut file = File::create(path)?;

        self.write(&mut file)?;

        Ok(())
    }
}