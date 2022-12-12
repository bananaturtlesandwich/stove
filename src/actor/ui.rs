use unreal_asset::{
    exports::ExportNormalTrait,
    properties::{Property, PropertyDataTrait},
    unreal_types::ToFName,
    Asset,
};

impl super::Actor {
    pub fn show(&self, asset: &mut Asset, ui: &mut egui::Ui) {
        ui.heading(&self.name);
        asset.exports[self.export]
            .get_normal_export_mut()
            .unwrap()
            .properties
            .iter_mut()
            .for_each(|prop| match prop {
                _ => show_simple_property(ui, prop),
            });
    }
}

// I don't want to have to install OrderedFloat to use functions
macro_rules! show_vector {
    ($ui:expr,$val:expr) => {
        drag($ui, &mut $val.value.x.0)
            | drag($ui, &mut $val.value.y.0)
            | drag($ui, &mut $val.value.z.0)
    };
}

macro_rules! show_vector4 {
    ($ui:expr,$val:expr) => {
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

fn drag<Num: egui::emath::Numeric>(ui: &mut egui::Ui, val: &mut Num) -> egui::Response {
    ui.add(
        egui::widgets::DragValue::new(val)
            .clamp_range(Num::MIN..=Num::MAX)
            .speed(10.0)
            .fixed_decimals(1),
    )
}

fn drag_angle(ui: &mut egui::Ui, val: &mut f32) -> egui::Response {
    ui.add(
        egui::widgets::DragValue::new(val)
            .suffix("°")
            .fixed_decimals(1),
    )
}

fn show_simple_property(ui: &mut egui::Ui, prop: &mut Property) {
    ui.horizontal(|ui| {
        ui.label(prop.get_name().content + ":");
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
            // Property::ByteProperty(_) => todo!(),
            Property::DoubleProperty(double) => drag(ui, &mut double.value.0),
            // Property::NameProperty(_) => todo!(),
            Property::StrProperty(str) => {
                ui.text_edit_singleline(str.value.get_or_insert(String::new()))
            }
            Property::TextProperty(txt) => {
                ui.text_edit_singleline(txt.culture_invariant_string.get_or_insert(String::new()))
            }
            // Property::ObjectProperty(_) => todo!(),
            // Property::AssetObjectProperty(_) => todo!(),
            // Property::SoftObjectProperty(_) => todo!(),
            Property::IntPointProperty(point) => drag(ui, &mut point.x) | drag(ui, &mut point.y),
            Property::VectorProperty(vec) => show_vector!(ui, vec),
            Property::Vector4Property(vec) => show_vector4!(ui, vec),
            Property::Vector2DProperty(vec) => drag(ui, &mut vec.x.0) | drag(ui, &mut vec.y.0),
            Property::BoxProperty(pak) => {
                ui.collapsing("v1:", |ui| show_vector!(ui, &mut pak.v1))
                    .header_response
                    | ui.collapsing("v2:", |ui| show_vector!(ui, &mut pak.v2))
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
            // Property::SetProperty(_) => todo!(),
            // Property::ArrayProperty(_) => todo!(),
            // Property::MapProperty(_) => todo!(),
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
            Property::SkeletalMeshAreaWeightedTriangleSampler(skel) => show_sampler!(ui, skel),
            // Property::SoftAssetPathProperty(_) => todo!(),
            // Property::SoftObjectPathProperty(_) => todo!(),
            // Property::SoftClassPathProperty(_) => todo!(),
            // Property::MulticastDelegateProperty(_) => todo!(),
            // Property::RichCurveKeyProperty(_) => todo!(),
            // Property::ViewTargetBlendParamsProperty(_) => todo!(),
            // Property::GameplayTagContainerProperty(_) => todo!(),
            // Property::SmartNameProperty(_) => todo!(),
            // Property::StructProperty(_) => todo!(),
            // Property::EnumProperty(_) => todo!(),
            // Property::UnknownProperty(_) => todo!(),
            _ => ui.link("unimplemented"),
        }
    })
    .response
    .on_hover_text(prop.to_fname().content);
}
