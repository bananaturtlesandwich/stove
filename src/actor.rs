use unreal_asset::{
    cast,
    error::Error,
    exports::ExportNormalTrait,
    properties::{
        struct_property::StructProperty,
        vector_property::{RotatorProperty, VectorProperty},
        Property, PropertyDataTrait,
    },
    reader::asset_trait::AssetTrait,
    types::Vector,
    unreal_types::{FName, PackageIndex},
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
            .map(|pos| glam::vec3(pos.value.x.0, pos.value.y.0, pos.value.z.0) * 0.01)
            .unwrap_or_default()
    }

    pub fn get_rotation(&self, asset: &Asset) -> glam::Vec3 {
        asset.exports[self.transform]
            .get_normal_export()
            .unwrap()
            .properties
            .iter()
            .rev()
            .find_map(|prop| {
                if let Property::StructProperty(struc) = prop {
                    if &struc.name.content == "RelativeRotation" {
                        return cast!(Property, RotatorProperty, &struc.value[0]);
                    }
                }
                None
            })
            .map(|rot| glam::vec3(rot.value.x.0, rot.value.y.0, rot.value.z.0))
            .unwrap_or_default()
    }

    pub fn set_translation(&self, asset: &mut Asset, new: [f32; 3]) {
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
            Some(pos) => {
                pos.value.x.0 = new[0] * 100.0;
                pos.value.y.0 = new[1] * 100.0;
                pos.value.z.0 = new[2] * 100.0;
            }
            // create if does not exist
            None => {
                let name = asset.add_fname("RelativeTransform");
                asset.exports[self.transform]
                    .get_normal_export_mut()
                    .unwrap()
                    .properties
                    .push(Property::StructProperty(StructProperty {
                        name: name.clone(),
                        // from research this seems to be the field values
                        struct_type: Some(FName::from_slice("Vector")),
                        struct_guid: Some([0; 16]),
                        property_guid: None,
                        duplication_index: 0,
                        serialize_none: true,
                        value: vec![Property::VectorProperty(VectorProperty {
                            name,
                            property_guid: None,
                            duplication_index: 0,
                            value: Vector::new(0.0.into(), 0.0.into(), 0.0.into()),
                        })],
                    }));
            }
        }
    }

    pub fn set_rotation(&self, asset: &mut Asset, new: [f32; 3]) {
        match asset.exports[self.transform]
            .get_normal_export_mut()
            .unwrap()
            .properties
            .iter_mut()
            .rev()
            .find_map(|prop| {
                if let Property::StructProperty(struc) = prop {
                    if &struc.name.content == "RelativeRotation" {
                        return cast!(Property, RotatorProperty, &mut struc.value[0]);
                    }
                }
                None
            }) {
            Some(rot) => {
                rot.value.x.0 = new[0];
                rot.value.y.0 = new[1];
                rot.value.z.0 = new[2];
            }
            // create if does not exist
            None => {
                let name = asset.add_fname("RelativeRotation");
                asset.exports[self.transform]
                    .get_normal_export_mut()
                    .unwrap()
                    .properties
                    .push(Property::StructProperty(StructProperty {
                        name: name.clone(),
                        // from research this seems to be the field values
                        struct_type: Some(FName::from_slice("Rotator")),
                        struct_guid: Some([0; 16]),
                        property_guid: None,
                        duplication_index: 0,
                        serialize_none: true,
                        value: vec![Property::RotatorProperty(RotatorProperty {
                            name,
                            property_guid: None,
                            duplication_index: 0,
                            value: Vector::new(0.0.into(), 0.0.into(), 0.0.into()),
                        })],
                    }));
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
                ui.horizontal(|ui| match struc.name.content.as_str() {
                    "RelativeLocation" | "RelativeScale3D" => {
                        if let Some(Property::VectorProperty(vec)) = struc.value.first_mut() {
                            ui.label(&vec.name.content);
                            drag(ui, &mut vec.value.x.0);
                            drag(ui, &mut vec.value.y.0);
                            drag(ui, &mut vec.value.z.0);
                        }
                    }
                    "RelativeRotation" => {
                        if let Some(Property::RotatorProperty(rot)) = struc.value.first_mut() {
                            ui.label(&rot.name.content);
                            drag_angle(ui, &mut rot.value.x.0);
                            drag_angle(ui, &mut rot.value.y.0);
                            drag_angle(ui, &mut rot.value.z.0);
                        }
                    }
                    _ => (),
                });
            }
        }
    }
}

fn drag<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) -> egui::Response {
    ui.add(egui::widgets::DragValue::new(val).clamp_range(Num::MIN..=Num::MAX))
}

fn drag_angle(ui: &mut egui::Ui, val: &mut f32) {
    let mut buf = *val;
    ui.add(egui::widgets::DragValue::new(&mut buf).suffix("°"));
    *val = buf % 360.0;
}
