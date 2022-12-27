use std::io;

use byteorder::{LittleEndian, ReadBytesExt};
use unreal_asset::{
    engine_version::EngineVersion,
    exports::{ExportBaseTrait, ExportNormalTrait},
    object_version::ObjectVersion,
    reader::asset_trait::AssetTrait,
    Asset,
};

#[test]
fn parse_mesh() {
    let mut asset = Asset::new(
        include_bytes!("A02_Tutorial_Fog.uasset").to_vec(),
        Some(include_bytes!("A02_Tutorial_Fog.uexp").to_vec()),
    );
    asset.set_engine_version(EngineVersion::VER_UE4_25);
    asset.parse_data().unwrap();
    get_mesh_verts(asset).unwrap();
}

/// parses the extra data of the static mesh export to get vertices
pub fn get_mesh_verts(asset: Asset) -> Result<Vec<f32>, io::Error> {
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
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "failed to find mesh export"));
        };
    // get the normal export
    let Some(mesh) = mesh.get_normal_export()
        else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "failed to cast mesh data"));
        };
    let mut data = io::Cursor::new(dbg!(&mesh.extras));
    // padding
    data.read_i32::<LittleEndian>()?;
    if !super::StripDataFlags::read(&mut data)?.editor_data_stripped()
        // data isn't cooked
        || data.read_i32::<LittleEndian>()? == 0
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "mesh is raw"));
    }
    // bodysetup reference
    data.read_i32::<LittleEndian>()?;
    // nav collision reference
    if asset.object_version >= ObjectVersion::VER_UE4_STATIC_MESH_STORE_NAV_COLLISION {
        data.read_i32::<LittleEndian>()?;
    }
    // lighting guid
    for _ in 0..16 {
        data.read_u8()?;
    }
    // array of socket references
    for _ in 0..data.read_i32::<LittleEndian>()? {
        data.read_i32::<LittleEndian>()?;
    }
    // KeepMobileMinLODSettingOnDesktop
    if asset.get_engine_version() >= EngineVersion::VER_UE4_27 {
        data.read_i32::<LittleEndian>()?;
    }
    // array of lod resources
    // discard len because just first entry
    data.read_i32::<LittleEndian>()?;
    let flags = super::StripDataFlags::read(&mut data)?;
    // array of sections
    let length = data.read_i32::<LittleEndian>()?;
    for _ in 0..length {
        // mat index
        data.read_i32::<LittleEndian>()?;
        // first index
        data.read_i32::<LittleEndian>()?;
        // tri count
        data.read_i32::<LittleEndian>()?;
        // min vertex index
        data.read_i32::<LittleEndian>()?;
        // max vertex index
        data.read_i32::<LittleEndian>()?;
        // collides
        data.read_i32::<LittleEndian>()?;
        // casts shadow
        data.read_i32::<LittleEndian>()?;
        // force opaque
        if asset.get_engine_version() >= EngineVersion::VER_UE4_25 {
            data.read_i32::<LittleEndian>()?;
        }
        //visible in ray tracing
        if asset.get_engine_version() >= EngineVersion::VER_UE4_26 {
            data.read_i32::<LittleEndian>()?;
        }
    }
    // max deviation
    data.read_f32::<LittleEndian>()?;
    match asset.get_engine_version() >= EngineVersion::VER_UE4_23 {
        true if !flags.data_stripped_for_server()
        // lod isn't cooked out
        && data.read_i32::<LittleEndian>()? == 0
        // data is inlined
        && data.read_i32::<LittleEndian>()? != 0 =>
        {
            super::StripDataFlags::read(&mut data)?;
        }
        false if !flags.data_stripped_for_server() && !flags.class_data_stripped(2) => (),
        _ => {
            dbg!("owo");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "mesh data is stripped",
            ));
        }
    }
    // stride
    data.read_i32::<LittleEndian>()?;
    // vertex count
    data.read_i32::<LittleEndian>()?;
    // size
    data.read_i32::<LittleEndian>()?;
    // finally the vertex positions!
    let len = data.read_i32::<LittleEndian>()?;
    let mut buf = vec![0f32; len as usize];
    for _ in 0..len {
        for _ in 0..3 {
            buf.push(data.read_f32::<LittleEndian>()?);
        }
    }
    Ok(buf)
}
