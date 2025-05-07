use std::io::{Cursor, Seek};

use anyhow::Result;
use binrw::binrw;

use super::{animation_flags::PPMAnimationFlags, frame::PPMFrame};

#[binrw]
#[brw(little)]
//args in order: frame_count: u16, animation_data_size: u32
#[brw(import(args: (u16, u32)))]
#[derive(Debug, Clone, Default)]
pub struct PPMAnimationData {
    frame_offset_table_size: u16,
    #[brw(pad_before = 4)] //unknown, always seen as 0 so we just pad instead.
    animation_flags: PPMAnimationFlags,

    //Frame Data
    #[br(count = args.0)]
    animation_offsets: Vec<u32>,

    #[brw(seek_before = std::io::SeekFrom::Start(0x6A8 + frame_offset_table_size.to_owned() as u64))]
    #[br(count = args.1)]
    animation_data: Vec<u8>,
}

impl PPMAnimationData {
    pub fn get_frames(&self) -> Result<Vec<PPMFrame>> {
        let mut frames = Vec::new();

        let mut previous_frame = None;

        let mut cursor = Cursor::new(self.animation_data.as_slice());

        for offset in self.animation_offsets.iter() {
            cursor.seek(std::io::SeekFrom::Start(*offset as u64))?;
            let frame = PPMFrame::parse(&mut cursor, &self.animation_flags, previous_frame)?;

            previous_frame = Some(frame.clone());
            frames.push(frame);
        }

        Ok(frames)
    }
}
