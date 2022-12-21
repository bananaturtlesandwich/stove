use std::{fs, io, path::Path};

use unreal_asset::{
    engine_version::EngineVersion,
    error::Error,
    exports::{ExportBaseTrait, ExportNormalTrait},
    flags::EPackageFlags,
    properties::{Property, PropertyDataTrait},
    unreal_types::ToFName,
    Asset,
};

/// creates an asset from the specified path and version
pub fn open(file: impl AsRef<Path>, version: EngineVersion) -> Result<Asset, Error> {
    let bulk = file.as_ref().with_extension("uexp");
    let mut asset = Asset::new(
        fs::read(&file)?,
        match bulk.exists() {
            true => Some(fs::read(bulk)?),
            // the none option is given as some uassets may not use the event driven loader
            false => None,
        },
    );
    asset.set_engine_version(version);
    asset.parse_data()?;
    Ok(asset)
}

/// saves an asset's data to the specified path
pub fn save(asset: &mut Asset, path: impl AsRef<Path>) -> Result<(), Error> {
    resolve_names(asset);
    let mut main = io::Cursor::new(Vec::new());
    let mut bulk = main.clone();
    asset.write_data(
        &mut main,
        match asset.use_separate_bulk_data_files {
            true => Some(&mut bulk),
            false => None,
        },
    )?;
    fs::write(
        path.as_ref().with_extension(
            match EPackageFlags::from_bits_truncate(asset.package_flags)
                .intersects(EPackageFlags::PKG_CONTAINS_MAP)
            {
                true => "umap",
                false => "uasset",
            },
        ),
        main.into_inner(),
    )?;
    // if the asset has no bulk data then the bulk cursor will be empty
    if asset.use_separate_bulk_data_files {
        fs::write(path.as_ref().with_extension("uexp"), bulk.into_inner())?;
    }
    Ok(())
}

/// so i don't have to deal with borrow checker in ui code
fn resolve_names(asset: &mut Asset) {
    for import in asset.imports.clone().iter() {
        asset.add_fname(&import.class_package.content);
        asset.add_fname(&import.class_name.content);
        asset.add_fname(&import.object_name.content);
    }
    for export in asset.exports.clone().iter() {
        asset.add_fname(&export.get_base_export().object_name.content);
        // resolve the rest of the name references
        if let Some(norm) = export.get_normal_export() {
            for prop in norm.properties.iter() {
                resolve_prop_name(prop, asset);
            }
        }
    }
}

fn resolve_prop_name(prop: &Property, asset: &mut Asset) {
    asset.add_fname(&prop.to_fname().content);
    asset.add_fname(&prop.get_name().content);
    match prop {
        Property::ByteProperty(prop) => {
            if let unreal_asset::properties::int_property::BytePropertyValue::FName(name) =
                &prop.value
            {
                asset.add_fname(&name.content);
            }
        }
        Property::NameProperty(prop) => {
            asset.add_fname(&prop.value.content);
        }
        Property::TextProperty(prop) => {
            if let Some(id) = prop.table_id.as_ref() {
                asset.add_fname(&id.content);
            }
        }
        Property::SoftObjectProperty(prop) => {
            asset.add_fname(&prop.value.asset_path_name.content);
        }
        Property::SoftAssetPathProperty(prop) => {
            if let Some(path) = prop.asset_path_name.as_ref() {
                asset.add_fname(&path.content);
            }
        }
        Property::SoftObjectPathProperty(prop) => {
            if let Some(path) = prop.asset_path_name.as_ref() {
                asset.add_fname(&path.content);
            }
        }
        Property::SoftClassPathProperty(prop) => {
            if let Some(path) = prop.asset_path_name.as_ref() {
                asset.add_fname(&path.content);
            }
        }
        Property::DelegateProperty(del) => {
            asset.add_fname(&del.value.delegate.content);
        }
        Property::MulticastDelegateProperty(del) => {
            for delegate in del.value.iter() {
                asset.add_fname(&delegate.delegate.content);
            }
        }
        Property::MulticastSparseDelegateProperty(del) => {
            for delegate in del.value.iter() {
                asset.add_fname(&delegate.delegate.content);
            }
        }
        Property::MulticastInlineDelegateProperty(del) => {
            for delegate in del.value.iter() {
                asset.add_fname(&delegate.delegate.content);
            }
        }
        Property::SmartNameProperty(prop) => {
            asset.add_fname(&prop.display_name.content);
        }
        Property::StructProperty(prop) => {
            if let Some(typ) = prop.struct_type.as_ref() {
                asset.add_fname(&typ.content);
            }
            for prop in prop.value.iter() {
                resolve_prop_name(prop, asset);
            }
        }
        Property::ArrayProperty(prop) => {
            for prop in prop.value.iter() {
                resolve_prop_name(prop, asset);
            }
        }
        Property::EnumProperty(prop) => {
            asset.add_fname(&prop.value.content);
            if let Some(typ) = prop.enum_type.as_ref() {
                asset.add_fname(&typ.content);
            }
        }
        Property::UnknownProperty(prop) => {
            asset.add_fname(&prop.serialized_type.content);
        }
        _ => (),
    }
}
