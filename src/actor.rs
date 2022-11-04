use unreal_asset::{
    error::Error,
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    properties::Property,
    unreal_types::{PackageIndex, ToFName},
    Asset,
};

fn get_export_unchecked(asset: &Asset, index: PackageIndex) -> &Export {
    &asset.exports[index.index as usize - 1]
}
pub struct Actor {
    export: PackageIndex,
    transform: PackageIndex,
}

// may change to a list of class types because some objects will have all 3 values defaulted
const tags: [&str; 3] = ["RelativeLocation", "RelativeRotation", "RelativeScale3D"];

impl Actor {
    pub fn new(asset: &Asset, export: PackageIndex) -> Result<Self, Error> {
        match get_export_unchecked(asset, export)
            .get_base_export()
            .create_before_serialization_dependencies
            .iter()
            .find(
                |&&index| match get_export_unchecked(asset, index).get_normal_export() {
                    Some(norm) => norm
                        .properties
                        .iter()
                        .find(|prop| tags.contains(&prop.to_fname().content.as_str()))
                        .is_some(),
                    None => false,
                },
            ) {
            Some(&transform) => Ok(Self { export, transform }),
            None => Err(Error::invalid_package_index(
                "failed to find a scene component for an actor".to_string(),
            )),
        }
    }

    pub fn show(&self, asset: &mut Asset, ui: &mut egui::Ui) {
        let mut children = Vec::new();
        if let Some(export) = asset.get_export_mut(self.export) {
            show_export(export, ui);
            children = export
                .get_base_export()
                .create_before_serialization_dependencies
                .clone()
        }
        for child in children {
            if let Some(export) = asset.get_export_mut(child) {
                show_export(export, ui);
            }
        }
    }
}
fn show_export(export: &mut Export, ui: &mut egui::Ui) {
    ui.heading(&export.get_base_export().object_name.content);
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in norm.properties.iter_mut() {
            show_property(prop, ui);
        }
    }
}

fn show_property(property: &mut Property, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        match property {
            Property::BoolProperty(bool) => {
                ui.label(&bool.name.content);
                ui.checkbox(&mut bool.value, &bool.name.content);
            }
            Property::UInt16Property(uint) => {
                ui.label(&uint.name.content);
                slider(ui, &mut uint.value);
            }
            Property::UInt32Property(uint) => {
                ui.label(&uint.name.content);
                slider(ui, &mut uint.value);
            }
            Property::UInt64Property(uint) => {
                ui.label(&uint.name.content);
                slider(ui, &mut uint.value);
            }
            Property::FloatProperty(float) => {
                ui.label(&float.name.content);
                slider(ui, &mut float.value.into_inner());
            }
            Property::Int16Property(int) => {
                ui.label(&int.name.content);
                slider(ui, &mut int.value);
            }
            Property::Int64Property(int) => {
                ui.label(&int.name.content);
                slider(ui, &mut int.value);
            }
            Property::Int8Property(int) => {
                ui.label(&int.name.content);
                slider(ui, &mut int.value);
            }
            Property::IntProperty(int) => {
                ui.label(&int.name.content);
                slider(ui, &mut int.value);
            }
            Property::ByteProperty(byte) => {
                use unreal_asset::properties::int_property::ByteType;
                ui.label(&byte.name.content);
                match byte.byte_type {
                    ByteType::Byte =>{slider(ui, &mut byte.value);}
,
                    // byte.enum_type references the index of the fname in the name map so i need asset
                    // but the variable i'm editing is already a mutable reference to asset :/
                    ByteType::Long => {ui.label("ah rust makes this complicated");},
                };
            }
            Property::DoubleProperty(double) => {
                ui.label(&double.name.content);
                slider(ui, &mut double.value.into_inner());
            }
            Property::NameProperty(name) => {
                ui.label(&name.name.content);
                // the trouble with this is that i could change references elsewhere
                ui.label("fname shenanigans");
            }
            Property::StrProperty(str) => {
                ui.label(&str.name.content);
                text_edit(ui, &mut str.value);
            }
            Property::TextProperty(text) => {
                ui.label(&text.name.content);
                text_edit(ui, &mut text.culture_invariant_string);
            }
            // we do nothing here to abstract away references and allow simplicity in the editor
            Property::ObjectProperty(_) => {}
            Property::AssetObjectProperty(obj) => {
                ui.label(&obj.name.content);
                text_edit(ui, &mut obj.value);
            }
            Property::SoftObjectProperty(obj) => {
                ui.label(&obj.name.content);
                ui.label("fname shenanigans");
            }
            Property::IntPointProperty(int_point) => {
                ui.label(&int_point.name.content);
                slider(ui, &mut int_point.x);
                slider(ui, &mut int_point.y);
            }
            Property::VectorProperty(vec) => {
                ui.label(&vec.name.content);
                slider(ui, &mut vec.value.x.into_inner());
                slider(ui, &mut vec.value.y.into_inner());
                slider(ui, &mut vec.value.z.into_inner());
            }
            Property::ColorProperty(colour) => {
                ui.label(&colour.name.content);
                let mut val = [
                    colour.color.r,
                    colour.color.g,
                    colour.color.b,
                    colour.color.a,
                ];
                ui.color_edit_button_srgba_unmultiplied(&mut val);
                colour.color.r = val[0];
                colour.color.g = val[1];
                colour.color.b = val[2];
                colour.color.a = val[3];
            }
            Property::TimeSpanProperty(time) => {
                ui.label(&time.name.content);
                slider(ui, &mut time.ticks);
            }
            Property::DateTimeProperty(date) => {
                ui.label(&date.name.content);
                slider(ui, &mut date.ticks);
            }
            Property::SetProperty(set) => {
                // show_property(&mut Property::ArrayProperty(set.value), ui);
            }
            Property::ArrayProperty(arr) => {
                ui.collapsing(&arr.name.content, |ui| {
                    for prop in arr.value.iter_mut() {
                        show_property(prop, ui);
                    }
                });
            }
            Property::MapProperty(map) => {
                ui.collapsing(&map.name.content, |ui| {
                    for set in map.value.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.label(&set.0.to_fname().content);
                            show_property(set.1, ui);
                        });
                    }
                });
            }
            Property::PerPlatformBoolProperty(bools) => {
                ui.collapsing(&bools.name.content, |ui| {
                    for bool in bools.value.iter_mut() {
                        ui.checkbox(bool, "");
                    }
                });
            }
            Property::PerPlatformIntProperty(ints) => {
                ui.collapsing(&ints.name.content, |ui| {
                    for int in ints.value.iter_mut() {
                        slider(ui, int);
                    }
                });
            }
            Property::PerPlatformFloatProperty(floats) => {
                ui.collapsing(&floats.name.content, |ui| {
                    for float in floats.value.iter_mut() {
                        slider(ui, &mut float.into_inner());
                    }
                });
            }
            Property::StructProperty(struc) => {
                ui.collapsing(&struc.name.content, |ui| {
                    for prop in struc.value.iter_mut() {
                        show_property(prop, ui);
                    }
                });
            }
            // everything else is yet to be implemented because i'm lazy
            _ => {}
        }
    });
}

/// a wrapper for adding sliders with the range already specified to reduce code duplication
fn slider<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) {
    ui.add(egui::widgets::Slider::new(val, Num::MIN..=Num::MAX));
}

fn text_edit(ui: &mut egui::Ui, val: &mut Option<String>) {
    let mut buf = val.clone().unwrap_or_default();
    ui.text_edit_singleline(&mut buf);
    *val = Some(buf);
}
