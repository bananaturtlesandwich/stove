use unreal_asset::{
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    properties::{array_property::ArrayProperty, Property, PropertyDataTrait},
    unreal_types::{FName, ToFName},
    Asset,
};

impl super::Actor {
    pub fn show(&self, asset: &mut Asset, ui: &mut egui::Ui) {
        ui.heading(&self.name);
        fn show_export(ui: &mut egui::Ui, export: &mut Export) {
            if let Some(norm) = export.get_normal_export_mut() {
                for prop in norm.properties.iter_mut() {
                    show_property(ui, prop);
                }
            }
        }
        show_export(ui, &mut asset.exports[self.export]);
        for i in asset.exports[self.export]
            .get_base_export()
            .create_before_serialization_dependencies
            .clone()
            .iter()
        {
            if let Some(ex) = asset.get_export_mut(*i) {
                let name = &ex.get_base_export().object_name.content.clone();
                ui.push_id(ex.get_base_export().serial_offset, |ui| {
                    ui.collapsing(name, |ui| {
                        show_export(ui, ex);
                    });
                });
            }
        }
    }
}

fn show_array_property(ui: &mut egui::Ui, arr: &mut ArrayProperty) -> egui::Response {
    ui.push_id(&arr.name.content, |ui| {
        ui.collapsing("", |ui| {
            for (i, entry) in arr.value.iter_mut().enumerate() {
                ui.push_id(i, |ui| show_property(ui, entry));
            }
        })
        .header_response
    })
    .response
}

// I don't want to install OrderedFloat
macro_rules! show_vector {
    ($ui:ident,$val:expr) => {
        drag($ui, &mut $val.value.x.0)
            | drag($ui, &mut $val.value.y.0)
            | drag($ui, &mut $val.value.z.0)
    };
}

macro_rules! show_vector4 {
    ($ui:ident,$val:expr) => {
        drag($ui, &mut $val.value.w.0)
            | drag($ui, &mut $val.value.x.0)
            | drag($ui, &mut $val.value.y.0)
            | drag($ui, &mut $val.value.z.0)
    };
}

macro_rules! show_sampler {
    ($ui:ident,$val:expr) => {
        $ui.collapsing("alias", |$ui| {
            for i in $val.alias.iter_mut() {
                drag($ui, i);
            }
        })
        .header_response
            | $ui
                .collapsing("prob", |$ui| {
                    for i in $val.prob.iter_mut() {
                        drag($ui, &mut i.0);
                    }
                })
                .header_response
            | $ui.label("total weight:")
            | drag($ui, &mut $val.total_weight.0)
    };
}

macro_rules! show_path {
    ($ui:ident,$val:expr) => {
        $ui.text_edit_singleline(
            &mut $val
                .asset_path_name
                .get_or_insert(FName::from_slice(""))
                .content,
        ) | $ui.text_edit_singleline($val.sub_path.get_or_insert(String::new()))
            | $ui.text_edit_singleline($val.path.get_or_insert(String::new()))
    };
}

macro_rules! show_delegate {
    ($ui:ident,$val:expr) => {
        $ui.push_id(&$val.name.content, |ui| {
            ui.collapsing("", |ui| {
                for delegate in $val.value.iter_mut() {
                    ui.text_edit_singleline(&mut delegate.delegate.content);
                }
            })
        })
        .response
    };
}

fn drag<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) -> egui::Response {
    ui.add(
        egui::widgets::DragValue::new(val)
            .clamp_range(Num::MIN..=Num::MAX)
            .speed(10.0),
    )
}

fn drag_angle(ui: &mut egui::Ui, val: &mut f32) -> egui::Response {
    ui.add(egui::widgets::DragValue::new(val).suffix("°"))
}

