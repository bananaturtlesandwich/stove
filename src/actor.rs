use unreal_asset::{
    cast,
    error::Error,
    exports::{ExportBaseTrait, ExportNormalTrait},
    properties::{vector_property::VectorProperty, Property, PropertyDataTrait},
    unreal_types::PackageIndex,
    Asset,
};

pub struct Actor {
    export: usize,
    transform: usize,
}

impl Actor {
    pub fn get_translation<'a>(&self, asset: &'a mut Asset) -> Option<&'a mut VectorProperty> {
        asset.exports[self.transform]
            .get_normal_export_mut()
            .unwrap()
            .properties
            .iter_mut()
            .rev()
            .find_map(|prop| {
                if let Property::StructProperty(struc) = prop {
                    cast!(Property, VectorProperty, &mut struc.value[0])
                } else {
                    None
                }
            })
    }

    pub fn name<'a>(&self, asset: &'a Asset) -> &'a str {
        // this is safe because invalid exports were already dealt with in the constructor
        &asset.exports[self.export]
            .get_base_export()
            .object_name
            .content
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
        for prop in norm.properties.iter().rev() {
            match prop.get_name().content.as_str() {
                "BlueprintCreatedComponents" => {
                    if let Property::ArrayProperty(arr) = prop {
                        // for some reason some blueprintcreatedcomponents are padded with null references
                        for entry in arr.value.iter() {
                            if let Property::ObjectProperty(obj) = entry {
                                if asset.get_export(obj.value).is_some() {
                                    return Ok(Self {
                                        export,
                                        transform: obj.value.index as usize - 1,
                                    });
                                }
                            }
                        }
                    }
                }
                "RelativeLocation" | "RelativeRotation" | "RelativeScale3D" => {
                    return Ok(Self {
                        export,
                        transform: export,
                    })
                }
                "RootComponent" => {
                    if let Property::ObjectProperty(obj) = prop {
                        return Ok(Self {
                            export,
                            transform: obj.value.index as usize - 1,
                        });
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
        ui.heading(self.name(asset));
        if let Some(trans) = self.get_translation(asset) {
            ui.horizontal(|ui| {
                drag(ui, &mut trans.value.x.0);
                drag(ui, &mut trans.value.y.0);
                drag(ui, &mut trans.value.z.0);
            });
        }
    }
}

fn drag<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) -> egui::Response {
    ui.add(egui::widgets::DragValue::new(val).clamp_range(Num::MIN..=Num::MAX))
}
