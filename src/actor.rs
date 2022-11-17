use unreal_asset::{
    cast,
    error::Error,
    exports::ExportNormalTrait,
    properties::{
        struct_property::StructProperty, vector_property::VectorProperty, Property,
        PropertyDataTrait,
    },
    reader::asset_trait::AssetTrait,
    unreal_types::PackageIndex,
    Asset,
};

pub struct Actor {
    export: usize,
    transform: usize,
    pub name: String,
    pub class: String,
}

impl Actor {
    pub fn get_translation(&self, asset: &Asset) -> glam::Vec3 {
        asset.exports[self.transform]
            .get_normal_export()
            .unwrap()
            .properties
            .iter()
            .rev()
            .find_map(|prop| {
                if let Property::StructProperty(struc) = prop {
                    if &struc.name.content == "RelativeLocation" {
                        return cast!(Property, VectorProperty, &struc.value[0]);
                    }
                }
                None
            })
            .map(|trans| glam::vec3(trans.value.x.0, trans.value.y.0, trans.value.z.0) * 0.01)
            .unwrap_or_default()
    }

    pub fn set_translation(&self, asset: &mut Asset, pos: [f32; 3]) {
        match asset.exports[self.transform]
            .get_normal_export_mut()
            .unwrap()
            .properties
            .iter_mut()
            .rev()
            .find_map(|prop| {
                if let Property::StructProperty(struc) = prop {
                    if &struc.name.content == "RelativeLocation" {
                        return cast!(Property, VectorProperty, &mut struc.value[0]);
                    }
                }
                None
            }) {
            Some(trans) => {
                trans.value.x.0 = pos[0] * 100.0;
                trans.value.y.0 = pos[1] * 100.0;
                trans.value.z.0 = pos[2] * 100.0;
            }
            // create if does not exist
            None => {
                let name = asset.add_fname("RelativeTransform");
                let mut trans =
                    StructProperty::new(asset, name.clone(), false, 0, 0, asset.engine_version)
                        .unwrap();
                trans.value.push(Property::VectorProperty(
                    VectorProperty::new(asset, name, false, 0).unwrap(),
                ));
                asset.exports[self.transform]
                    .get_normal_export_mut()
                    .unwrap()
                    .properties
                    .push(Property::StructProperty(trans));
            }
        }
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
            "couldn't find transform component for actor at index {}",
            package.index
        )))
    }

    pub fn show(&self, asset: &mut Asset, ui: &mut egui::Ui) {
        ui.heading(&self.name);
        for prop in asset.exports[self.transform]
            .get_normal_export_mut()
            .unwrap()
            .properties
            .iter_mut()
        {
            if let Property::StructProperty(struc) = prop {
                if let Some(Property::VectorProperty(vec)) = struc.value.first_mut() {
                    ui.horizontal(|ui| {
                        ui.label(&vec.name.content);
                        drag(ui, &mut vec.value.x.0);
                        drag(ui, &mut vec.value.y.0);
                        drag(ui, &mut vec.value.z.0);
                    });
                }
            }
        }
    }
}

fn drag<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) -> egui::Response {
    ui.add(egui::widgets::DragValue::new(val).clamp_range(Num::MIN..=Num::MAX))
}
