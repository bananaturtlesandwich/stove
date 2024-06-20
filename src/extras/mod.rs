mod mesh;
mod texture;

use std::io;

use byteorder::{ReadBytesExt, LE};
pub use mesh::*;
pub use texture::*;

#[derive(Default)]
struct StripDataFlags {
    pub global: u8,
    pub class: u8,
}

impl StripDataFlags {
    fn read(data: &mut io::Cursor<&[u8]>) -> Result<Self, io::Error> {
        Ok(Self {
            global: data.read_u8()?,
            class: data.read_u8()?,
        })
    }

    fn editor_data_stripped(&self) -> bool {
        (self.global & 1) != 0
    }

    fn data_stripped_for_server(&self) -> bool {
        (self.global & 2) != 0
    }

    fn class_data_stripped(&self, flag: u8) -> bool {
        (self.class & flag) != 0
    }
}

struct BulkData {
    data: Vec<u8>,
}

impl BulkData {
    fn new<R: io::Read + io::Seek>(
        data: &mut io::Cursor<&[u8]>,
        mut bulk: Option<R>,
        data_offset: i64,
    ) -> io::Result<Self> {
        use io::Read;
        // bulk data flags
        let mut flags = BulkDataFlags::from_bits_truncate(data.read_u32::<LE>()?);
        let len = match flags.intersects(BulkDataFlags::Size64Bit) {
            true => data.read_i64::<LE>()? as usize,
            false => data.read_i32::<LE>()? as usize,
        };
        // size on disk
        match flags.intersects(BulkDataFlags::Size64Bit) {
            true => data.read_u64::<LE>()?,
            false => data.read_u32::<LE>()? as u64,
        };
        let mut file_offset = data.read_u64::<LE>()?;
        if !flags.intersects(BulkDataFlags::NoOffsetFixUp) {
            file_offset = (file_offset as i64 + data_offset) as u64
        }
        if flags.intersects(BulkDataFlags::BadDataVersion) {
            // idk
            data.read_i16::<LE>()?;
            flags &= !BulkDataFlags::BadDataVersion;
        }
        let mut buf = vec![0; len];
        match flags {
            flags if len == 0 || flags.intersects(BulkDataFlags::Unused) => (),
            flags if flags.intersects(BulkDataFlags::ForceInlinePayload) => {
                data.read_exact(&mut buf)?;
            }
            flags
                if flags.intersects(
                    BulkDataFlags::OptionalPayload | BulkDataFlags::PayloadInSeperateFile,
                ) =>
            {
                let Some(bulk) = bulk.as_mut() else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "texture is raw",
                    ));
                };
                bulk.seek(io::SeekFrom::Start(file_offset))?;
                bulk.read_exact(&mut buf)?;
            }
            flags if flags.intersects(BulkDataFlags::PayloadAtEndOfFile) => {
                let cur = data.position();
                data.set_position(file_offset);
                data.read_exact(&mut buf)?;
                data.set_position(cur);
            }
            _ => (),
        }
        Ok(Self { data: buf })
    }
}

bitflags::bitflags! {
    struct BulkDataFlags: u32 {
        const PayloadAtEndOfFile = 0x0001;
        const CompressedZlib = 0x0002;
        const Unused = 0x0020;
        const ForceInlinePayload = 0x0040;
        const PayloadInSeperateFile = 0x0100;
        const SerializeCompressedBitWindow = 0x0200;
        const OptionalPayload = 0x0800;
        const Size64Bit = 0x2000;
        const BadDataVersion = 0x8000;
        const NoOffsetFixUp = 0x10000;
    }
}
