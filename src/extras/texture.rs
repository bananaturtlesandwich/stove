use super::*;
use unreal_asset::{
    engine_version::EngineVersion,
    exports::{ExportBaseTrait, ExportNormalTrait},
    reader::archive_trait::ArchiveTrait,
};

#[test]
fn parse_tex() -> Result<(), unreal_asset::error::Error> {
    let parse = |asset, exp, bulk: Option<_>, name, version| {
        let (_, x, y, rgba) = get_tex_info(
            unreal_asset::Asset::new(
                io::Cursor::new(asset),
                Some(io::Cursor::new(exp)),
                version,
                None,
            )?,
            bulk.map(io::Cursor::new),
        )?;
        let mut image = png::Encoder::new(std::fs::File::create(format!("{name}.png"))?, x, y);
        image.set_color(png::ColorType::Rgba);
        image.set_depth(png::BitDepth::Eight);
        image
            .write_header()
            .unwrap()
            .write_image_data(&rgba)
            .unwrap();
        Ok::<_, unreal_asset::error::Error>(())
    };
    parse(
        include_bytes!("tests/Basic_SplitRGB.uasset").as_slice(),
        include_bytes!("tests/Basic_SplitRGB.uexp").as_slice(),
        Some(include_bytes!("tests/Basic_SplitRGB.ubulk").as_slice()),
        "Basic_SplitRGB",
        EngineVersion::VER_UE4_25,
    )?;
    parse(
        include_bytes!("tests/moon0023.uasset").as_slice(),
        include_bytes!("tests/moon0023.uexp").as_slice(),
        None,
        "moon",
        EngineVersion::VER_UE5_1,
    )?;
    parse(
        include_bytes!("tests/T_WD105_hr_ControlTower_01a_B.uasset").as_slice(),
        include_bytes!("tests/T_WD105_hr_ControlTower_01a_B.uexp").as_slice(),
        None,
        "T_WD105_hr_ControlTower_01a_B",
        EngineVersion::VER_UE4_27,
    )?;
    Ok(())
}

pub fn get_tex_paths<C: io::Read + io::Seek>(mat: unreal_asset::Asset<C>) -> Vec<String> {
    mat.imports
        .iter()
        .filter(|imp| imp.class_name == "Texture2D")
        .filter_map(|imp| mat.get_import(imp.outer_index))
        .filter(|imp| {
            imp.object_name.get_content(|path| {
                !matches!(
                    path,
                    "/Engine/EngineResources/Black"
                        | "/Engine/EngineResources/Black_Low"
                        | "/Engine/EngineResources/DefaultTexture"
                        | "/Engine/EngineResources/DefaultTexture_Low"
                        | "/Engine/EngineMaterials/DefaultWhiteGrid"
                        | "/Engine/EngineMaterials/DefaultWhiteGrid_Low"
                )
            })
        })
        .map(|imp| imp.object_name.get_owned_content())
        .collect()
}

// reference implementations:
// umodel texture export: https://github.com/gildor2/UEViewer/blob/master/Unreal/UnrealMaterial/UnTexture4.cpp#L144
// umodel png exporter: https://github.com/gildor2/UEViewer/blob/master/Unreal/Wrappers/TexturePNG.cpp#L192
// CAS UAssetAPI texture export: https://github.com/LongerWarrior/UEAssetToolkitGenerator/blob/master/UAssetApi/ExportTypes/Texture2DExport.cs#L182
// CAS UAssetAPI decoder: https://github.com/LongerWarrior/UEAssetToolkitGenerator/blob/master/CookedAssetSerializer/Textures/TextureDecoder.cs#L95
/// parses the extra data of the texture export to get data
pub fn get_tex_info<C: io::Read + io::Seek>(
    asset: unreal_asset::Asset<C>,
    bulk: Option<C>,
) -> Result<(bool, u32, u32, Vec<u8>), io::Error> {
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
        // if >= UE5_0 then this is relative to position before reading
        true => data.read_u64::<LE>()?,
        false => data.read_u32::<LE>()? as u64,
    };

    if engine >= EngineVersion::VER_UE5_0 {
        data.set_position(data.position() + 16);
    }

    // x
    data.read_i32::<LE>()?;
    // y
    data.read_i32::<LE>()?;
    let packed = data.read_u32::<LE>()?;
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
    if engine < EngineVersion::VER_UE5_0 && data.read_i32::<LE>()? == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "mip is raw"));
    }
    let bulk = BulkData::new(&mut data, bulk, asset.bulk_data_start_offset)?;
    // x
    let x = data.read_i32::<LE>()? as usize;
    // y
    let y = data.read_i32::<LE>()? as usize;
    // no need to read anything else
    let mut bgra = vec![0; x * y];
    macro_rules! run {
        ($func: ident) => {
            texture2ddecoder::$func(&bulk.data, x, y, &mut bgra)
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
        "PF_B8G8R8A8" => Ok(bgra = bulk
            .data
            .chunks(4)
            // would prefer array_chunks but that's a nightly feature
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()),
        "PF_G8" => Ok(bgra = bulk
            .data
            .into_iter()
            .map(|g| u32::from_le_bytes([g; 4]))
            .collect()),
        _ => Err("currently unsupported soz :p"),
    }
    .map_err(|e: &str| io::Error::new(io::ErrorKind::InvalidInput, format!("{format}: {e}")))?;
    Ok((
        matches!(format.as_str(), "PF_G8" | "PF_BC5"),
        x as u32,
        y as u32,
        bgra.into_iter().flat_map(u32::to_le_bytes).collect(),
    ))
}

const HAS_OPT_DATA: u32 = 1 << 30;
