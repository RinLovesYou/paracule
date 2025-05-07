use anyhow::{Result, ensure};
use binrw::binrw;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
pub struct PPMAnimationFlags {
    flags: u16,
}

impl PPMAnimationFlags {
    pub fn get_loop(&self) -> bool {
        self.flags & 0x2 != 0
    }

    pub fn set_loop(&mut self, value: bool) {
        match value {
            true => self.flags |= 0x2,
            false => self.flags &= !0x2,
        }
    }

    pub fn get_hide_layer(&self, layer: u8) -> Result<bool> {
        ensure!(layer > 0 && layer <= 2, "Layer index must be 1 or 2");

        match layer {
            1 => Ok(self.flags & 0x10 != 0),
            2 => Ok(self.flags & 0x20 != 0),
            _ => unreachable!(),
        }
    }

    pub fn set_hide_layer(&mut self, layer: u8, value: bool) -> Result<()> {
        ensure!(layer > 0 && layer <= 2, "Layer index must be 1 or 2");

        match layer {
            1 => match value {
                true => self.flags |= 0x10,
                false => self.flags &= !0x10,
            },
            2 => match value {
                true => self.flags |= 0x20,
                false => self.flags &= !0x20,
            },
            _ => unreachable!(),
        }

        Ok(())
    }
}
