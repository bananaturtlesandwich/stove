use unreal_asset::{exports::ExportNormalTrait, properties::Property, Asset};

impl super::Actor {
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
