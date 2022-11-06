use unreal_asset::{
    error::Error,
    exports::{ExportBaseTrait, ExportNormalTrait},
    properties::{Property, PropertyDataTrait},
    unreal_types::{PackageIndex, ToFName},
    Asset,
};

pub struct Actor {
    export: PackageIndex,
}

impl Actor {
    pub fn new(asset: &Asset, export: PackageIndex) -> Result<Self, Error> {
        match asset.get_export(export) {
            Some(_) => Ok(Self { export }),
            None => Err(Error::invalid_package_index(format!(
                "failed to find actor at index {}",
                export.index - 1
            ))),
        }
    }

    pub fn name<'a>(&self, asset: &'a Asset) -> &'a str {
        // this is safe because invalid exports were already dealt with in the constructor
        &asset.exports[self.export.index as usize - 1]
            .get_base_export()
            .object_name
            .content
    }

    pub fn show(&self, asset: &mut Asset, ui: &mut egui::Ui) {
        let mut children = Vec::new();
        if let Some(Some(export)) = asset
            .get_export_mut(self.export)
            .map(|ex| ex.get_normal_export_mut())
        {
            ui.heading(&export.base_export.object_name.content);
            for prop in export.properties.iter_mut() {
                show_property(prop, ui);
            }
            // because i can't have a non-mutable reference as well
            children = export
                .get_base_export()
                .create_before_serialization_dependencies
                .clone()
        }
        for child in children {
            if let Some(Some(export)) = asset
                .get_export_mut(child)
                .map(|ex| ex.get_normal_export_mut())
            {
                for prop in export.properties.iter_mut() {
                    show_property(prop, ui);
                }
            }
        }
    }
}

