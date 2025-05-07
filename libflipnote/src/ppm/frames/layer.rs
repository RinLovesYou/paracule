use std::io::Cursor;

use anyhow::{Result, ensure};

use super::line::{LineEncoding, PPMLine};

#[derive(Debug, Clone)]
pub struct PPMLayer {
    pub lines: Vec<PPMLine>,
}

impl Default for PPMLayer {
    fn default() -> Self {
        Self {
            lines: vec![PPMLine::default(); 192],
        }
    }
}

impl PPMLayer {
    pub fn new(encodings: &[LineEncoding]) -> Self {
        let mut layer = Self::default();

        for (i, encoding) in encodings.iter().enumerate() {
            layer.lines[i].encoding = *encoding;
        }

        layer
    }

    pub fn parse(&mut self, cursor: &mut Cursor<&[u8]>) -> Result<()> {
        for line in self.lines.iter_mut() {
            line.parse(cursor)?;
        }
        Ok(())
    }

    pub fn get_data(&self) -> Result<Vec<u8>> {
        let mut data = vec![0u8; 256 * 192];

        for (i, line) in self.lines.iter().enumerate() {
            for (j, pixel) in line.get_data().iter().enumerate() {
                data[i * 256 + j] = *pixel;
            }
        }

        Ok(data)
    }

    pub fn get(&self, x: usize, y: usize) -> Result<bool> {
        ensure!(y < self.lines.len(), "y must be less than 192");

        Ok(self.lines[y].get(x)? != 0)
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> Result<()> {
        ensure!(y < self.lines.len(), "y must be less than 192");

        self.lines[y].set(x, value as u8)?;

        Ok(())
    }

    pub fn apply_diffing(&mut self, x: usize, y: usize, previous_value: u8) -> Result<()> {
        ensure!(y < self.lines.len(), "y must be less than 192");

        self.lines[y].apply_diffing(x, previous_value)?;

        Ok(())
    }
}
