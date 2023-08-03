use unreal_asset::{
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    properties::{
        array_property::ArrayProperty, int_property::BytePropertyValue,
        soft_path_property::SoftObjectPathPropertyValue, Property, PropertyDataTrait,
    },
    types::fname::{FName, ToSerializedName},
    Asset,
};

impl super::Actor {
    pub fn show(&self, asset: &mut Asset<std::fs::File>, ui: &mut egui::Ui) {
        ui.heading(&self.name);
        fn export(ui: &mut egui::Ui, export: &mut Export) {
            if let Some(norm) = export.get_normal_export_mut() {
                for prop in norm.properties.iter_mut() {
                    property(ui, prop);
                }
            }
        }
        export(ui, &mut asset.asset_data.exports[self.export]);
        for i in asset.asset_data.exports[self.export]
            .get_base_export()
            .create_before_serialization_dependencies
            .clone()
            .iter()
        {
            if let Some(ex) = asset.get_export_mut(*i) {
                let (name, id, index) = {
                    let ex = ex.get_base_export();
                    (
                        ex.object_name.get_owned_content(),
                        ex.serial_offset,
                        -ex.class_index.index - 1,
                    )
                };
                let response = ui
                    .push_id(id, |ui| ui.collapsing(name, |ui| export(ui, ex)))
                    .response;
                if let Some(import) = asset.imports.get(index as usize) {
                    import
                        .object_name
                        .get_content(|name| response.on_hover_text(name));
                }
            }
        }
    }
}

fn option<T>(
    ui: &mut egui::Ui,
    val: &mut Option<T>,
    mut func: impl FnMut(&mut egui::Ui, &mut T) -> egui::Response,
    init: impl Fn() -> T,
) -> egui::Response {
    ui.horizontal(|ui| {
        if let Some(inner) = val.as_mut() {
            func(ui, inner);
        }
        match val.is_some() {
            true => {
                if ui.button("x").clicked() {
                    *val = None;
                }
            }
            false => {
                if ui.button("+").clicked() {
                    *val = Some(init());
                }
            }
        }
    })
    .response
}

fn array_property(ui: &mut egui::Ui, arr: &mut ArrayProperty) -> egui::Response {
    ui.push_id(arr.name.get_owned_content(), |ui| {
        ui.collapsing("", |ui| {
            for (i, entry) in arr.value.iter_mut().enumerate() {
                ui.push_id(i, |ui| property(ui, entry));
            }
        })
        .header_response
    })
    .response
}

// I don't want to install OrderedFloat
macro_rules! vector {
    ($ui:ident, $val:expr) => {
        drag($ui, &mut $val.value.x.0)
            | drag($ui, &mut $val.value.y.0)
            | drag($ui, &mut $val.value.z.0)
    };
}

macro_rules! vector4 {
    ($ui:ident, $val:expr) => {
        drag($ui, &mut $val.value.w.0)
            | drag($ui, &mut $val.value.x.0)
            | drag($ui, &mut $val.value.y.0)
            | drag($ui, &mut $val.value.z.0)
    };
}

