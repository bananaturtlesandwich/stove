use unreal_asset::{
    cast,
    error::Error,
    exports::ExportNormalTrait,
    properties::{Property, PropertyDataTrait},
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
    pub fn index(&self) -> PackageIndex {
        PackageIndex::new(self.export as i32 + 1)
    }
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
            .map(|pos| glam::vec3(-pos.value.x.0, pos.value.z.0, pos.value.y.0) * 0.01)
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

    pub fn get_scale(&self, asset: &Asset) -> glam::Vec3 {
        asset.exports[self.transform]
            .get_normal_export()
            .unwrap()
            .properties
            .iter()
            .rev()
            .find_map(|prop| {
                if let Property::StructProperty(struc) = prop {
                    if &struc.name.content == "RelativeScale3D" {
                        return cast!(Property, VectorProperty, &struc.value[0]);
                    }
                }
                None
            })
            .map(|rot| glam::vec3(-rot.value.x.0, rot.value.z.0, rot.value.y.0))
            .unwrap_or(glam::Vec3::ONE)
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

    pub fn show(&self, asset: &mut Asset, ui: &mut egui::Ui) {
        ui.heading(&self.name);
        ui.columns(4, |ui| {
            for prop in asset.exports[self.transform]
                .get_normal_export_mut()
                .unwrap()
                .properties
                .iter_mut()
            {
                if let Property::StructProperty(struc) = prop {
                    match struc.name.content.as_str() {
                        "RelativeLocation" | "RelativeScale3D" => {
                            if let Some(Property::VectorProperty(vec)) = struc.value.first_mut() {
                                ui[0].horizontal(|ui| ui.label(&vec.name.content[8..]));
                                drag(&mut ui[1], &mut vec.value.x.0);
                                drag(&mut ui[2], &mut vec.value.y.0);
                                drag(&mut ui[3], &mut vec.value.z.0);
                            }
                        }
                        "RelativeRotation" => {
                            if let Some(Property::RotatorProperty(rot)) = struc.value.first_mut() {
                                ui[0].horizontal(|ui| ui.label(&rot.name.content[8..]));
                                drag_angle(&mut ui[1], &mut rot.value.x.0);
                                drag_angle(&mut ui[2], &mut rot.value.y.0);
                                drag_angle(&mut ui[3], &mut rot.value.z.0);
                            }
                        }
                        _ => (),
                    }
                }
            }
        });
    }
}

fn drag<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) -> egui::Response {
    ui.add(
        egui::widgets::DragValue::new(val)
            .clamp_range(Num::MIN..=Num::MAX)
            .fixed_decimals(1),
    )
}

fn drag_angle(ui: &mut egui::Ui, val: &mut f32) {
    ui.add(
        egui::widgets::DragValue::new(val)
            .suffix("°")
            .fixed_decimals(1),
    );
}
