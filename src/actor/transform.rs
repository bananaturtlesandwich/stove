use unreal_asset::{
    cast,
    exports::ExportNormalTrait,
    properties::{
        struct_property::StructProperty,
        vector_property::{RotatorProperty, VectorProperty},
        Property, PropertyDataTrait,
    },
    types::vector::Vector,
};

pub const LOCATION: &str = "RelativeLocation";
pub const ROTATION: &str = "RelativeRotation";
pub const SCALE: &str = "RelativeScale3D";

impl super::Actor {
    pub fn location(&self, map: &crate::Asset) -> bevy::math::Vec3 {
        map.asset_data.exports[self.transform]
            .get_normal_export()
            .and_then(|norm| {
                norm.properties.iter().rev().find_map(|prop| {
                    if let Property::StructProperty(struc) = prop {
                        if struc.name == LOCATION {
                            return cast!(Property, VectorProperty, &struc.value[0]);
                        }
                    }
                    None
                })
            })
            .map(|pos| {
                bevy::math::dvec3(-pos.value.x.0, pos.value.z.0, pos.value.y.0).as_vec3() * 0.01
            })
            .unwrap_or_default()
    }

    pub fn coords(&self, map: &crate::Asset, proj: bevy::math::Mat4) -> bevy::math::Vec2 {
        use bevy::math::Vec4Swizzles;
        let coords = proj * self.location(map).extend(1.0);
        coords.xy() / coords.w.abs()
    }

