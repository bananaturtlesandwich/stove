use super::*;
use unreal_asset::properties::{
    array_property::ArrayProperty, int_property::BytePropertyValue,
    object_property::SoftObjectPath, soft_path_property::SoftObjectPathPropertyValue,
};
use unreal_asset::types::fname::ToSerializedName;

impl Actor {
    pub fn show(
        &self,
        asset: &mut crate::Asset,
        ui: &mut egui::Ui,
        transform: &mut bevy::prelude::Transform,
    ) {
        ui.heading(format!("{} ({})", self.name, self.export));
        fn export(
            ui: &mut egui::Ui,
            export: &mut crate::Export,
            transform: &mut bevy::prelude::Transform,
        ) {
            if let Some(norm) = export.get_normal_export_mut() {
                for prop in norm.properties.iter_mut() {
                    property(ui, prop, transform);
                }
            }
        }
        export(ui, &mut asset.asset_data.exports[self.export], transform);
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
                    .push_id(id, |ui| {
                        ui.collapsing(egui::RichText::new(name).strong(), |ui| {
                            export(ui, ex, transform)
                        })
                    })
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
    mut func: impl FnMut(&mut egui::Ui, &mut T),
    init: impl Fn() -> T,
) {
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
    });
}

fn array_property(
    ui: &mut egui::Ui,
    arr: &mut ArrayProperty,
    transform: &mut bevy::prelude::Transform,
) {
    ui.collapsing("", |ui| {
        for (i, entry) in arr.value.iter_mut().enumerate() {
            ui.push_id(i, |ui| property(ui, entry, transform));
        }
    });
}

// I don't want to install OrderedFloat
macro_rules! vector {
    ($ui:ident, $($val:expr),+) => {{
        $(
            drag($ui, &mut $val);
        )+
    }};
}

macro_rules! sampler {
    ($ui:ident, $val:expr) => {{
        $ui.collapsing("alias", |ui| {
            for i in $val.alias.iter_mut() {
                drag(ui, i);
            }
        });
        $ui.collapsing("prob", |ui| {
            for i in $val.prob.iter_mut() {
                drag(ui, &mut i.0);
            }
        });
        $ui.label("total weight:");
        drag($ui, &mut $val.total_weight.0);
    }};
}

macro_rules! delegate {
    ($ui:ident, $val:expr) => {{
        $ui.collapsing("", |ui| {
            for (i, delegate) in $val.value.iter_mut().enumerate() {
                ui.push_id(i, |ui| fname(ui, &mut delegate.delegate));
            }
        });
    }};
}

fn drag<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) {
    ui.add(
        egui::widgets::DragValue::new(val)
            .clamp_range(Num::MIN..=Num::MAX)
            .speed(1.0),
    );
}

fn text(ui: &mut egui::Ui, val: &mut String) {
    egui::TextEdit::singleline(val).clip_text(false).show(ui);
}

fn fname(ui: &mut egui::Ui, name: &mut FName) {
    match name {
        FName::Backed {
            index, name_map, ..
        } => {
            // inline text() to get the response
            let res = egui::TextEdit::singleline(name_map.get_mut().get_name_reference_mut(*index))
                .clip_text(false)
                .show(ui)
                .response;
            if res.gained_focus() {
                let content = name_map.get_ref().get_owned_name(*index);
                let i = name_map.get_mut().add_name_reference(content, true);
                let f = name_map.get_ref().create_fname(i, 0);
                *name = f;
            }
        }
        FName::Dummy { value, .. } => text(ui, value),
    }
}

fn soft_path(ui: &mut egui::Ui, path: &mut SoftObjectPath) {
    ui.label("sub path:");
    option(ui, &mut path.sub_path_string, text, String::new);
    ui.label("asset name:");
    fname(ui, &mut path.asset_path.asset_name);
    ui.label("package name:");
    option(ui, &mut path.asset_path.package_name, fname, FName::default);
}

fn soft_property(ui: &mut egui::Ui, path: &mut SoftObjectPathPropertyValue) {
    match path {
        SoftObjectPathPropertyValue::Old(path) => {
            option(ui, path, text, String::new);
        }
        SoftObjectPathPropertyValue::New(path) => soft_path(ui, path),
    }
}