macro_rules! sampler {
    ($ui:ident, $val:expr) => {
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

macro_rules! path {
    ($ui:ident, $val:expr) => {
        match &mut $val.value {
            SoftObjectPathPropertyValue::Old(path) => option($ui, path, text_edit, String::new),
            SoftObjectPathPropertyValue::New(path) => {
                option($ui, &mut path.sub_path_string, text_edit, String::new)
                    | option(
                        $ui,
                        &mut path.asset_path.package_name,
                        fname_edit,
                        FName::default,
                    )
                    | fname_edit($ui, &mut path.asset_path.asset_name)
            }
        }
    };
}

macro_rules! delegate {
    ($ui:ident, $val:expr) => {
        $ui.push_id($val.name.get_owned_content(), |ui| {
            ui.collapsing("", |ui| {
                for delegate in $val.value.iter_mut() {
                    fname_edit(ui, &mut delegate.delegate);
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
            .speed(1.0),
    )
}

fn drag_angle(ui: &mut egui::Ui, val: &mut f64) -> egui::Response {
    ui.add(egui::widgets::DragValue::new(val).suffix("°"))
}

fn text_edit(ui: &mut egui::Ui, val: &mut String) -> egui::Response {
    egui::TextEdit::singleline(val)
        .clip_text(false)
        .show(ui)
        .response
}

fn fname_edit(ui: &mut egui::Ui, name: &mut FName) -> egui::Response {
    if let FName::Backed {
        index,
        number,
        name_map,
        ..
    } = name
    {
        if *number >= 0 {
            let string = name_map.get_ref().get_owned_name(*index);
            let f = name_map.get_mut().add_fname_with_number(&string, -1);
            *name = f;
        }
    }
    match name {
        FName::Backed {
            index, name_map, ..
        } => text_edit(ui, name_map.get_mut().get_name_reference_mut(*index)),
        FName::Dummy { value, .. } => text_edit(ui, value),
    }
}

fn property(ui: &mut egui::Ui, prop: &mut Property) {
    if let Property::ObjectProperty(_) = prop {
        return;
    }
    match prop.get_name().get_owned_content().as_str() {
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
                    Property::ByteProperty(byte) => match &mut byte.value {
                        BytePropertyValue::Byte(byte) => drag(ui, byte),
                        BytePropertyValue::FName(name) => {
                            option(ui, &mut byte.enum_type, fname_edit, FName::default)
                                | fname_edit(ui, name)
                        }
                    },
                    Property::DoubleProperty(double) => drag(ui, &mut double.value.0),
                    Property::NameProperty(name) => fname_edit(ui, &mut name.value),
                    Property::StrProperty(str) => {
                        option(ui, &mut str.value, text_edit, String::new)
                    }
                    Property::TextProperty(txt) => option(
                        ui,
                        &mut txt.culture_invariant_string,
                        text_edit,
                        String::new,
                    ),
                    Property::ObjectProperty(obj) => ui.link(obj.value.index.to_string()),
                    Property::AssetObjectProperty(obj) => {
                        option(ui, &mut obj.value, text_edit, String::new)
                    }
                    Property::SoftObjectProperty(obj) => {
                        option(ui, &mut obj.value.sub_path_string, text_edit, String::new)
                            | fname_edit(ui, &mut obj.value.asset_path.asset_name)
                            | option(
                                ui,
                                &mut obj.value.asset_path.package_name,
                                fname_edit,
                                FName::default,
                            )
                    }
                    Property::IntPointProperty(point) => {
                        drag(ui, &mut point.value.x) | drag(ui, &mut point.value.y)
                    }
                    Property::VectorProperty(vec) => vector!(ui, vec),
                    Property::Vector4Property(vec) => vector4!(ui, vec),
                    Property::Vector2DProperty(vec) => {
                        drag(ui, &mut vec.value.x.0) | drag(ui, &mut vec.value.y.0)
                    }
                    Property::BoxProperty(pak) => {
                        ui.collapsing("v1", |ui| vector!(ui, &mut pak.v1))
                            .header_response
                            | ui.collapsing("v2", |ui| vector!(ui, &mut pak.v2))
                                .header_response
                    }
                    Property::QuatProperty(quat) => vector4!(ui, quat),
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
                    Property::SetProperty(set) => array_property(ui, &mut set.value),
                    Property::ArrayProperty(arr) => array_property(ui, arr),
                    Property::MapProperty(map) => {
                        ui.push_id(map.name.get_owned_content(), |ui| {
                            ui.collapsing("", |ui| {
                                for value in map.value.values_mut() {
                                    property(ui, value);
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
                    Property::WeightedRandomSamplerProperty(rand) => sampler!(ui, rand),
                    Property::SkeletalMeshSamplingLODBuiltDataProperty(lod) => {
                        sampler!(ui, lod.sampler_property)
                    }
                    Property::SkeletalMeshAreaWeightedTriangleSampler(skel) => {
                        sampler!(ui, skel)
                    }
                    Property::SoftAssetPathProperty(path) => path!(ui, path),
                    Property::SoftObjectPathProperty(path) => path!(ui, path),
                    Property::SoftClassPathProperty(path) => path!(ui, path),
                    Property::DelegateProperty(del) => fname_edit(ui, &mut del.value.delegate),
                    Property::MulticastDelegateProperty(del) => delegate!(ui, del),
                    Property::MulticastSparseDelegateProperty(del) => delegate!(ui, del),
                    Property::MulticastInlineDelegateProperty(del) => delegate!(ui, del),
                    // Property::RichCurveKeyProperty(_) => todo!(),
                    // Property::ViewTargetBlendParamsProperty(_) => todo!(),
                    // Property::GameplayTagContainerProperty(_) => todo!(),
                    Property::SmartNameProperty(name) => fname_edit(ui, &mut name.display_name),
                    Property::StructProperty(str) => {
                        ui.push_id(str.name.get_owned_content(), |ui| {
                            ui.collapsing("", |ui| {
                                for val in str.value.iter_mut() {
                                    property(ui, val)
                                }
                            })
                        })
                        .response
                    }
                    Property::EnumProperty(enm) => {
                        option(ui, &mut enm.value, fname_edit, FName::default)
                    }
                    // Property::UnknownProperty(unknown) => todo!(),
                    _ => ui.link("unimplemented"),
                }
            })
            .response
            .on_hover_text(prop.to_serialized_name());
        }
    };
}
