use std::{fs, io, path::Path};

use unreal_asset::{
    engine_version::EngineVersion,
    error::Error,
    exports::{ExportBaseTrait, ExportNormalTrait},
    properties::{Property, PropertyDataTrait},
    types::ToFName,
    Asset,
};

/// creates an asset from the specified path and version
pub fn open(file: impl AsRef<Path>, version: EngineVersion) -> Result<Asset, Error> {
    let bulk = file.as_ref().with_extension("uexp");
    let mut asset = Asset::new(
        fs::read(&file)?,
        // the none option is given as some uassets may not use the event driven loader
        bulk.exists().then_some(fs::read(bulk)?),
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
        asset.use_separate_bulk_data_files.then_some(&mut bulk),
    )?;
    fs::write(path.as_ref().with_extension("umap"), main.into_inner())?;
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
                resolve_prop_name(prop, asset, false);
            }
        }
    }
}

fn resolve_prop_name(prop: &Property, asset: &mut Asset, is_array: bool) {
    asset.add_fname(&prop.to_fname().content);
    // the name of properties in arrays is their index
    if !is_array {
        asset.add_fname(&prop.get_name().content);
    }
    match prop {
        Property::ByteProperty(prop) => {
            if let Some(en) = &prop.enum_type {
                asset.add_fname(&en.content);
            }
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
                resolve_prop_name(prop, asset, false);
            }
        }
        Property::ArrayProperty(prop) => {
            for prop in prop.value.iter() {
                resolve_prop_name(prop, asset, true);
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
        Property::SetProperty(prop) => {
            for prop in prop.value.value.iter() {
                resolve_prop_name(prop, asset, true);
            }
            for prop in prop.removed_items.value.iter() {
                resolve_prop_name(prop, asset, true);
            }
        }
        Property::MapProperty(prop) => {
            for (_, key, value) in prop.value.iter() {
                resolve_prop_name(key, asset, false);
                resolve_prop_name(value, asset, false);
            }
        }
        Property::MaterialAttributesInputProperty(prop) => {
            asset.add_fname(&prop.material_expression.input_name.content);
            asset.add_fname(&prop.material_expression.expression_name.content);
        }
        Property::ExpressionInputProperty(prop) => {
            asset.add_fname(&prop.material_expression.input_name.content);
            asset.add_fname(&prop.material_expression.expression_name.content);
        }
        Property::ColorMaterialInputProperty(prop) => {
            asset.add_fname(&prop.material_expression.input_name.content);
            asset.add_fname(&prop.material_expression.expression_name.content);
        }
        Property::ScalarMaterialInputProperty(prop) => {
            asset.add_fname(&prop.material_expression.input_name.content);
            asset.add_fname(&prop.material_expression.expression_name.content);
        }
        Property::ShadingModelMaterialInputProperty(prop) => {
            asset.add_fname(&prop.material_expression.input_name.content);
            asset.add_fname(&prop.material_expression.expression_name.content);
        }
        Property::VectorMaterialInputProperty(prop) => {
            asset.add_fname(&prop.material_expression.input_name.content);
            asset.add_fname(&prop.material_expression.expression_name.content);
        }
        Property::Vector2MaterialInputProperty(prop) => {
            asset.add_fname(&prop.material_expression.input_name.content);
            asset.add_fname(&prop.material_expression.expression_name.content);
        }
        Property::StringAssetReferenceProperty(prop) => {
            if let Some(path) = &prop.asset_path_name {
                asset.add_fname(&path.content);
            }
        }
        Property::GameplayTagContainerProperty(prop) => {
            for name in prop.value.iter() {
                asset.add_fname(&name.content);
            }
        }
        Property::UniqueNetIdProperty(net) => {
            if let Some(id) = &net.value {
                asset.add_fname(&id.ty.content);
            }
        }
        Property::NiagaraVariableProperty(prop) => {
            for prop in prop.struct_property.value.iter() {
                resolve_prop_name(prop, asset, false);
            }
            asset.add_fname(&prop.variable_name.content);
        }
        Property::NiagaraVariableWithOffsetProperty(prop) => {
            for prop in prop.niagara_variable.struct_property.value.iter() {
                resolve_prop_name(prop, asset, false);
            }
            asset.add_fname(&prop.niagara_variable.variable_name.content);
        }
        _ => (),
    }
}
