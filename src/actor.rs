use unreal_asset::{
    cast,
    error::Error,
    exports::{ExportBaseTrait, ExportNormalTrait},
    properties::Property,
    unreal_types::{PackageIndex, ToFName},
    Asset,
};

pub struct Actor {
    export: usize,
}

impl Actor {
    pub fn new(asset: &Asset, export: PackageIndex) -> Result<Self, Error> {
        match asset.get_export(export) {
            Some(_) => Ok(Self {
                export: export.index as usize - 1,
            }),
            None => Err(Error::invalid_package_index(format!(
                "failed to find actor at index {}",
                export.index - 1
            ))),
        }
    }

    pub fn name<'a>(&self, asset: &'a Asset) -> &'a str {
        // this is safe because invalid exports were already dealt with in the constructor
        &asset.exports[self.export]
            .get_base_export()
            .object_name
            .content
    }

    pub fn show(&self, asset: &mut Asset, ui: &mut egui::Ui) {
        for (i, prop) in asset.exports[self.export]
            .get_normal_export()
            .unwrap()
            .properties
            // clone so that i can check type for those special cases where i need access to the asset first
            .clone()
            .iter()
            .enumerate()
        {
            self.show_property(prop, i, asset, ui);
        }
    }

    /// this was the best way to compromise imo
    fn show_property(
        &self,
        prop_ref: &Property,
        prop_index: usize,
        asset: &mut Asset,
        ui: &mut egui::Ui,
    ) {
        match prop_ref {
            Property::NameProperty(name) => {
                let mut buf = name.value.content.clone();
                let response = ui.text_edit_singleline(&mut buf);
                if response.changed() {
                    let fname = asset.add_fname(&buf);
                    cast!(
                        Property,
                        NameProperty,
                        &mut asset.exports[self.export]
                            .get_normal_export_mut()
                            .unwrap()
                            .properties[prop_index]
                    )
                    .unwrap()
                    .value = fname;
                }
                response
            }
            // leave the normal byte properties to the other function
            Property::ByteProperty(byte) if byte.enum_type.is_some() => {
                let mut buf = asset.get_name_reference(byte.enum_type.unwrap() as i32);
                let response = ui.text_edit_singleline(&mut buf);
                // since it's an index i think i have to add an fname on every change
                if response.changed() {
                    let fname = asset.add_fname(&buf);
                    cast!(
                        Property,
                        ByteProperty,
                        &mut asset.exports[self.export]
                            .get_normal_export_mut()
                            .unwrap()
                            .properties[prop_index]
                    )
                    .unwrap()
                    .value = fname.index as i64;
                }
                response
            }
            Property::EnumProperty(e) => {
                let mut buf = e.value.content.clone();
                let response = ui.text_edit_singleline(&mut buf);
                if response.changed() {
                    let fname = asset.add_fname(&buf);
                    cast!(
                        Property,
                        EnumProperty,
                        &mut asset.exports[self.export]
                            .get_normal_export_mut()
                            .unwrap()
                            .properties[prop_index]
                    )
                    .unwrap()
                    .value = fname;
                }
                response
            }
            _ => show_simple_properties(
                &mut asset.exports[self.export]
                    .get_normal_export_mut()
                    .unwrap()
                    .properties[prop_index],
                ui,
            ),
        }
        // this displays the property type
        .on_hover_text(&prop_ref.to_fname().content);
    }
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

fn show_simple_properties(property: &mut Property, ui: &mut egui::Ui) -> egui::Response {
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
        Property::DoubleProperty(double) => {
            ui.label(&double.name.content) | drag(ui, &mut double.value.0)
        }
        Property::StrProperty(str) => ui.label(&str.name.content) | text_edit(ui, &mut str.value),
        Property::TextProperty(text) => {
            ui.label(&text.name.content) | text_edit(ui, &mut text.culture_invariant_string)
        }
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
        Property::PerPlatformBoolProperty(bools) => {
            ui.collapsing(&bools.name.content, |ui| {
                for bool in bools.value.iter_mut() {
                    ui.checkbox(bool, "");
                }
            })
            .header_response
        }
        Property::PerPlatformIntProperty(ints) => {
            ui.collapsing(&ints.name.content, |ui| {
                for int in ints.value.iter_mut() {
                    drag(ui, int);
                }
            })
            .header_response
        }
        Property::PerPlatformFloatProperty(floats) => {
            ui.collapsing(&floats.name.content, |ui| {
                for float in floats.value.iter_mut() {
                    drag(ui, &mut float.0);
                }
            })
            .header_response
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
        _ => ui.hyperlink_to(
            "currently unimplemented",
            "https://github.com/bananaturtlesandwich/stove/issues/new",
        ),
    }
}
