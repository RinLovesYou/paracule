//! Line decoding code adapted from the [PPM File Format Documentation](https://github.com/Flipnote-Collective/flipnote-studio-docs/wiki/PPM-format#line-compression) by the Flipnote Collective.

use std::io::Cursor;

use anyhow::{ensure, Result};
use binrw::BinReaderExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEncoding {
    Skip,
    Coded,
    InvertedCoded,
    #[default]
    Raw,
}

impl From<u8> for LineEncoding {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Skip,
            1 => Self::Coded,
            2 => Self::InvertedCoded,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PPMLine {
    data: Vec<u8>,
    pub encoding: LineEncoding,
}

impl Default for PPMLine {
    fn default() -> Self {
        Self {
            data: vec![0u8; 256],
            encoding: LineEncoding::default(),
        }
    }
}

impl PPMLine {
    pub fn new(encoding: LineEncoding) -> Self {
        Self {
            data: vec![0u8; 256],
            encoding,
        }
    }

    pub fn parse(&mut self, cursor: &mut Cursor<&[u8]>) -> Result<()> {
        match self.encoding {
            LineEncoding::Skip => Ok(()),
            LineEncoding::Coded => self.parse_coded_line(cursor, false),
            LineEncoding::InvertedCoded => self.parse_coded_line(cursor, true),
            LineEncoding::Raw => self.parse_raw_line(cursor),
        }
    }

    pub fn parse_coded_line(&mut self, cursor: &mut Cursor<&[u8]>, inverted: bool) -> Result<()> {
        let mut chunk_flags = cursor.read_be::<u32>()?;
        let mut pixel = 0;

        if inverted {
            self.data = vec![1u8; 256];
        }

        while chunk_flags & 0xFFFFFFFF != 0 {
            if chunk_flags & 0x80000000 != 0 {
                let chunk = cursor.read_le::<u8>()?;

                for bit in 0..8 {
                    self.data[pixel] = chunk >> bit & 0x1;
                    pixel += 1;
                }
            } else {
                //no data, skip this chunk
                pixel += 8;
            }

            chunk_flags <<= 1;
        }

        Ok(())
    }

    pub fn parse_raw_line(&mut self, cursor: &mut Cursor<&[u8]>) -> Result<()> {
        let mut pixel = 0;

        while pixel < 256 {
            let chunk = cursor.read_le::<u8>()?;
            for bit in 0..8 {
                self.data[pixel] = chunk >> bit & 0x1;
                pixel += 1;
            }
        }
        Ok(())
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn get(&self, x: usize) -> Result<u8> {
        ensure!(x < self.data.len(), "x must be less than the line length");
        Ok(self.data[x])
    }

    pub fn set(&mut self, x: usize, value: u8) -> Result<()> {
        ensure!(x < self.data.len(), "x must be less than the line length");
        self.data[x] = value;
        Ok(())
    }

    pub fn apply_diffing(&mut self, x: usize, previous_data: u8) -> Result<()> {
        ensure!(x < self.data.len(), "x must be less than the line length");

        self.data[x] ^= previous_data;

        Ok(())
    }
}
