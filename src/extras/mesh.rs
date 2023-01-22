use std::io;

use byteorder::{ReadBytesExt, LE};
use unreal_asset::{
    engine_version::EngineVersion,
    exports::{ExportBaseTrait, ExportNormalTrait},
    object_version::ObjectVersion,
    reader::asset_trait::AssetTrait,
    types::vector::Vector,
    Asset,
};

#[test]
fn parse_mesh() {
    let mut asset = Asset::new(
        include_bytes!("A02_Outside_Castle.uasset").to_vec(),
        Some(include_bytes!("A02_Outside_Castle.uexp").to_vec()),
    );
    asset.set_engine_version(EngineVersion::VER_UE4_25);
    asset.parse_data().unwrap();
    get_mesh_verts(asset).unwrap();
}

/// parses the extra data of the static mesh export to get vertices
pub fn get_mesh_verts(asset: Asset) -> Result<Vec<Vector<f32>>, io::Error> {
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
        data.read_i32::<LE>()?;
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
                "mesh data is stripped",
            ));
        }
    }
    // stride
    data.read_i32::<LE>()?;
    // vertex count
    data.read_i32::<LE>()?;
    // size
    data.read_i32::<LE>()?;
    // finally the vertex positions!
    let mut buf = Vec::with_capacity(data.read_i32::<LE>()? as usize);
    for _ in 0..buf.capacity() {
        buf.push(Vector::new(
            data.read_f32::<LE>()?,
            data.read_f32::<LE>()?,
            data.read_f32::<LE>()?,
        ));
    }
    Ok(buf)
}
