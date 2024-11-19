use anyhow::{ensure, Result};
use binrw::binrw;

use crate::{ppm::constants::{PPM_COLOR_BLUE, PPM_COLOR_RED, PPM_PAPER_COLORS}, utils::image_utils::RgbWrapper};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PPMFrameType {
    #[default]
    Normal,
    Diffed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PPMPaperColor {
    #[default]
    White,
    Black,
}

impl Into<usize> for PPMPaperColor {
    fn into(self) -> usize {
        match self {
            PPMPaperColor::White => 0,
            PPMPaperColor::Black => 1,
        }
    }
}

impl PPMPaperColor {
    pub fn get_rgb_color(&self) -> RgbWrapper {
        PPM_PAPER_COLORS[self.clone() as usize]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PPMLayerColor {
    #[default]
    InverseOfPaper,
    Red,
    Blue
}

impl PPMLayerColor {
    pub fn get_rgb_color(&self, paper_color: &PPMPaperColor) -> RgbWrapper {
        match self {
            PPMLayerColor::InverseOfPaper => match paper_color {
                PPMPaperColor::White => PPM_PAPER_COLORS[1],
                PPMPaperColor::Black => PPM_PAPER_COLORS[0],
            },
            PPMLayerColor::Red => PPM_COLOR_RED,
            PPMLayerColor::Blue => PPM_COLOR_BLUE,
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
pub struct PPMFrameHeader {
    header: u8,
}

impl PPMFrameHeader {
    pub fn get_frame_type(&self) -> PPMFrameType {
        match (self.header >> 7) & 0x1 {
            0 => PPMFrameType::Diffed,
            _ => PPMFrameType::Normal,
        } 
    }

    pub fn set_frame_type(&mut self, frame_type: PPMFrameType) {
        let value: u8 = match frame_type {
            PPMFrameType::Diffed => 0,
            PPMFrameType::Normal => 1,
        };

        self.header = (self.header & 0x7F) | (value << 7);
    }

    pub fn get_is_translated(&self) -> bool {
        match (self.header >> 5) & 0x3 {
            0 => false,
            _ => true,
        }
    }

    pub fn set_is_translated(&mut self, value: bool) {
        let value: u8 = match value {
            true => 1,
            false => 0,
        };

        self.header = (self.header & 0xDF) | (value << 5);
    }

    pub fn get_paper_color(&self) -> PPMPaperColor {
        match self.header & 0x1 {
            0 => PPMPaperColor::Black,
            _ => PPMPaperColor::White
        }
    }

    pub fn set_paper_color(&mut self, paper_color: PPMPaperColor) {
        let value: u8 = match paper_color {
            PPMPaperColor::Black => 0,
            PPMPaperColor::White => 1,
        };

        self.header = (self.header & 0xFE) | value;
    }

    pub fn get_layer_color(&self, layer: u8) -> Result<PPMLayerColor> {
        ensure!(layer > 0 && layer <= 2, "Layer index must be 1 or 2");

        let value = match layer {
            1 => (self.header >> 1) & 0x3,
            2 => (self.header >> 3) & 0x3,
            _ => unreachable!(),
        };

        match value {
            0 | 1 => Ok(PPMLayerColor::InverseOfPaper),
            2 => Ok(PPMLayerColor::Red),
            3 => Ok(PPMLayerColor::Blue),
            _ => unreachable!(),
        }
    }

    pub fn set_layer_color(&mut self, layer: u8, layer_color: PPMLayerColor) -> Result<()> {
        ensure!(layer > 0 && layer <= 2, "Layer index must be 1 or 2");

        let value: u8 = match layer_color {
            PPMLayerColor::InverseOfPaper => 0,
            PPMLayerColor::Red => 2,
            PPMLayerColor::Blue => 3,
        };

        match layer {
            1 => self.header = (self.header & 0xF9) | (value << 1),
            2 => self.header = (self.header & 0xE7) | (value << 3),
            _ => unreachable!(),
        }

        Ok(())
    }

}