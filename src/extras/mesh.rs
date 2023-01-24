use std::io;

use byteorder::{ReadBytesExt, LE};
use unreal_asset::{
    engine_version::EngineVersion,
    exports::{ExportBaseTrait, ExportNormalTrait},
    object_version::ObjectVersion,
    reader::asset_trait::AssetTrait,
    Asset,
};

#[test]
fn parse_mesh() -> Result<(), unreal_asset::error::Error> {
    let mut asset = Asset::new(
        include_bytes!("A02_Outside_Castle.uasset").to_vec(),
        Some(include_bytes!("A02_Outside_Castle.uexp").to_vec()),
    );
    asset.set_engine_version(EngineVersion::VER_UE4_25);
    asset.parse_data()?;
    get_mesh_verts(asset)?;
    Ok(())
}

/// parses the extra data of the static mesh export to get vertex positions
pub fn get_mesh_verts(
    asset: Asset,
) -> Result<(Vec<glam::Vec3>, Vec<glam::Vec4>, Indices), io::Error> {
    // get the static mesh
    let Some(mesh) = asset
        .exports
        .iter()
        .find(|ex| {
            asset.get_import(ex.get_base_export().class_index)
                .map(|import| &import.object_name.content == "StaticMesh")
                .unwrap_or(false)
        })
        else {
            return Err(
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                     "failed to find mesh export"
                    )
                );
        };
    // get the normal export
    let Some(mesh) = mesh.get_normal_export()
        else {
            return Err(
                io::Error::new(
                io::ErrorKind::InvalidInput,
                "failed to cast mesh data"
                )
            );
        };
    let mut data = io::Cursor::new(&mesh.extras);
    // padding
    data.read_i32::<LE>()?;
    if !super::StripDataFlags::read(&mut data)?.editor_data_stripped()
        // data isn't cooked
        || data.read_i32::<LE>()? == 0
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "mesh is raw"));
    }
    // bodysetup reference
    data.read_i32::<LE>()?;
    // nav collision reference
    if asset.object_version >= ObjectVersion::VER_UE4_STATIC_MESH_STORE_NAV_COLLISION {
        data.read_i32::<LE>()?;
    }
    // lighting guid
    for _ in 0..16 {
        data.read_u8()?;
    }
    // array of socket references
    for _ in 0..data.read_i32::<LE>()? {
        data.read_i32::<LE>()?;
    }
    // KeepMobileMinLODSettingOnDesktop
    if asset.get_engine_version() >= EngineVersion::VER_UE4_27 {
        data.read_i32::<LE>()?;
    }
    // array of lod resources
    // discard len because we'll just read the first entry
    data.read_i32::<LE>()?;
    let flags = super::StripDataFlags::read(&mut data)?;
    // array of sections
    for _ in 0..data.read_i32::<LE>()? {
        // mat index
        data.read_i32::<LE>()?;
        // first index
        data.read_i32::<LE>()?;
        // tri count
        data.read_i32::<LE>()?;
        // min vertex index
        data.read_i32::<LE>()?;
        // max vertex index
        data.read_i32::<LE>()?;
        // collides
        data.read_i32::<LE>()?;
        // casts shadow
        data.read_i32::<LE>()?;
        // force opaque
        if asset.get_engine_version() >= EngineVersion::VER_UE4_25 {
            data.read_i32::<LE>()?;
        }
        //visible in ray tracing
        if asset.get_engine_version() <= EngineVersion::VER_UE4_26 {
            data.read_i32::<LE>()?;
        }
    }
    // max deviation
    data.read_f32::<LE>()?;
    match asset.get_engine_version() >= EngineVersion::VER_UE4_23 {
        true if !flags.data_stripped_for_server()
        // lod isn't cooked out
        && data.read_i32::<LE>()? == 0
        // data is inlined
        && data.read_i32::<LE>()? == 1 =>
        {
            super::StripDataFlags::read(&mut data)?;
        }
        false if !flags.data_stripped_for_server() && !flags.class_data_stripped(2) => (),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "vertex position data is stripped",
            ));
        }
    }
    // stride
    data.read_i32::<LE>()?;
    // vertex count
    data.read_i32::<LE>()?;
    // size
    data.read_i32::<LE>()?;
    // vertex positions!
    let mut positions = Vec::with_capacity(data.read_i32::<LE>()? as usize);
    for _ in 0..positions.capacity() {
        positions.push(glam::vec3(
            data.read_f32::<LE>()?,
            data.read_f32::<LE>()?,
            data.read_f32::<LE>()?,
        ));
    }
    // static mesh vertex buffer
    if match asset.get_object_version()
        >= ObjectVersion::VER_UE4_STATIC_SKELETAL_MESH_SERIALIZATION_FIX
    {
        true => super::StripDataFlags::read(&mut data)?,
        false => super::StripDataFlags::default(),
    }
    .data_stripped_for_server()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "vertex data is stripped",
        ));
    }
    let num_tex_coords = data.read_i32::<LE>()?;
    // strides
    if asset.get_engine_version() < EngineVersion::VER_UE4_19 {
        data.read_i32::<LE>()?;
    }
    // num verts
    let num_verts = data.read_i32::<LE>()?;
    let full_precision_uvs = data.read_i32::<LE>()? != 0;
    let high_precision_tangent =
        asset.get_engine_version() >= EngineVersion::VER_UE4_12 && data.read_i32::<LE>()? != 0;
    match asset.get_engine_version() >= EngineVersion::VER_UE4_20 {
        true => {
            // item size
            data.read_i32::<LE>()?;
            // item count
            data.read_i32::<LE>()?;
            // packed normals
            for _ in 0..num_verts {
                match high_precision_tangent {
                    true => {
                        for _ in 0..2 {
                            for _ in 0..4 {
                                data.read_u16::<LE>()?;
                            }
                        }
                    }
                    false => {
                        for _ in 0..2 {
                            data.read_u32::<LE>()?;
                        }
                    }
                }
            }
            // item size
            data.read_i32::<LE>()?;
            // item count
            data.read_i32::<LE>()?;
            // mesh uv
            for _ in 0..num_verts {
                for _ in 0..num_tex_coords {
                    for _ in 0..2 {
                        match full_precision_uvs {
                            true => {
                                data.read_f32::<LE>()?;
                            }

                            false => {
                                data.read_u16::<LE>()?;
                            }
                        }
                    }
                }
            }
        }
        false => todo!("read bulk array of uvs"),
    }
    // color vertex buffer
    if super::StripDataFlags::read(&mut data)?.data_stripped_for_server() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "colour data is stripped",
        ));
    }
    // stride
    data.read_i32::<LE>()?;
    // num verts
    data.read_i32::<LE>()?;
    // size
    data.read_i32::<LE>()?;
    // vertex colours
    let mut colours = Vec::with_capacity(data.read_i32::<LE>()? as usize);
    for _ in 0..colours.capacity() {
        let argb = data.read_i32::<LE>()? as i64;
        colours.push(
            glam::vec4(
                ((argb & 0xFF000000) >> 24) as f32,
                ((argb & 0x00FF0000) >> 16) as f32,
                ((argb & 0x0000FF00) >> 8) as f32,
                (argb & 0x000000FF) as f32,
            ) / 255.0,
        )
    }

    let indices = match asset.get_object_version()
        >= ObjectVersion::VER_UE4_SUPPORT_32BIT_STATIC_MESH_INDICES
    {
        true => {
            let use32bits = data.read_i32::<LE>()? != 0;
            // size
            data.read_i32::<LE>()?;
            match use32bits {
                true => {
                    let mut indices = Vec::with_capacity(data.read_i32::<LE>()? as usize / 4);
                    for _ in 0..indices.capacity() {
                        indices.push(data.read_u32::<LE>()?);
                    }
                    Indices::U32(indices)
                }
                false => {
                    let mut indices = Vec::with_capacity(data.read_i32::<LE>()? as usize / 2);
                    for _ in 0..indices.capacity() {
                        indices.push(data.read_u16::<LE>()?);
                    }
                    Indices::U16(indices)
                }
            }
        }
        false => {
            //size
            data.read_i32::<LE>()?;
            let mut indices = Vec::with_capacity(data.read_i32::<LE>()? as usize);
            for _ in 0..indices.capacity() {
                indices.push(data.read_u16::<LE>()?);
            }
            Indices::U16(indices)
        }
    };
    Ok((positions, colours, indices))
}

pub enum Indices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}
