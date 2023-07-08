use std::fs::File;
use unreal_asset::{
    cast,
    exports::ExportNormalTrait,
    properties::{
        struct_property::StructProperty,
        vector_property::{RotatorProperty, VectorProperty},
        Property, PropertyDataTrait,
    },
    types::vector::Vector,
    Asset,
};

pub const LOCATION: &str = "RelativeLocation";
pub const ROTATION: &str = "RelativeRotation";
pub const SCALE: &str = "RelativeScale3D";

impl super::Actor {
    pub fn location(&self, map: &Asset<File>) -> glam::Vec3 {
        map.asset_data.exports[self.transform]
            .get_normal_export()
            .and_then(|norm| {
                norm.properties.iter().rev().find_map(|prop| {
                    if let Property::StructProperty(struc) = prop {
                        if struc.name.get_content() == LOCATION {
                            return cast!(Property, VectorProperty, &struc.value[0]);
                        }
                    }
                    None
                })
            })
            .map(|pos| glam::dvec3(-pos.value.x.0, pos.value.z.0, pos.value.y.0).as_vec3() * 0.01)
            .unwrap_or_default()
    }

    pub fn set_location(&self, map: &mut Asset<File>, mut new: glam::Vec3) {
        new *= 100.0;
        let mut names = map.get_name_map();
        let Some(norm) = map.asset_data.exports[self.transform].get_normal_export_mut() else {
            return
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name().get_content() == LOCATION)
        {
            Some(scale) => {
                if let Property::StructProperty(struc) = scale {
                    if let Property::VectorProperty(vec) = &mut struc.value[0] {
                        vec.value.x.0 = -new.x as f64;
                        vec.value.y.0 = new.z as f64;
                        vec.value.z.0 = new.y as f64;
                    }
                }
            }
            None => norm
                .properties
                .push(Property::StructProperty(StructProperty {
                    name: names
                        .clone_resource()
                        .get_mut()
                        .add_fname(LOCATION),
                    ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                        ancestry: Vec::new(),
                    },
                    struct_type: Some(names.clone_resource().get_mut().add_fname("Vector")),
                    struct_guid: Some([0; 16]),
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
                            (new.x as f64).into(),
                            (new.z as f64).into(),
                            (new.y as f64).into(),
                        ),
                    })],
                })),
        }
    }

    pub fn add_location(&self, map: &mut Asset<File>, offset: glam::Vec3) {
        let mut names = map.get_name_map();
        let Some(norm) = map.asset_data.exports[self.transform].get_normal_export_mut() else {
            return
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name().get_content() == LOCATION)
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
            None => norm
                .properties
                .push(Property::StructProperty(StructProperty {
                    name: names
                        .clone_resource()
                        .get_mut()
                        .add_fname(LOCATION),
                    ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                        ancestry: Vec::new(),
                    },
                    struct_type: Some(names.clone_resource().get_mut().add_fname("Vector")),
                    struct_guid: Some([0; 16]),
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
                })),
        }
    }

    pub fn rotation(&self, map: &Asset<File>) -> glam::Vec3 {
        map.asset_data.exports[self.transform]
            .get_normal_export()
            .map(|norm| {
                norm.properties
                    .iter()
                    .rev()
                    .find_map(|prop| {
                        if let Property::StructProperty(struc) = prop {
                            if struc.name.get_content() == ROTATION {
                                return cast!(Property, RotatorProperty, &struc.value[0]);
                            }
                        }
                        None
                    })
                    .map(|rot| glam::dvec3(rot.value.x.0, rot.value.y.0, rot.value.z.0).as_vec3())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn combine_rotation(&self, map: &mut Asset<File>, offset: glam::Quat) {
        let mut names = map.get_name_map();
        let Some(norm) = map.asset_data.exports[self.transform].get_normal_export_mut() else {
            return;
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name().get_content() == ROTATION)
        {
            Some(scale) => {
                if let Property::StructProperty(struc) = scale {
                    if let Property::RotatorProperty(vec) = &mut struc.value[0] {
                        (vec.value.x.0, vec.value.y.0, vec.value.z.0) = (offset.as_f64()
                            * glam::DQuat::from_euler(
                                glam::EulerRot::XYZ,
                                vec.value.x.0.to_radians(),
                                vec.value.y.0.to_radians(),
                                vec.value.z.0.to_radians(),
                            ))
                        .to_euler(glam::EulerRot::XYZ);
                        (vec.value.x.0, vec.value.y.0, vec.value.z.0) = (
                            vec.value.x.0.to_degrees(),
                            vec.value.y.0.to_degrees(),
                            vec.value.z.0.to_degrees(),
                        );
                    }
                }
            }
            None => norm
                .properties
                .push(Property::StructProperty(StructProperty {
                    name: names
                        .clone_resource()
                        .get_mut()
                        .add_fname(ROTATION),
                    ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                        ancestry: Vec::new(),
                    },
                    struct_type: Some(names.clone_resource().get_mut().add_fname("Rotator")),
                    struct_guid: Some([0; 16]),
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
                })),
        }
    }

    pub fn scale(&self, map: &Asset<File>) -> glam::Vec3 {
        map.asset_data.exports[self.transform]
            .get_normal_export()
            .and_then(|norm| {
                norm.properties.iter().rev().find_map(|prop| {
                    if let Property::StructProperty(struc) = prop {
                        if struc.name.get_content() == SCALE {
                            return cast!(Property, VectorProperty, &struc.value[0]);
                        }
                    }
                    None
                })
            })
            .map(|rot| glam::dvec3(rot.value.x.0, rot.value.z.0, rot.value.y.0).as_vec3())
            .unwrap_or(glam::Vec3::ONE)
    }

    pub fn mul_scale(&self, map: &mut Asset<File>, offset: glam::Vec3) {
        let mut names = map.get_name_map();
        let Some(norm) = map.asset_data.exports[self.transform].get_normal_export_mut() else {
            return;
        };
        match norm
            .properties
            .iter_mut()
            .find(|prop| prop.get_name().get_content() == SCALE)
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
            None => norm
                .properties
                .push(Property::StructProperty(StructProperty {
                    name: names
                        .clone_resource()
                        .get_mut()
                        .add_fname(SCALE),
                    ancestry: unreal_asset::unversioned::ancestry::Ancestry {
                        ancestry: Vec::new(),
                    },
                    struct_type: Some(names.clone_resource().get_mut().add_fname("Vector")),
                    struct_guid: Some([0; 16]),
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
                })),
        }
    }

    pub fn model_matrix(&self, map: &Asset<File>) -> glam::Mat4 {
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
