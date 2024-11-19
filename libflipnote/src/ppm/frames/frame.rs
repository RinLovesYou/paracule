use std::io::{Cursor, Read};

use anyhow::Result;
use binrw::BinReaderExt;

use crate::utils::image_utils::ImageWrapper;

use super::{
    animation_flags::PPMAnimationFlags,
    frame_header::{PPMFrameHeader, PPMFrameType},
    layer::PPMLayer,
    line::LineEncoding,
};

#[derive(Debug, Clone, Default)]
pub struct PPMFrame {
    header: PPMFrameHeader,
    translate_x: i8,
    translate_y: i8,

    layers: [PPMLayer; 2],

    hide_layer_1: bool,
    hide_layer_2: bool,
}

impl PPMFrame {
    pub fn parse(
        cursor: &mut Cursor<&[u8]>,
        flags: &PPMAnimationFlags,
        previous_frame: Option<PPMFrame>,
    ) -> Result<Self> {
        let mut frame = Self::default();
        frame.hide_layer_1 = flags.get_hide_layer(1)?;
        frame.hide_layer_2 = flags.get_hide_layer(2)?;

        frame.header = cursor.read_le()?;

        if frame.header.get_is_translated() {
            frame.translate_x = cursor.read_le()?;
            frame.translate_y = cursor.read_le()?;
        }

        for layer in frame.layers.iter_mut() {
            let mut encodings_compressed = [0u8; 0x30];
            cursor.read(&mut encodings_compressed)?;

            let mut encoding_index = 0;

            for byte in encodings_compressed.iter() {
                let mut bit_offset = 0;

                while bit_offset < 8 {
                    layer.lines[encoding_index].encoding =
                        LineEncoding::from((byte >> bit_offset) & 0x3);
                    encoding_index += 1;
                    bit_offset += 2;
                }
            }
        }

        for layer in frame.layers.iter_mut() {
            layer.parse(cursor)?;
        }

        if let Some(previous_frame) = previous_frame {
            frame.decode_diffing(&previous_frame)?;
        }

        Ok(frame)
    }

    pub fn decode_diffing(&mut self, previous_frame: &PPMFrame) -> Result<()> {
        if self.header.get_frame_type() != PPMFrameType::Diffed {
            return Ok(());
        }

        let translate_y = self.translate_y as usize;
        let translate_x = self.translate_x as usize;

        for y in 0..192 {
            if y - translate_y >= 192 {
                break;
            }

            for x in 0..256 {
                if x - translate_x >= 256 {
                    break;
                }

                for layer in 0..2 {
                    let previous_value =
                        previous_frame.layers[layer].get(x - translate_x, y - translate_y)?;

                    self.layers[layer].apply_diffing(x, y, previous_value as u8)?;
                }
            }
        }

        Ok(())
    }

    pub fn get_image(&self) -> Result<ImageWrapper> {
        let mut image = ImageWrapper::new(256, 192);

        let paper_color = self.header.get_paper_color();

        let layer_1_color = self.header.get_layer_color(1)?.get_rgb_color(&paper_color);
        let layer_2_color = self.header.get_layer_color(2)?.get_rgb_color(&paper_color);
        let paper_color = paper_color.get_rgb_color();

        for y in 0..192 {
            for x in 0..256 {
                image.set_pixel(x, y, &paper_color)?;
                let layer_1 = self.layers[0].get(x as usize, y as usize)?;
                let layer_2 = self.layers[1].get(x as usize, y as usize)?;

                if !self.hide_layer_2 && layer_2 {
                    image.set_pixel(x, y, &layer_2_color)?;
                }

                //top layer is stored first.
                if !self.hide_layer_1 && layer_1 {
                    image.set_pixel(x, y, &layer_1_color)?;
                }
            }
        }

        Ok(image)
    }
}
