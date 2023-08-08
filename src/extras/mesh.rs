use std::io;

use super::*;
use byteorder::{ReadBytesExt, LE};
use unreal_asset::{
    cast,
    engine_version::EngineVersion,
    exports::{ExportBaseTrait, ExportNormalTrait},
    object_version::ObjectVersion,
    properties::{Property, PropertyDataTrait},
    reader::archive_trait::ArchiveTrait,
};

#[test]
fn parse_mesh() -> Result<(), unreal_asset::error::Error> {
    use obj_exporter::*;
    let (verts, indices, ..) = get_mesh_info(unreal_asset::Asset::new(
        io::Cursor::new(include_bytes!("A02_Outside_Castle.uasset").as_slice()),
        Some(io::Cursor::new(
            include_bytes!("A02_Outside_Castle.uexp").as_slice(),
        )),
        EngineVersion::VER_UE4_25,
        None,
    )?)?;
    export_to_file(
        &ObjSet {
            material_library: None,
            objects: vec![Object {
                name: "A02_Outside_Castle".to_string(),
                vertices: verts
                    .into_iter()
                    .map(|glam::Vec3 { x, y, z }| Vertex {
                        x: x as f64,
                        y: y as f64,
                        z: z as f64,
                    })
                    .collect(),
                tex_vertices: vec![],
                normals: vec![],
                geometry: vec![Geometry {
                    material_name: None,
                    shapes: indices
                        .chunks(3)
                        .map(|face| Shape {
                            primitive: Primitive::Triangle(
                                (face[0] as usize, None, None),
                                (face[1] as usize, None, None),
                                (face[2] as usize, None, None),
                            ),
                            groups: vec![],
                            smoothing_groups: vec![],
                        })
                        .collect(),
                }],
            }],
        },
        "A02_Outside_Castle.obj",
    )?;
    Ok(())
}

