use unreal_asset::{
    error::Error,
    exports::ExportNormalTrait,
    properties::{Property, PropertyDataTrait},
    reader::asset_trait::AssetTrait,
    unreal_types::PackageIndex,
    Asset,
};

mod duplicate;
mod transform;
mod transplant;
mod ui;

pub struct Actor {
    export: usize,
    transform: usize,
    pub name: String,
    pub class: String,
}

impl Actor {
    pub fn index(&self) -> PackageIndex {
        PackageIndex::new(self.export as i32 + 1)
    }
    pub fn new(asset: &Asset, package: PackageIndex) -> Result<Self, Error> {
        let export = package.index as usize - 1;
        let Some(ex) = asset.get_export(package) else{
            return Err(Error::invalid_package_index(format!(
                "failed to find actor at index {}",
                package.index
            )))
        };
        let Some(norm) = ex.get_normal_export() else {
            return Err(Error::no_data(format!("actor at index {} failed to parse", package.index)))
        };
        let name = norm.base_export.object_name.content.clone();
        let class = asset
            .get_import(norm.base_export.class_index)
            .map(|import| import.object_name.content.clone())
            .unwrap_or_default();
        // normally these are further back so reversed should be a bit faster
        for prop in norm.properties.iter().rev() {
            match prop.get_name().content.as_str() {
                // of course this wouldn't be able to be detected if all transforms were left default
                "RelativeLocation" | "RelativeRotation" | "RelativeScale3D" => {
                    return Ok(Self {
                        export,
                        transform: export,
                        name,
                        class,
                    })
                }
                "RootComponent" => {
                    if let Property::ObjectProperty(obj) = prop {
                        if obj.value.is_export() {
                            return Ok(Self {
                                export,
                                transform: obj.value.index as usize - 1,
                                name,
                                class,
                            });
                        }
                    }
                }
                _ => continue,
            }
        }
        Err(Error::no_data(format!(
            "couldn't find transform component for {}",
            &norm.base_export.object_name.content
        )))
    }
}
