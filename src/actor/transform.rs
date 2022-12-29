use unreal_asset::{
    cast,
    exports::ExportNormalTrait,
    properties::{
        struct_property::StructProperty,
        vector_property::{RotatorProperty, VectorProperty},
        Property, PropertyDataTrait,
    },
    types::{vector::Vector, FName},
    Asset,
};

impl super::Actor {
    pub fn get_location(&self, map: &Asset) -> glam::Vec3 {
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

    pub fn add_location(&self, map: &mut Asset, location: glam::Vec3) {
        let Some(norm) = map.exports[self.transform].get_normal_export_mut()
        else {
            return;
        };
        let location = location * 100.0;
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name().content == "RelativeLocation")
        {
            Some(loc) => {
                if let Property::StructProperty(struc) = loc {
                    if let Property::VectorProperty(vec) = &mut struc.value[0] {
                        vec.value.x.0 += location.x;
                        vec.value.y.0 += location.z;
                        vec.value.z.0 += location.y;
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
                        value: Vector::new(location.x.into(), location.z.into(), location.y.into()),
                    })],
                })),
        }
    }

    pub fn get_rotation(&self, map: &Asset) -> glam::Vec3 {
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
                    .map(|rot| glam::vec3(rot.value.z.0, rot.value.y.0, rot.value.x.0))
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn add_rotation(&self, map: &mut Asset, mut rotation: glam::Vec3) {
        let Some(norm) = map.exports[self.transform].get_normal_export_mut()
        else {
            return;
        };
        rotation.x = rotation.x.to_degrees();
        rotation.y = rotation.y.to_degrees();
        rotation.z = rotation.z.to_degrees();
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name().content == "RelativeRotation")
        {
            Some(scale) => {
                if let Property::StructProperty(struc) = scale {
                    if let Property::RotatorProperty(vec) = &mut struc.value[0] {
                        vec.value.x.0 += rotation.z;
                        vec.value.y.0 += rotation.x;
                        vec.value.z.0 += rotation.y;
                    }
                }
            }
            None => norm
                .properties
                .push(Property::StructProperty(StructProperty {
                    name: FName::from_slice("RelativeRotation"),
                    struct_type: Some(FName::from_slice("Rotator")),
                    struct_guid: None,
                    property_guid: None,
                    duplication_index: 0,
                    serialize_none: true,
                    value: vec![Property::RotatorProperty(RotatorProperty {
                        name: FName::from_slice("RelativeRotation"),
                        property_guid: None,
                        duplication_index: 0,
                        value: Vector::new(rotation.z.into(), rotation.x.into(), rotation.y.into()),
                    })],
                })),
        }
    }

    pub fn get_scale(&self, map: &Asset) -> glam::Vec3 {
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

    pub fn add_scale(&self, map: &mut Asset, scale: glam::Vec3) {
        let Some(norm) = map.exports[self.transform].get_normal_export_mut()
        else {
            return;
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name().content == "RelativeScale3D")
        {
            Some(sca) => {
                if let Property::StructProperty(struc) = sca {
                    if let Property::VectorProperty(vec) = &mut struc.value[0] {
                        vec.value.x.0 += scale.x;
                        vec.value.y.0 += scale.z;
                        vec.value.z.0 += scale.y;
                    }
                }
            }
            None => norm
                .properties
                .push(Property::StructProperty(StructProperty {
                    name: FName::from_slice("RelativeScale3D"),
                    struct_type: Some(FName::from_slice("Vector")),
                    struct_guid: None,
                    property_guid: None,
                    duplication_index: 0,
                    serialize_none: true,
                    value: vec![Property::VectorProperty(VectorProperty {
                        name: FName::from_slice("RelativeScale3D"),
                        property_guid: None,
                        duplication_index: 0,
                        value: Vector::new(
                            (1.0 + scale.x).into(),
                            (1.0 + scale.z).into(),
                            (1.0 + scale.y).into(),
                        ),
                    })],
                })),
        }
    }

    pub fn model_matrix(&self, map: &Asset) -> glam::Mat4 {
        let rot = self.get_rotation(map);
        glam::Mat4::from_scale_rotation_translation(
            self.get_scale(map),
            glam::Quat::from_euler(
                glam::EulerRot::XYZ,
                rot.x.to_radians(),
                rot.y.to_radians(),
                rot.z.to_radians(),
            ),
            self.get_location(map),
        )
    }
}