fn show_property(ui: &mut egui::Ui, prop: &mut Property) {
    if let Property::ObjectProperty(_) = prop {
        return;
    }
    match prop.get_name().content.as_str() {
        "UCSModifiedProperties" | "UCSSerializationIndex" | "BlueprintCreatedComponents" => (),
        name => {
            ui.horizontal(|ui| {
                ui.label(name);
                match prop {
                    Property::BoolProperty(bool) => ui.checkbox(&mut bool.value, ""),
                    Property::UInt16Property(uint) => drag(ui, &mut uint.value),
                    Property::UInt32Property(uint) => drag(ui, &mut uint.value),
                    Property::UInt64Property(uint) => drag(ui, &mut uint.value),
                    Property::FloatProperty(float) => drag(ui, &mut float.value.0),
                    Property::Int16Property(int) => drag(ui, &mut int.value),
                    Property::Int64Property(int) => drag(ui, &mut int.value),
                    Property::Int8Property(int) => drag(ui, &mut int.value),
                    Property::IntProperty(int) => drag(ui, &mut int.value),
                    Property::ByteProperty(byte) => drag(ui, &mut byte.value),
                    Property::DoubleProperty(double) => drag(ui, &mut double.value.0),
                    Property::NameProperty(name) => {
                        ui.text_edit_singleline(&mut name.value.content)
                    }
                    Property::StrProperty(str) => {
                        ui.text_edit_singleline(str.value.get_or_insert(String::new()))
                    }
                    Property::TextProperty(txt) => ui.text_edit_singleline(
                        txt.culture_invariant_string.get_or_insert(String::new()),
                    ),
                    Property::ObjectProperty(obj) => ui.link(obj.value.index.to_string()),
                    Property::AssetObjectProperty(obj) => {
                        ui.text_edit_singleline(obj.value.get_or_insert(String::new()))
                    }
                    Property::SoftObjectProperty(obj) => {
                        ui.text_edit_singleline(&mut obj.value.content)
                    }
                    Property::IntPointProperty(point) => {
                        drag(ui, &mut point.x) | drag(ui, &mut point.y)
                    }
                    Property::VectorProperty(vec) => show_vector!(ui, vec),
                    Property::Vector4Property(vec) => show_vector4!(ui, vec),
                    Property::Vector2DProperty(vec) => {
                        drag(ui, &mut vec.x.0) | drag(ui, &mut vec.y.0)
                    }
                    Property::BoxProperty(pak) => {
                        ui.collapsing("v1", |ui| show_vector!(ui, &mut pak.v1))
                            .header_response
                            | ui.collapsing("v2", |ui| show_vector!(ui, &mut pak.v2))
                                .header_response
                    }
                    Property::QuatProperty(quat) => show_vector4!(ui, quat),
                    Property::RotatorProperty(rot) => {
                        drag_angle(ui, &mut rot.value.x)
                            | drag_angle(ui, &mut rot.value.y)
                            | drag_angle(ui, &mut rot.value.z)
                    }
                    Property::LinearColorProperty(col) => {
                        let mut buf = [col.color.r.0, col.color.g.0, col.color.b.0, col.color.a.0];
                        let response = ui.color_edit_button_rgba_unmultiplied(&mut buf);
                        col.color.r.0 = buf[0];
                        col.color.g.0 = buf[1];
                        col.color.b.0 = buf[2];
                        col.color.a.0 = buf[3];
                        response
                    }
                    Property::ColorProperty(col) => {
                        let mut buf = [col.color.r, col.color.g, col.color.b, col.color.a];
                        let response = ui.color_edit_button_srgba_unmultiplied(&mut buf);
                        col.color.r = buf[0];
                        col.color.g = buf[1];
                        col.color.b = buf[2];
                        col.color.a = buf[3];
                        response
                    }
                    Property::TimeSpanProperty(time) => drag(ui, &mut time.ticks),
                    Property::DateTimeProperty(date) => drag(ui, &mut date.ticks),
                    Property::GuidProperty(guid) => {
                        let mut response = drag(ui, &mut guid.value[0]);
                        for i in 1..16 {
                            response |= drag(ui, &mut guid.value[i])
                        }
                        response
                    }
                    Property::SetProperty(set) => show_array_property(ui, &mut set.value),
                    Property::ArrayProperty(arr) => show_array_property(ui, arr),
                    Property::MapProperty(map) => {
                        ui.push_id(&map.name.content, |ui| {
                            ui.collapsing("", |ui| {
                                for (_, value) in map.value.iter_mut() {
                                    show_property(ui, value);
                                }
                            })
                        })
                        .response
                    }
                    Property::PerPlatformBoolProperty(bools) => {
                        ui.collapsing("", |ui| {
                            for bool in bools.value.iter_mut() {
                                ui.checkbox(bool, "");
                            }
                        })
                        .header_response
                    }
                    Property::PerPlatformIntProperty(ints) => {
                        ui.collapsing("", |ui| {
                            for int in ints.value.iter_mut() {
                                drag(ui, int);
                            }
                        })
                        .header_response
                    }
                    Property::PerPlatformFloatProperty(floats) => {
                        ui.collapsing("", |ui| {
                            for float in floats.value.iter_mut() {
                                drag(ui, &mut float.0);
                            }
                        })
                        .header_response
                    }
                    // Property::MaterialAttributesInputProperty(_) => todo!(),
                    // Property::ExpressionInputProperty(_) => todo!(),
                    // Property::ColorMaterialInputProperty(_) => todo!(),
                    // Property::ScalarMaterialInputProperty(_) => todo!(),
                    // Property::ShadingModelMaterialInputProperty(_) => todo!(),
                    // Property::VectorMaterialInputProperty(_) => todo!(),
                    // Property::Vector2MaterialInputProperty(_) => todo!(),
                    Property::WeightedRandomSamplerProperty(rand) => show_sampler!(ui, rand),
                    Property::SkeletalMeshSamplingLODBuiltDataProperty(lod) => {
                        show_sampler!(ui, lod.sampler_property)
                    }
                    Property::SkeletalMeshAreaWeightedTriangleSampler(skel) => {
                        show_sampler!(ui, skel)
                    }
                    Property::SoftAssetPathProperty(path) => show_path!(ui, path),
                    Property::SoftObjectPathProperty(path) => show_path!(ui, path),
                    Property::SoftClassPathProperty(path) => show_path!(ui, path),
                    Property::DelegateProperty(del) => {
                        ui.text_edit_singleline(&mut del.value.delegate.content)
                    }
                    Property::MulticastDelegateProperty(del) => show_delegate!(ui, del),
                    Property::MulticastSparseDelegateProperty(del) => show_delegate!(ui, del),
                    Property::MulticastInlineDelegateProperty(del) => show_delegate!(ui, del),
                    // Property::RichCurveKeyProperty(_) => todo!(),
                    // Property::ViewTargetBlendParamsProperty(_) => todo!(),
                    // Property::GameplayTagContainerProperty(_) => todo!(),
                    Property::SmartNameProperty(name) => {
                        ui.text_edit_singleline(&mut name.display_name.content)
                    }
                    Property::StructProperty(str) => {
                        ui.push_id(&str.name.content, |ui| {
                            ui.collapsing("", |ui| {
                                for val in str.value.iter_mut() {
                                    show_property(ui, val)
                                }
                            })
                        })
                        .response
                    }
                    Property::EnumProperty(enm) => ui.text_edit_singleline(&mut enm.value.content),
                    // Property::UnknownProperty(unknown) => todo!(),
                    _ => ui.link("unimplemented"),
                }
            })
            .response
            .on_hover_text(prop.to_fname().content);
        }
    };
}
