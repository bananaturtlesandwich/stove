use unreal_asset::{
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    properties::{Property, PropertyDataTrait},
    reader::asset_trait::AssetTrait,
    unreal_types::PackageIndex,
    Asset,
};

pub struct Actor {
    export: PackageIndex,
}

impl Actor {
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
        ui.label(&property.get_name().content);
        match property {
            Property::BoolProperty(bool) => {
                ui.checkbox(&mut bool.value, &bool.name.content);
            }
            Property::UInt32Property(uint) => {
                ui.add(egui::widgets::Slider::new(
                    &mut uint.value,
                    u32::MIN..=u32::MAX,
                ));
            }
            Property::UInt64Property(uint) => {
                ui.add(egui::widgets::Slider::new(
                    &mut uint.value,
                    u64::MIN..=u64::MAX,
                ));
            }
            Property::FloatProperty(float) => {
                ui.add(egui::widgets::Slider::new(
                    &mut float.value.into_inner(),
                    f32::MIN..=f32::MAX,
                ));
            }
            Property::Int16Property(int) => {
                ui.add(egui::widgets::Slider::new(
                    &mut int.value,
                    i16::MIN..=i16::MAX,
                ));
            }
            Property::Int64Property(int) => {
                ui.add(egui::widgets::Slider::new(
                    &mut int.value,
                    i64::MIN..=i64::MAX,
                ));
            }
            Property::Int8Property(int) => {
                ui.add(egui::widgets::Slider::new(
                    &mut int.value,
                    i8::MIN..=i8::MAX,
                ));
            }
            Property::IntProperty(int) => {
                ui.add(egui::widgets::Slider::new(
                    &mut int.value,
                    i32::MIN..=i32::MAX,
                ));
            }
            Property::ByteProperty(byte) => {
                use unreal_asset::properties::int_property::ByteType;
                match byte.byte_type {
                    ByteType::Byte => ui.add(egui::widgets::Slider::new(
                        &mut byte.value,
                        i64::MIN..=i64::MAX,
                    )),
                    // byte.enum_type references the index of the fname in the name map so i need asset
                    // but the variable i'm editing is already a mutable reference to asset :/
                    ByteType::Long => ui.label("ah rust makes this complicated"),
                };
            }
            Property::DoubleProperty(double) => {
                ui.add(egui::widgets::Slider::new(
                    &mut double.value.into_inner(),
                    f64::MIN..=f64::MAX,
                ));
            }
            Property::NameProperty(_) => {
                // the trouble with this is that i could change references elsewhere
                ui.label("fname shenanigans");
            }
            Property::StrProperty(str) => {
                let mut buf = str.value.clone().unwrap_or_default();
                ui.text_edit_singleline(&mut buf);
                str.value = Some(buf);
            }
            Property::TextProperty(text) => {
                let mut buf = text.culture_invariant_string.clone().unwrap_or_default();
                ui.text_edit_singleline(&mut buf);
                text.culture_invariant_string = Some(buf);
            }
            // we do nothing here to abstract away references and allow simplicity in the editor
            Property::ObjectProperty(_) => {}
            Property::AssetObjectProperty(obj) => {
                let mut buf = obj.value.clone().unwrap_or_default();
                ui.text_edit_singleline(&mut buf);
                obj.value = Some(buf);
            }
            Property::SoftObjectProperty(_) => {
                ui.label("fname shenanigans");
            }
            Property::IntPointProperty(int_point) => {
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Slider::new(
                        &mut int_point.x,
                        i32::MIN..=i32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut int_point.y,
                        i32::MIN..=i32::MAX,
                    ));
                });
            }
            Property::VectorProperty(vec) => {
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.value.x.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.value.y.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.value.z.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                });
            }
            Property::Vector4Property(vec) => {
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.value.w.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.value.x.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.value.y.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.value.z.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                });
            }
            Property::Vector2DProperty(vec) => {
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.x.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut vec.y.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                });
            }
            Property::BoxProperty(prop) => {
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Slider::new(
                        &mut prop.v1.value.x.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut prop.v1.value.y.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut prop.v1.value.z.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Slider::new(
                        &mut prop.v2.value.x.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut prop.v2.value.y.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                    ui.add(egui::widgets::Slider::new(
                        &mut prop.v2.value.z.into_inner(),
                        f32::MIN..=f32::MAX,
                    ));
                });
            }
            Property::QuatProperty(_) => todo!(),
            Property::RotatorProperty(_) => todo!(),
            Property::LinearColorProperty(_) => todo!(),
            Property::ColorProperty(_) => todo!(),
            Property::TimeSpanProperty(_) => todo!(),
            Property::DateTimeProperty(_) => todo!(),
            Property::GuidProperty(_) => todo!(),
            Property::SetProperty(_) => todo!(),
            Property::ArrayProperty(_) => todo!(),
            Property::MapProperty(_) => todo!(),
            Property::PerPlatformBoolProperty(_) => todo!(),
            Property::PerPlatformIntProperty(_) => todo!(),
            Property::PerPlatformFloatProperty(_) => todo!(),
            Property::MaterialAttributesInputProperty(_) => todo!(),
            Property::ExpressionInputProperty(_) => todo!(),
            Property::ColorMaterialInputProperty(_) => todo!(),
            Property::ScalarMaterialInputProperty(_) => todo!(),
            Property::ShadingModelMaterialInputProperty(_) => todo!(),
            Property::VectorMaterialInputProperty(_) => todo!(),
            Property::Vector2MaterialInputProperty(_) => todo!(),
            Property::WeightedRandomSamplerProperty(_) => todo!(),
            Property::SkeletalMeshSamplingLODBuiltDataProperty(_) => todo!(),
            Property::SkeletalMeshAreaWeightedTriangleSampler(_) => todo!(),
            Property::SoftAssetPathProperty(_) => todo!(),
            Property::SoftObjectPathProperty(_) => todo!(),
            Property::SoftClassPathProperty(_) => todo!(),
            Property::MulticastDelegateProperty(_) => todo!(),
            Property::RichCurveKeyProperty(_) => todo!(),
            Property::ViewTargetBlendParamsProperty(_) => todo!(),
            Property::GameplayTagContainerProperty(_) => todo!(),
            Property::SmartNameProperty(_) => todo!(),
            Property::StructProperty(_) => todo!(),
            Property::EnumProperty(_) => todo!(),
            Property::UnknownProperty(_) => todo!(),
        }
    });
}