// reference implementations:
// umodel: https://github.com/gildor2/UEViewer/blob/master/Unreal/UnrealMesh/UnMesh4.cpp#L2633
// cue4parse: https://github.com/FabianFG/CUE4Parse/blob/master/CUE4Parse/UE4/Assets/Exports/StaticMesh/UStaticMesh.cs#L13
// CAS UAssetAPI: https://github.com/LongerWarrior/UEAssetToolkitGenerator/blob/master/UAssetApi/ExportTypes/StaticMeshExport.cs#L6
/// parses the extra data of the static mesh export to get render data
pub fn get_mesh_info<C: io::Read + io::Seek>(
    asset: unreal_asset::Asset<C>,
) -> io::Result<(
    Vec<glam::Vec3>,
    Vec<u32>,
    Vec<Vec<(f32, f32)>>,
    Vec<String>,
    Vec<(u32, u32)>,
)> {
    // get the static mesh
    let Some(mesh) = asset.asset_data.exports.iter().find(|ex| {
        asset
            .get_import(ex.get_base_export().class_index)
            .map(|import| import.object_name == "StaticMesh")
            .unwrap_or(false)
    }) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "failed to find mesh export",
        ));
    };
    // get the normal export
    let Some(mesh) = mesh.get_normal_export() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "failed to cast mesh data",
        ));
    };
    let mats = mesh
        .properties
        .iter()
        .find(|prop| prop.get_name() == "StaticMaterials")
        .and_then(|prop| cast!(Property, ArrayProperty, prop))
        .map(|arr| {
            arr.value
                .iter()
                .filter_map(|prop| cast!(Property, StructProperty, prop))
                .filter_map(|struc| {
                    struc
                        .value
                        .iter()
                        .find(|prop| prop.get_name() == "MaterialInterface")
                })
                .filter_map(|inter| cast!(Property, ObjectProperty, inter))
                .filter_map(|obj| asset.get_import(obj.value))
                .filter_map(|imp| asset.get_import(imp.outer_index))
                .map(|imp| imp.object_name.get_owned_content())
                .collect()
        })
        .unwrap_or_default();
    let engine = asset.get_engine_version();
    let object = asset.get_object_version();
    let mut data = io::Cursor::new(mesh.extras.as_slice());
    // if this isn't read it breaks
    data.read_i32::<LE>()?;
    if !StripDataFlags::read(&mut data)?.editor_data_stripped()
        // data isn't cooked
        || data.read_u32::<LE>()? == 0
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "mesh is raw"));
    }
    // bodysetup reference
    data.read_u32::<LE>()?;
    // nav collision reference
    if object >= ObjectVersion::VER_UE4_STATIC_MESH_STORE_NAV_COLLISION {
        data.read_u32::<LE>()?;
    }
    // lighting guid
    for _ in 0..16 {
        data.read_u8()?;
    }
    // array of socket references
    for _ in 0..data.read_u32::<LE>()? {
        data.read_u32::<LE>()?;
    }
    // KeepMobileMinLODSettingOnDesktop is not here by default
    /*
    if engine >= EngineVersion::VER_UE4_27 {
        data.read_i32::<LE>()?;
    }
    */
    // array of lod resources
    // discard len because we'll just read the first LOD
    data.read_u32::<LE>()?;
    let flags = StripDataFlags::read(&mut data)?;
    if flags.data_stripped_for_server() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "mesh data is stripped for server",
        ));
    }
    let mut mat_data = Vec::with_capacity(data.read_u32::<LE>()? as usize);
    // array of sections
    for _ in 0..mat_data.capacity() {
        mat_data.push((
            // mat index
            data.read_u32::<LE>()?,
            // first index
            data.read_u32::<LE>()?,
        ));
        // tri count
        data.read_u32::<LE>()?;
        // min vertex index
        data.read_u32::<LE>()?;
        // max vertex index
        data.read_u32::<LE>()?;
        // collides
        data.read_u32::<LE>()?;
        // casts shadow
        data.read_u32::<LE>()?;
        // force opaque
        if engine >= EngineVersion::VER_UE4_25 {
            data.read_u32::<LE>()?;
        }
        //visible in ray tracing
        if engine >= EngineVersion::VER_UE4_26 {
            data.read_u32::<LE>()?;
        }
    }
    // max deviation
    data.read_f32::<LE>()?;
    match engine >= EngineVersion::VER_UE4_23 {
        // lod isn't cooked out
        true if data.read_u32::<LE>()? == 0
        // data is inlined
        && data.read_u32::<LE>()? == 1 =>
        {
            StripDataFlags::read(&mut data)?;
        }
        false if !flags.class_data_stripped(2) => (),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "lod data is cooked out",
            ));
        }
    }

    // position buffer
    // stride
    data.read_u32::<LE>()?;
    // vertex count
    data.read_u32::<LE>()?;
    // size
    data.read_u32::<LE>()?;
    let mut positions = Vec::with_capacity(data.read_u32::<LE>()? as usize);
    for _ in 0..positions.capacity() {
        let (x, y, z) = (
            data.read_f32::<LE>()?,
            data.read_f32::<LE>()?,
            data.read_f32::<LE>()?,
        );
        positions.push(glam::vec3(-x, z, y) * 0.01);
    }

    // vertex buffer
    if match object >= ObjectVersion::VER_UE4_STATIC_SKELETAL_MESH_SERIALIZATION_FIX {
        true => StripDataFlags::read(&mut data)?,
        false => StripDataFlags::default(),
    }
    .data_stripped_for_server()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "vertex buffer is stripped",
        ));
    }
    let num_tex_coords = data.read_u32::<LE>()?;
    // stride
    if engine < EngineVersion::VER_UE4_19 {
        data.read_u32::<LE>()?;
    }
    // num verts
    let num_verts = data.read_u32::<LE>()?;
    let precise_uvs = data.read_u32::<LE>()? == 1;
    let precise_tangents = engine >= EngineVersion::VER_UE4_12 && data.read_u32::<LE>()? == 1;
    fn read_tangents(
        data: &mut io::Cursor<&[u8]>,
        precise_tangents: bool,
    ) -> Result<(), io::Error> {
        match precise_tangents {
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
        Ok(())
    }
    fn half_to_single(half: u16) -> f32 {
        const SHIFTED: u32 = 0x7C00 << 13;
        const MAGIC: f32 = (113 << 23) as f32;
        let mut single = (half as u32 & 0x7FFF) << 13;
        let exp = SHIFTED & single as u32;
        single += (127 - 15) << 23;
        match exp == SHIFTED {
            true => single += (128 - 16) << 23,
            false => {
                single += 1 << 23;
                single = (single as f32 - MAGIC) as u32;
            }
        }
        single |= (half as u32 & 0x8000) << 16;
        single as f32
    }
    fn read_tex_coords(
        data: &mut io::Cursor<&[u8]>,
        num_tex_coords: u32,
        precise_uvs: bool,
    ) -> Result<Vec<(f32, f32)>, io::Error> {
        let mut uvs = Vec::with_capacity(num_tex_coords as usize);
        for _ in 0..num_tex_coords {
            uvs.push(match precise_uvs {
                true => (data.read_f32::<LE>()?, data.read_f32::<LE>()?),
                false => (
                    half_to_single(data.read_u16::<LE>()?),
                    half_to_single(data.read_u16::<LE>()?),
                ),
            })
        }
        Ok(uvs)
    }

    let mut uvs = Vec::with_capacity(num_verts as usize);
    match engine >= EngineVersion::VER_UE4_20 {
        true => {
            // item size
            data.read_u32::<LE>()?;
            // item count
            data.read_u32::<LE>()?;
            // packed normals
            for _ in 0..num_verts {
                read_tangents(&mut data, precise_tangents)?;
            }
            // item size
            data.read_u32::<LE>()?;
            // item count
            data.read_u32::<LE>()?;
            // mesh uv
            for _ in 0..num_verts {
                uvs.push(read_tex_coords(&mut data, num_tex_coords, precise_uvs)?);
            }
        }
        false => {
            // size
            data.read_u32::<LE>()?;
            // length
            data.read_u32::<LE>()?;
            for _ in 0..num_verts {
                read_tangents(&mut data, precise_tangents)?;
                uvs.push(read_tex_coords(&mut data, num_tex_coords, precise_uvs)?);
            }
        }
    }

    // color vertex buffer
    if match object >= ObjectVersion::VER_UE4_STATIC_SKELETAL_MESH_SERIALIZATION_FIX {
        true => StripDataFlags::read(&mut data)?,
        false => StripDataFlags::default(),
    }
    .data_stripped_for_server()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "colour data is stripped",
        ));
    }
    // stride
    data.read_u32::<LE>()?;
    // when num verts is 0 array isn't serialised
    if data.read_u32::<LE>()? > 0 {
        // size
        data.read_u32::<LE>()?;
        // vertex colours
        for _ in 0..data.read_u32::<LE>()? {
            data.read_i32::<LE>()?;
        }
    }

    let indices = match object >= ObjectVersion::VER_UE4_SUPPORT_32BIT_STATIC_MESH_INDICES {
        true => {
            let x32 = data.read_u32::<LE>()? == 1;
            // size
            data.read_u32::<LE>()?;
            match x32 {
                true => {
                    let mut indices = Vec::with_capacity(data.read_u32::<LE>()? as usize / 4);
                    for _ in 0..indices.capacity() {
                        indices.push(data.read_u32::<LE>()?);
                    }
                    indices
                }
                false => {
                    let mut indices = Vec::with_capacity(data.read_u32::<LE>()? as usize / 2);
                    for _ in 0..indices.capacity() {
                        indices.push(data.read_u16::<LE>()? as u32);
                    }
                    indices
                }
            }
        }
        false => {
            //size
            data.read_u32::<LE>()?;
            let mut indices = Vec::with_capacity(data.read_u32::<LE>()? as usize);
            for _ in 0..indices.capacity() {
                indices.push(data.read_u16::<LE>()? as u32);
            }
            indices
        }
    };
    Ok((positions, indices, uvs, mats, mat_data))
}