    pub fn add_location(&self, map: &mut crate::Asset, offset: bevy::math::Vec3) {
        let mut names = map.get_name_map();
        let Some(norm) = map.asset_data.exports[self.transform].get_normal_export_mut() else {
            return;
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name() == LOCATION)
        {
            Some(scale) => {
                if let Property::StructProperty(struc) = scale {
                    if let Property::VectorProperty(vec) = &mut struc.value[0] {
                        vec.value.x.0 -= offset.x as f64;
                        vec.value.y.0 += offset.z as f64;
                        vec.value.z.0 += offset.y as f64;
                    }
                }
            }
            None => {
                let name = names.get_mut().add_fname(LOCATION);
                let struct_type = Some(names.get_mut().add_fname("Vector"));
                norm.properties
                    .push(Property::StructProperty(StructProperty {
                        name,
                        ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                            ancestry: Vec::new(),
                        },
                        struct_type,
                        struct_guid: Some([0; 16].into()),
                        property_guid: None,
                        duplication_index: 0,
                        serialize_none: true,
                        value: vec![Property::VectorProperty(VectorProperty {
                            name: names.get_mut().add_fname(LOCATION),
                            ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                                ancestry: Vec::new(),
                            },
                            property_guid: None,
                            duplication_index: 0,
                            value: Vector::new(
                                (-offset.x as f64).into(),
                                (offset.z as f64).into(),
                                (offset.y as f64).into(),
                            ),
                        })],
                    }));
            }
        }
    }

    pub fn get_raw_location(&self, map: &crate::Asset) -> bevy::math::DVec3 {
        map.asset_data.exports[self.transform]
            .get_normal_export()
            .and_then(|norm| {
                norm.properties.iter().rev().find_map(|prop| {
                    if let Property::StructProperty(struc) = prop {
                        if struc.name == LOCATION {
                            return cast!(Property, VectorProperty, &struc.value[0]);
                        }
                    }
                    None
                })
            })
            .map(|pos| bevy::math::dvec3(pos.value.x.0, pos.value.y.0, pos.value.z.0))
            .unwrap_or_default()
    }

    pub fn add_raw_location(&self, map: &mut crate::Asset, offset: bevy::math::DVec3) {
        let mut names = map.get_name_map();
        let Some(norm) = map.asset_data.exports[self.transform].get_normal_export_mut() else {
            return;
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name() == LOCATION)
        {
            Some(scale) => {
                if let Property::StructProperty(struc) = scale {
                    if let Property::VectorProperty(vec) = &mut struc.value[0] {
                        vec.value.x.0 += offset.x;
                        vec.value.y.0 += offset.y;
                        vec.value.z.0 += offset.z;
                    }
                }
            }
            None => norm
                .properties
                .push(Property::StructProperty(StructProperty {
                    name: names.clone_resource().get_mut().add_fname(LOCATION),
                    ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                        ancestry: Vec::new(),
                    },
                    struct_type: Some(names.clone_resource().get_mut().add_fname("Vector")),
                    struct_guid: Some([0; 16].into()),
                    property_guid: None,
                    duplication_index: 0,
                    serialize_none: true,
                    value: vec![Property::VectorProperty(VectorProperty {
                        name: names.get_mut().add_fname(LOCATION),
                        ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                            ancestry: Vec::new(),
                        },
                        property_guid: None,
                        duplication_index: 0,
                        value: Vector::new(offset.x.into(), offset.z.into(), offset.y.into()),
                    })],
                })),
        }
    }

    pub fn rotation(&self, map: &crate::Asset) -> bevy::math::Quat {
        map.asset_data.exports[self.transform]
            .get_normal_export()
            .map(|norm| {
                norm.properties
                    .iter()
                    .rev()
                    .find_map(|prop| {
                        if let Property::StructProperty(struc) = prop {
                            if struc.name == ROTATION {
                                return cast!(Property, RotatorProperty, &struc.value[0]);
                            }
                        }
                        None
                    })
                    .map(|rot| {
                        bevy::math::DQuat::from_euler(
                            bevy::math::EulerRot::XYZ,
                            rot.value.x.0.to_radians(),
                            rot.value.y.0.to_radians(),
                            rot.value.z.0.to_radians(),
                        )
                        .as_f32()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn combine_rotation(&self, map: &mut crate::Asset, offset: bevy::math::Quat) {
        let mut names = map.get_name_map();
        let Some(norm) = map.asset_data.exports[self.transform].get_normal_export_mut() else {
            return;
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name() == ROTATION)
        {
            Some(scale) => {
                if let Property::StructProperty(struc) = scale {
                    if let Property::RotatorProperty(vec) = &mut struc.value[0] {
                        (vec.value.x.0, vec.value.y.0, vec.value.z.0) = (offset.as_f64()
                            * bevy::math::DQuat::from_euler(
                                bevy::math::EulerRot::XYZ,
                                vec.value.x.0.to_radians(),
                                vec.value.y.0.to_radians(),
                                vec.value.z.0.to_radians(),
                            ))
                        .to_euler(bevy::math::EulerRot::XYZ);
                        (vec.value.x.0, vec.value.y.0, vec.value.z.0) = (
                            vec.value.x.0.to_degrees(),
                            vec.value.y.0.to_degrees(),
                            vec.value.z.0.to_degrees(),
                        );
                    }
                }
            }
            None => {
                let name = names.get_mut().add_fname(ROTATION);
                let struct_type = Some(names.get_mut().add_fname("Rotator"));
                norm.properties
                    .push(Property::StructProperty(StructProperty {
                        name,
                        ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                            ancestry: Vec::new(),
                        },
                        struct_type,
                        struct_guid: Some([0; 16].into()),
                        property_guid: None,
                        duplication_index: 0,
                        serialize_none: true,
                        value: vec![Property::RotatorProperty(RotatorProperty {
                            name: names.get_mut().add_fname(ROTATION),
                            ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                                ancestry: Vec::new(),
                            },
                            property_guid: None,
                            duplication_index: 0,
                            value: Vector::new(
                                (offset.x as f64).into(),
                                (offset.z as f64).into(),
                                (offset.y as f64).into(),
                            ),
                        })],
                    }));
            }
        }
    }

    pub fn scale(&self, map: &crate::Asset) -> bevy::math::Vec3 {
        map.asset_data.exports[self.transform]
            .get_normal_export()
            .and_then(|norm| {
                norm.properties.iter().rev().find_map(|prop| {
                    if let Property::StructProperty(struc) = prop {
                        if struc.name == SCALE {
                            return cast!(Property, VectorProperty, &struc.value[0]);
                        }
                    }
                    None
                })
            })
            .map(|rot| bevy::math::dvec3(rot.value.x.0, rot.value.z.0, rot.value.y.0).as_vec3())
            .unwrap_or(bevy::math::Vec3::ONE)
    }

    pub fn mul_scale(&self, map: &mut crate::Asset, offset: bevy::math::Vec3) {
        let mut names = map.get_name_map();
        let Some(norm) = map.asset_data.exports[self.transform].get_normal_export_mut() else {
            return;
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name() == SCALE)
        {
            Some(scale) => {
                if let Property::StructProperty(struc) = scale {
                    if let Property::VectorProperty(vec) = &mut struc.value[0] {
                        vec.value.x.0 *= offset.x as f64;
                        vec.value.y.0 *= offset.z as f64;
                        vec.value.z.0 *= offset.y as f64;
                    }
                }
            }
            None => {
                let name = names.get_mut().add_fname(SCALE);
                let struct_type = Some(names.get_mut().add_fname("Vector"));
                norm.properties
                    .push(Property::StructProperty(StructProperty {
                        name,
                        ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                            ancestry: Vec::new(),
                        },
                        struct_type,
                        struct_guid: Some([0; 16].into()),
                        property_guid: None,
                        duplication_index: 0,
                        serialize_none: true,
                        value: vec![Property::VectorProperty(VectorProperty {
                            name: names.get_mut().add_fname(SCALE),
                            ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                                ancestry: Vec::new(),
                            },
                            property_guid: None,
                            duplication_index: 0,
                            value: Vector::new(
                                (offset.x as f64).into(),
                                (offset.z as f64).into(),
                                (offset.y as f64).into(),
                            ),
                        })],
                    }));
            }
        }
    }

    pub fn transform(&self, map: &crate::Asset) -> bevy::prelude::Transform {
        bevy::prelude::Transform {
            translation: self.location(map),
            rotation: self.rotation(map),
            scale: self.scale(map),
        }
    }
}
