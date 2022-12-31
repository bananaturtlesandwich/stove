use unreal_asset::{
    cast,
    exports::ExportNormalTrait,
    properties::{
        struct_property::StructProperty, vector_property::VectorProperty, Property,
        PropertyDataTrait,
    },
    types::{vector::Vector, FName},
    Asset,
};

impl super::Actor {
    pub fn location(&self, map: &Asset) -> glam::Vec3 {
        map.exports[self.transform]
            .get_normal_export()
            .map(|norm| {
                norm.properties
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
                    .map(|pos| glam::vec3(pos.value.x.0, pos.value.z.0, pos.value.y.0) * 0.01)
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn add_location(&self, map: &mut Asset, offset: glam::Vec3) {
        let Some(norm) = map.exports[self.transform].get_normal_export_mut()
        else {
            return;
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name().content == "RelativeLocation")
        {
            Some(scale) => {
                if let Property::StructProperty(struc) = scale {
                    if let Property::VectorProperty(vec) = &mut struc.value[0] {
                        vec.value.x.0 += offset.x;
                        vec.value.y.0 += offset.z;
                        vec.value.z.0 += offset.y;
                    }
                }
            }
            None => norm
                .properties
                .push(Property::StructProperty(StructProperty {
                    name: FName::from_slice("RelativeLocation"),
                    struct_type: Some(FName::from_slice("Vector")),
                    struct_guid: None,
                    property_guid: None,
                    duplication_index: 0,
                    serialize_none: true,
                    value: vec![Property::VectorProperty(VectorProperty {
                        name: FName::from_slice("RelativeLocation"),
                        property_guid: None,
                        duplication_index: 0,
                        value: Vector::new(offset.x.into(), offset.z.into(), offset.y.into()),
                    })],
                })),
        }
    }

    pub fn rotation(&self, map: &Asset) -> glam::Vec3 {
        map.exports[self.transform]
            .get_normal_export()
            .map(|norm| {
                norm.properties
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
            })
            .unwrap_or_default()
    }

    pub fn scale(&self, map: &Asset) -> glam::Vec3 {
        map.exports[self.transform]
            .get_normal_export()
            .map(|norm| {
                norm.properties
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
                    .map(|rot| glam::vec3(rot.value.x.0, rot.value.z.0, rot.value.y.0))
                    .unwrap_or(glam::Vec3::ONE)
            })
            .unwrap_or(glam::Vec3::ONE)
    }

    pub fn model_matrix(&self, map: &Asset) -> glam::Mat4 {
        let rot = self.rotation(map);
        glam::Mat4::from_scale_rotation_translation(
            self.scale(map),
            glam::Quat::from_euler(
                glam::EulerRot::XYZ,
                rot.x.to_radians(),
                rot.y.to_radians(),
                rot.z.to_radians(),
            ),
            self.location(map),
        )
    }
}
