use std::io;

use super::*;
use byteorder::{ReadBytesExt, LE};
use unreal_asset::{
    engine_version::EngineVersion,
    exports::{ExportBaseTrait, ExportNormalTrait},
    reader::archive_trait::ArchiveTrait,
};

#[test]
fn parse_tex() -> Result<(), unreal_asset::error::Error> {
    let (x, y, bgra) = get_tex_info(
        unreal_asset::Asset::new(
            io::Cursor::new(include_bytes!("Basic_SplitRGB.uasset").as_slice()),
            Some(io::Cursor::new(
                include_bytes!("Basic_SplitRGB.uexp").as_slice(),
            )),
            EngineVersion::VER_UE4_25,
            None,
        )?,
        Some(io::Cursor::new(
            include_bytes!("Basic_SplitRGB.ubulk").as_slice(),
        )),
    )?;
    let mut rgba: Vec<_> = bgra.into_iter().flat_map(u32::to_le_bytes).collect();
    for i in (0..rgba.len()).step_by(4) {
        rgba.swap(i, i + 2)
    }
    let mut image = png::Encoder::new(
        std::fs::File::create("Basic_SplitRGB.png")?,
        x as u32,
        y as u32,
    );
    image.set_color(png::ColorType::Rgba);
    image.set_depth(png::BitDepth::Eight);
    image
        .write_header()
        .unwrap()
        .write_image_data(&rgba)
        .unwrap();
    Ok(())
}

pub fn get_tex_path<C: io::Read + io::Seek>(mat: unreal_asset::Asset<C>) -> Option<String> {
    mat.imports
        .iter()
        .find(|imp| imp.class_name == "Texture2D")
        .and_then(|imp| mat.get_import(imp.outer_index))
        .map(|imp| imp.object_name.get_owned_content())
}

// reference implementations:
// umodel texture export: https://github.com/gildor2/UEViewer/blob/master/Unreal/UnrealMaterial/UnTexture4.cpp#L144
// umodel png exporter: https://github.com/gildor2/UEViewer/blob/master/Unreal/Wrappers/TexturePNG.cpp#L192
// CAS UAssetAPI texture export: https://github.com/LongerWarrior/UEAssetToolkitGenerator/blob/master/UAssetApi/ExportTypes/Texture2DExport.cs#L182
// CAS UAssetAPI decoder: https://github.com/LongerWarrior/UEAssetToolkitGenerator/blob/master/CookedAssetSerializer/Textures/TextureDecoder.cs#L95
/// parses the extra data of the texture export to get data
pub fn get_tex_info<C: io::Read + io::Seek>(
    asset: unreal_asset::Asset<C>,
    mut bulk: Option<C>,
) -> Result<(u32, u32, Vec<u32>), io::Error> {
    use io::Read;
    // get the static mesh
    let Some(tex) = asset.asset_data.exports.iter().find(|ex| {
        asset
            .get_import(ex.get_base_export().class_index)
            .map(|import| import.object_name == "Texture2D")
            .unwrap_or(false)
    }) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "failed to find texture export",
        ));
    };
    // get the normal export
    let Some(tex) = tex.get_normal_export() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "failed to cast texture data",
        ));
    };
    let engine = asset.get_engine_version();
    let mut data = io::Cursor::new(tex.extras.as_slice());
    // if this isn't read it breaks
    data.read_i32::<LE>()?;
    // umodel impl only has one but the other may be in super::Serialize4
    StripDataFlags::read(&mut data)?;
    StripDataFlags::read(&mut data)?;
    // data isn't cooked
    if data.read_i32::<LE>()? == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "texture is raw",
        ));
    }
    let format = asset.get_owned_name(data.read_i32::<LE>()?);
    // name number
    data.read_i32::<LE>()?;

    // skip offset
    match engine >= EngineVersion::VER_UE4_20 {
        true => data.read_i64::<LE>()?,
        false => data.read_i32::<LE>()? as i64,
    };

    // x
    data.read_i32::<LE>()?;
    // y
    data.read_i32::<LE>()?;
    let packed = data.read_i32::<LE>()?;
    let mut pixel_format = match data.read_i32::<LE>()? {
        len if len.is_negative() => {
            let mut buf = Vec::with_capacity(-len as usize);
            for _ in 0..buf.capacity() {
                buf.push(data.read_u16::<LE>()?);
            }
            String::from_utf16(&buf).unwrap_or_default()
        }
        len => {
            let mut buf = vec![0; len as usize];
            data.read_exact(&mut buf)?;
            String::from_utf8(buf).unwrap_or_default()
        }
    };
    // remove the null byte
    pixel_format.pop();
    if packed & HAS_OPT_DATA == HAS_OPT_DATA {
        // extras
        data.read_u32::<LE>()?;
        // num mips in tail
        data.read_u32::<LE>()?;
    }
    // first mip
    data.read_i32::<LE>()?;
    // ignore len since we're just reading the first mip
    data.read_i32::<LE>()?;
    // data isn't cooked
    if data.read_i32::<LE>()? == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "mip is raw"));
    }
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
        file_offset += asset.bulk_data_start_offset as u64
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
    // x
    let x = data.read_i32::<LE>()? as usize;
    // y
    let y = data.read_i32::<LE>()? as usize;
    // no need to read anything else
    let mut tex = vec![0; x * y];
    macro_rules! run {
        ($func: ident) => {
            texture2ddecoder::$func(&buf, x, y, &mut tex)
        };
    }
    match format.as_str() {
        "PF_DXT1" => run!(decode_bc1),
        "PF_DXT5" => run!(decode_bc3),
        "PF_ASTC_4x4" => run!(decode_astc_4_4),
        "PF_ASTC_6x6" => run!(decode_astc_6_6),
        "PF_ASTC_8x8" => run!(decode_astc_8_8),
        "PF_ASTC_10x10" => run!(decode_astc_10_10),
        "PF_ASTC_12x12" => run!(decode_astc_12_12),
        "PF_BC4" => run!(decode_bc4),
        "PF_BC5" => run!(decode_bc5),
        "PF_BC7" => run!(decode_bc7),
        "PF_ETC1" => run!(decode_etc1),
        "PF_ETC2_RGB" => run!(decode_etc2_rgb),
        "PF_ETC2_RGBA" => run!(decode_etc2_rgba1),
        _ => panic!("{format} not implemented"),
    }
    .map_err(|e: &str| io::Error::new(io::ErrorKind::InvalidInput, format!("{format}: {e}")))?;
    Ok((x as u32, y as u32, tex))
}

const HAS_OPT_DATA: i32 = 1 << 30;

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
