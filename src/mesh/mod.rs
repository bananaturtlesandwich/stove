mod parsing;
pub use parsing::*;
use std::io;

use byteorder::ReadBytesExt;

pub struct StripDataFlags {
    pub global: u8,
    pub class: u8,
}

impl StripDataFlags {
    pub fn read(data: &mut io::Cursor<&Vec<u8>>) -> Result<Self, io::Error> {
        Ok(Self {
            global: data.read_u8()?,
            class: data.read_u8()?,
        })
    }
    pub fn editor_data_stripped(&self) -> bool {
        (self.global & 1) != 0
    }

    pub fn data_stripped_for_server(&self) -> bool {
        (self.global & 2) != 0
    }

    pub fn class_data_stripped(&self, flag: u8) -> bool {
        (self.class & flag) != 0
    }
}