fn show_property(property: &mut Property, ui: &mut egui::Ui) {
    if property.get_name().content.starts_with("UCS") {
        return;
    }
    match property {
        Property::BoolProperty(bool) => ui.checkbox(&mut bool.value, &bool.name.content),
        Property::UInt16Property(uint) => ui.label(&uint.name.content) | drag(ui, &mut uint.value),
        Property::UInt32Property(uint) => ui.label(&uint.name.content) | drag(ui, &mut uint.value),
        Property::UInt64Property(uint) => ui.label(&uint.name.content) | drag(ui, &mut uint.value),
        Property::FloatProperty(float) => {
            ui.label(&float.name.content) | drag(ui, &mut float.value.0)
        }
        Property::Int16Property(int) => ui.label(&int.name.content) | drag(ui, &mut int.value),
        Property::Int64Property(int) => ui.label(&int.name.content) | drag(ui, &mut int.value),
        Property::Int8Property(int) => ui.label(&int.name.content) | drag(ui, &mut int.value),
        Property::IntProperty(int) => ui.label(&int.name.content) | drag(ui, &mut int.value),
        Property::ByteProperty(byte) => {
            use unreal_asset::properties::int_property::ByteType;
            ui.label(&byte.name.content)
                | match byte.byte_type {
                    ByteType::Byte => drag(ui, &mut byte.value),
                    // byte.enum_type references the index of the fname in the name map so i need asset
                    // but the variable i'm editing is already a mutable reference to asset :/
                    ByteType::Long => return,
                }
        }
        Property::DoubleProperty(double) => {
            ui.label(&double.name.content) | drag(ui, &mut double.value.0)
        }
        Property::NameProperty(name) => {
            ui.label(&name.name.content)|
            // the trouble with this is that i could change references elsewhere
            ui.text_edit_singleline(&mut name.value.content)
        }
        Property::StrProperty(str) => ui.label(&str.name.content) | text_edit(ui, &mut str.value),
        Property::TextProperty(text) => {
            ui.label(&text.name.content) | text_edit(ui, &mut text.culture_invariant_string)
        }
        // we do nothing here to allow simplicity in the editor
        Property::ObjectProperty(_) => return,
        Property::AssetObjectProperty(obj) => {
            ui.label(&obj.name.content) | text_edit(ui, &mut obj.value)
        }
        Property::SoftObjectProperty(obj) => {
            ui.label(&obj.name.content) | ui.text_edit_singleline(&mut obj.value.content)
        }
        Property::IntPointProperty(int_point) => {
            ui.label(&int_point.name.content)
                | ui.horizontal(|ui| {
                    drag(ui, &mut int_point.x);
                    drag(ui, &mut int_point.y);
                })
                .response
        }
        Property::VectorProperty(vec) => {
            ui.label(&vec.name.content)
                | ui.horizontal(|ui| {
                    drag(ui, &mut vec.value.x.0);
                    drag(ui, &mut vec.value.y.0);
                    drag(ui, &mut vec.value.z.0);
                })
                .response
        }
        Property::ColorProperty(colour) => {
            ui.label(&colour.name.content) | {
                let mut val = [
                    colour.color.r,
                    colour.color.g,
                    colour.color.b,
                    colour.color.a,
                ];
                let response = ui.color_edit_button_srgba_unmultiplied(&mut val);
                colour.color.r = val[0];
                colour.color.g = val[1];
                colour.color.b = val[2];
                colour.color.a = val[3];
                response
            }
        }
        Property::TimeSpanProperty(time) => {
            ui.label(&time.name.content) | drag(ui, &mut time.ticks)
        }
        Property::DateTimeProperty(date) => {
            ui.label(&date.name.content) | drag(ui, &mut date.ticks)
        }
        Property::ArrayProperty(arr) => {
            if let Some(arr_type) = &arr.array_type {
                if arr_type.content == "ObjectProperty" {
                    return;
                }
            }
            ui.push_id(arr.clone(), |ui| {
                ui.collapsing(&arr.name.content, |ui| {
                    for prop in arr.value.iter_mut() {
                        show_property(prop, ui);
                    }
                })
            })
            .response
        }
        Property::MapProperty(map) => {
            ui.push_id(map.clone(), |ui| {
                ui.collapsing(&map.name.content, |ui| {
                    for set in map.value.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.label(&set.0.to_fname().content);
                            show_property(set.1, ui);
                        });
                    }
                })
            })
            .response
        }
        Property::PerPlatformBoolProperty(bools) => {
            ui.push_id(bools.clone(), |ui| {
                ui.collapsing(&bools.name.content, |ui| {
                    for bool in bools.value.iter_mut() {
                        ui.checkbox(bool, "");
                    }
                })
            })
            .response
        }
        Property::PerPlatformIntProperty(ints) => {
            ui.push_id(ints.clone(), |ui| {
                ui.collapsing(&ints.name.content, |ui| {
                    for int in ints.value.iter_mut() {
                        drag(ui, int);
                    }
                })
            })
            .response
        }
        Property::PerPlatformFloatProperty(floats) => {
            ui.push_id(floats.clone(), |ui| {
                ui.collapsing(&floats.name.content, |ui| {
                    for float in floats.value.iter_mut() {
                        drag(ui, &mut float.0);
                    }
                })
            })
            .response
        }
        Property::StructProperty(struc) => {
            ui.push_id(struc.clone(), |ui| {
                ui.collapsing(&struc.name.content, |ui| {
                    for prop in struc.value.iter_mut() {
                        show_property(prop, ui);
                    }
                })
            })
            .response
        }
        Property::Vector4Property(vec) => {
            ui.label(&vec.name.content)
                | ui.horizontal(|ui| {
                    drag(ui, &mut vec.value.w.0);
                    drag(ui, &mut vec.value.x.0);
                    drag(ui, &mut vec.value.y.0);
                    drag(ui, &mut vec.value.z.0);
                })
                .response
        }
        Property::Vector2DProperty(vec) => {
            ui.label(&vec.name.content)
                | ui.horizontal(|ui| {
                    drag(ui, &mut vec.x.0);
                    drag(ui, &mut vec.y.0);
                })
                .response
        }
        Property::BoxProperty(vec) => {
            ui.label(&vec.name.content)
                | ui.horizontal(|ui| {
                    drag(ui, &mut vec.v1.value.x.0);
                    drag(ui, &mut vec.v1.value.y.0);
                    drag(ui, &mut vec.v1.value.z.0);
                })
                .response
                | ui.horizontal(|ui| {
                    drag(ui, &mut vec.v2.value.x.0);
                    drag(ui, &mut vec.v2.value.y.0);
                    drag(ui, &mut vec.v2.value.z.0);
                })
                .response
        }
        Property::QuatProperty(quat) => {
            ui.label(&quat.name.content)
                | ui.horizontal(|ui| {
                    drag(ui, &mut quat.value.w.0);
                    drag(ui, &mut quat.value.x.0);
                    drag(ui, &mut quat.value.y.0);
                    drag(ui, &mut quat.value.z.0);
                })
                .response
        }
        Property::RotatorProperty(rot) => {
            ui.label(&rot.name.content)
                | ui.horizontal(|ui| {
                    drag(ui, &mut rot.value.x.0);
                    drag(ui, &mut rot.value.y.0);
                    drag(ui, &mut rot.value.z.0);
                })
                .response
        }
        Property::LinearColorProperty(colour) => {
            ui.label(&colour.name.content) | {
                let mut val = [
                    colour.color.r.0,
                    colour.color.g.0,
                    colour.color.b.0,
                    colour.color.a.0,
                ];
                let response = ui.color_edit_button_rgba_unmultiplied(&mut val);
                colour.color.r = val[0].into();
                colour.color.g = val[1].into();
                colour.color.b = val[2].into();
                colour.color.a = val[3].into();
                response
            }
        }
        Property::GuidProperty(guid) => {
            ui.label(&guid.name.content)
                | ui.horizontal(|ui| {
                    for val in guid.value.iter_mut() {
                        drag(ui, val);
                    }
                })
                .response
        }
        _ => return,
    }
    // this displays the property type
    .on_hover_text(&property.to_fname().content);
}

/// a wrapper for adding drag values with the range already specified to reduce code duplication
fn drag<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) -> egui::Response {
    ui.add(egui::widgets::DragValue::new(val).clamp_range(Num::MIN..=Num::MAX))
}

fn text_edit(ui: &mut egui::Ui, val: &mut Option<String>) -> egui::Response {
    let mut buf = val.clone().unwrap_or_default();
    let response = ui.text_edit_singleline(&mut buf);
    *val = Some(buf);
    response
}