fn property(ui: &mut egui::Ui, prop: &mut Property, transform: &mut bevy::prelude::Transform) {
    if let Property::ObjectProperty(_) = prop {
        return;
    }
    if let Property::ArrayProperty(ArrayProperty { value, .. }) = prop {
        if !value
            .first()
            .is_some_and(|val| !matches!(val, Property::ObjectProperty(_)))
        {
            return;
        }
    }
    match prop.get_name().get_owned_content().as_str() {
        "UCSModifiedProperties" | "UCSSerializationIndex" | "BlueprintCreatedComponents" => (),
        name => {
            ui.push_id(name, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::widget_text::RichText::new(name).strong());
                    match prop {
                        Property::BoolProperty(bool) => {
                            ui.checkbox(&mut bool.value, "");
                        }
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
                                ui.label("enum type:");
                                option(ui, &mut byte.enum_type, fname, FName::default);
                                fname(ui, name);
                            }
                        },
                        Property::DoubleProperty(double) => drag(ui, &mut double.value.0),
                        Property::NameProperty(name) => fname(ui, &mut name.value),
                        Property::StrProperty(str) => option(ui, &mut str.value, text, String::new),
                        Property::TextProperty(txt) => {
                            option(ui, &mut txt.value, text, String::new);
                            ui.label("invariant string:");
                            option(ui, &mut txt.culture_invariant_string, text, String::new);
                        }
                        Property::AssetObjectProperty(obj) => {
                            option(ui, &mut obj.value, text, String::new)
                        }
                        Property::SoftObjectProperty(obj) => soft_path(ui, &mut obj.value),
                        Property::IntPointProperty(point) => {
                            vector!(ui, point.value.x, point.value.y)
                        }
                        Property::VectorProperty(vec) => {
                            let mut drag = |num| {
                                ui.add(
                                    egui::widgets::DragValue::new(num)
                                        .clamp_range(f64::MIN..=f64::MAX)
                                        .speed(1.0),
                                )
                            };
                            if (drag(&mut vec.value.x.0)
                                | drag(&mut vec.value.y.0)
                                | drag(&mut vec.value.z.0))
                            .changed()
                            {
                                vec.name.get_content(|name| match name {
                                    LOCATION => {
                                        transform.translation = bevy::math::dvec3(
                                            vec.value.x.0,
                                            vec.value.z.0,
                                            vec.value.y.0,
                                        )
                                        .as_vec3()
                                            * 0.01
                                    }
                                    SCALE => {
                                        transform.scale = bevy::math::dvec3(
                                            vec.value.x.0,
                                            vec.value.z.0,
                                            vec.value.y.0,
                                        )
                                        .as_vec3()
                                    }
                                    _ => (),
                                })
                            }
                        }
                        Property::Vector4Property(vec) => vector!(
                            ui,
                            vec.value.x.0,
                            vec.value.y.0,
                            vec.value.z.0,
                            vec.value.w.0
                        ),
                        Property::Vector2DProperty(vec) => {
                            vector!(ui, vec.value.x.0, vec.value.y.0)
                        }
                        Property::BoxProperty(pak) => {
                            ui.collapsing("v1", |ui| {
                                vector!(ui, pak.v1.value.x.0, pak.v1.value.y.0, pak.v1.value.z.0)
                            });
                            ui.collapsing("v2", |ui| {
                                vector!(ui, pak.v2.value.x.0, pak.v2.value.y.0, pak.v2.value.z.0)
                            });
                        }
                        Property::QuatProperty(quat) => vector!(
                            ui,
                            quat.value.x.0,
                            quat.value.y.0,
                            quat.value.z.0,
                            quat.value.w.0
                        ),

                        Property::RotatorProperty(rot) => {
                            let mut drag =
                                |num| ui.add(egui::widgets::DragValue::new(num).suffix("°"));
                            if (drag(&mut rot.value.x.0)
                                | drag(&mut rot.value.y.0)
                                | drag(&mut rot.value.z.0))
                            .changed()
                                && rot.name == ROTATION
                            {
                                transform.rotation = bevy::math::DQuat::from_euler(
                                    bevy::math::EulerRot::XYZ,
                                    rot.value.x.0.to_radians(),
                                    -rot.value.y.0.to_radians(),
                                    rot.value.z.0.to_radians(),
                                )
                                .as_quat()
                            }
                        }
                        Property::LinearColorProperty(col) => {
                            let mut buf =
                                [col.color.r.0, col.color.g.0, col.color.b.0, col.color.a.0];
                            if ui.color_edit_button_rgba_unmultiplied(&mut buf).changed() {
                                col.color.r.0 = buf[0];
                                col.color.g.0 = buf[1];
                                col.color.b.0 = buf[2];
                                col.color.a.0 = buf[3];
                            }
                        }
                        Property::ColorProperty(col) => {
                            let mut buf = [col.color.r, col.color.g, col.color.b, col.color.a];
                            if ui.color_edit_button_srgba_unmultiplied(&mut buf).changed() {
                                col.color.r = buf[0];
                                col.color.g = buf[1];
                                col.color.b = buf[2];
                                col.color.a = buf[3];
                            }
                        }
                        Property::TimeSpanProperty(time) => drag(ui, &mut time.ticks),
                        Property::DateTimeProperty(date) => drag(ui, &mut date.ticks),
                        Property::GuidProperty(guid) => {
                            for val in guid.value.0.iter_mut() {
                                drag(ui, val)
                            }
                        }
                        Property::SetProperty(set) => array_property(ui, &mut set.value, transform),
                        Property::ArrayProperty(arr) => array_property(ui, arr, transform),
                        Property::MapProperty(map) => {
                            ui.collapsing("", |ui| {
                                for (i, value) in map.value.values_mut().enumerate() {
                                    ui.push_id(i, |ui| property(ui, value, transform));
                                }
                            });
                        }
                        Property::PerPlatformBoolProperty(bools) => {
                            ui.collapsing("", |ui| {
                                for bool in bools.value.iter_mut() {
                                    ui.checkbox(bool, "");
                                }
                            });
                        }
                        Property::PerPlatformIntProperty(ints) => {
                            ui.collapsing("", |ui| {
                                for int in ints.value.iter_mut() {
                                    drag(ui, int);
                                }
                            });
                        }
                        Property::PerPlatformFloatProperty(floats) => {
                            ui.collapsing("", |ui| {
                                for float in floats.value.iter_mut() {
                                    drag(ui, &mut float.0);
                                }
                            });
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
                        Property::SoftAssetPathProperty(soft_path) => {
                            soft_property(ui, &mut soft_path.value)
                        }
                        Property::SoftObjectPathProperty(soft_path) => {
                            soft_property(ui, &mut soft_path.value)
                        }
                        Property::SoftClassPathProperty(soft_path) => {
                            soft_property(ui, &mut soft_path.value)
                        }
                        Property::DelegateProperty(del) => fname(ui, &mut del.value.delegate),
                        Property::MulticastDelegateProperty(del) => delegate!(ui, del),
                        Property::MulticastSparseDelegateProperty(del) => delegate!(ui, del),
                        Property::MulticastInlineDelegateProperty(del) => delegate!(ui, del),
                        // Property::RichCurveKeyProperty(_) => todo!(),
                        // Property::ViewTargetBlendParamsProperty(_) => todo!(),
                        // Property::GameplayTagContainerProperty(_) => todo!(),
                        Property::SmartNameProperty(name) => fname(ui, &mut name.display_name),
                        Property::StructProperty(str) => {
                            ui.collapsing("", |ui| {
                                for (i, val) in str.value.iter_mut().enumerate() {
                                    ui.push_id(i, |ui| property(ui, val, transform));
                                }
                            });
                        }
                        Property::EnumProperty(enm) => {
                            option(ui, &mut enm.value, fname, FName::default)
                        }
                        // Property::UnknownProperty(unknown) => todo!(),
                        _ => (),
                    };
                })
            })
            .response
            .on_hover_text(prop.to_serialized_name());
        }
    };
}
