use std::fs::File;
use unreal_asset::{
    cast,
    error::Error,
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    properties::{Property, PropertyDataTrait},
    reader::archive_trait::ArchiveTrait,
    types::{fname::FName, PackageIndex},
    Asset,
};

mod delete;
mod duplicate;
mod transform;
mod transplant;
mod ui;

pub enum DrawType {
    Mesh(String),
    Cube,
}

#[derive(bevy::prelude::Bundle)]
pub struct SelectedBundle {
    selected: Selected,
    outline: bevy_mod_outline::OutlineBundle,
}

impl Default for SelectedBundle {
    fn default() -> Self {
        Self {
            selected: Selected,
            outline: bevy_mod_outline::OutlineBundle {
                outline: bevy_mod_outline::OutlineVolume {
                    visible: true,
                    colour: bevy::prelude::Color::rgb(1.0, 1.0, 0.5),
                    width: 15.0,
                },
                ..Default::default()
            },
        }
    }
}

#[derive(bevy::prelude::Component)]
pub struct Selected;

#[derive(bevy::prelude::Component)]
pub struct Matched;

#[derive(bevy::prelude::Component)]
pub struct Actor {
    pub export: usize,
    transform: usize,
    pub name: String,
    pub class: String,
    pub draw_type: DrawType,
}

impl Actor {
    fn index(&self) -> PackageIndex {
        PackageIndex::new(self.export as i32 + 1)
    }

    pub fn new(asset: &crate::Asset, package: PackageIndex) -> Result<Self, Error> {
        if package.index == 0 {
            return Err(Error::invalid_package_index(
                "actor was null reference".to_string(),
            ));
        }
        let export = package.index as usize - 1;
        let Some(ex) = asset.get_export(package) else {
            return Err(Error::invalid_package_index(format!(
                "failed to find actor at index {}",
                package.index
            )));
        };
        let Some(norm) = ex.get_normal_export() else {
            return Err(Error::no_data(format!(
                "actor at index {} failed to parse",
                package.index
            )));
        };
        let name = match asset.get_engine_version()
            >= unreal_asset::engine_version::EngineVersion::VER_UE5_1
        {
            true => match norm.extras[8..12]
                .try_into()
                .ok()
                .map(|i| i32::from_le_bytes(i) as usize)
                .and_then(|len| String::from_utf8(norm.extras[12..12 + len].to_vec()).ok())
            {
                Some(name) if !name.chars().all(char::is_whitespace) => name,
                _ => norm.base_export.object_name.get_owned_content(),
            },
            false => norm.base_export.object_name.get_owned_content(),
        };
        let class = asset
            .get_import(norm.base_export.class_index)
            .map(|import| import.object_name.get_owned_content())
            .unwrap_or_default();
        let draw_type = norm
            .properties
            .iter()
            .find_map(|comp| {
                cast!(Property, ObjectProperty, comp)
                    .filter(|_| comp.get_name() == "StaticMeshComponent")
            })
            .and_then(|i| asset.get_export(i.value))
            .and_then(Export::get_normal_export)
            .and_then(|mesh| {
                mesh.properties.iter().find_map(|mesh| {
                    cast!(Property, ObjectProperty, mesh)
                        .filter(|_| mesh.get_name() == "StaticMesh")
                })
            })
            .and_then(|i| asset.get_import(i.value))
            .and_then(|i| asset.get_import(i.outer_index))
            .map_or(DrawType::Cube, |path| {
                DrawType::Mesh(path.object_name.get_owned_content())
            });
        // .base_export
        // .create_before_serialization_dependencies
        // .iter()
        // .filter_map(|i| asset.get_export(*i))
        // .filter_map(Export::get_normal_export)
        // .find(|i| {
        //     asset
        //         .get_import(i.get_base_export().class_index)
        //         .is_some_and(|import| import.object_name == "StaticMeshComponent")
        // })
        // .and_then(|norm| {
        //     norm.properties.iter().find_map(|prop| {
        //         cast!(Property, ObjectProperty, prop)
        //             .filter(|prop| prop.get_name() == "StaticMesh")
        //     })
        // })
        // .and_then(|obj| asset.get_import(obj.value))
        // .and_then(|import| asset.get_import(import.outer_index))
        // .map_or(DrawType::Cube, |path| {
        //     DrawType::Mesh(path.object_name.get_owned_content())
        // });
        // normally these are further back so reversed should be a bit faster
        for prop in norm.properties.iter().rev() {
            match prop.get_name().get_owned_content().as_str() {
                // of course this wouldn't be able to be detected if all transforms were left default
                "RelativeLocation" | "RelativeRotation" | "RelativeScale3D" => {
                    return Ok(Self {
                        export,
                        transform: export,
                        name,
                        class,
                        draw_type,
                    })
                }
                "RootComponent" => {
                    if let Property::ObjectProperty(obj) = prop {
                        if obj.value.is_export() {
                            return Ok(Self {
                                export,
                                transform: obj.value.index as usize - 1,
                                name,
                                class,
                                draw_type,
                            });
                        }
                    }
                }
                _ => continue,
            }
        }
        norm.base_export.object_name.get_content(|name| {
            Err(Error::no_data(format!(
                "couldn't find transform component for {name}",
            )))
        })
    }

    /// gets all exports related to the given actor
    fn get_actor_exports(
        &self,
        asset: &Asset<std::io::BufReader<File>>,
        offset: usize,
    ) -> Vec<Export> {
        let level = asset
            .asset_data
            .exports
            .iter()
            .find_map(|ex| unreal_asset::cast!(Export, LevelExport, ex))
            .unwrap();
        // get references to all the actor's children
        let mut child_indexes: Vec<PackageIndex> = asset.asset_data.exports[self.export]
            .get_base_export()
            .create_before_serialization_dependencies
            .iter()
            .filter(|dep| dep.is_export())
            // dw PackageIndex is just a wrapper around i32 which is cloned by default anyway
            .cloned()
            .collect();
        let actors: Vec<_> = child_indexes
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(i, child)| level.actors.contains(child).then_some(i))
            .collect();
        for i in actors {
            child_indexes.remove(i);
        }
        // add the top-level actor reference
        child_indexes.insert(0, self.index());

        // get all the exports from those indexes
        let mut children: Vec<Export> = child_indexes
            .iter()
            .filter_map(|index| asset.get_export(*index))
            // i'm pretty sure i have to clone here so i can modify then insert data
            .cloned()
            .collect();

        let package_offset = (offset + 1) as i32;
        // update export references to what they will be once added
        for (i, child_index) in child_indexes.into_iter().enumerate() {
            for child in children.iter_mut() {
                on_export_refs(child, |index| {
                    if index == &child_index {
                        index.index = package_offset + i as i32;
                    }
                });
            }
        }
        children
    }
}

/// gets all actor exports within a map (all exports direct children of PersistentLevel)
pub fn get_actors(asset: &crate::Asset) -> Vec<PackageIndex> {
    match asset
        .asset_data
        .exports
        .iter()
        .find_map(|ex| cast!(Export, LevelExport, ex))
    {
        Some(level) => level
            .actors
            .iter()
            .filter(|index| index.is_export())
            .copied()
            .collect(),
        None => Vec::new(),
    }
}

/// creates and assigns a unique name
fn give_unique_name(orig: &mut FName, asset: &mut crate::Asset) {
    // for the cases where the number is unnecessary
    let mut name = orig.get_owned_content();
    if asset.search_name_reference(&name).is_none() {
        *orig = asset.add_fname(&name);
        return;
    }
    let mut id: u16 = match name.rfind(|ch: char| ch.to_digit(10).is_none()) {
        Some(index) if index != name.len() - 1 => {
            name.drain(index + 1..).collect::<String>().parse().unwrap()
        }
        _ => 1,
    };
    while asset
        .search_name_reference(&format!("{}{}", &name, id))
        .is_some()
    {
        id += 1;
    }
    *orig = asset.add_fname(&(name + &id.to_string()))
}

/// on all possible export references
fn on_export_refs(export: &mut Export, mut func: impl FnMut(&mut PackageIndex)) {
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in norm.properties.iter_mut() {
            on_prop_refs(prop, &mut func);
        }
    }
    let export = export.get_base_export_mut();
    export
        .create_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
    export
        .create_before_serialization_dependencies
        .iter_mut()
        .for_each(&mut func);
    export
        .serialization_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
    func(&mut export.outer_index);
}

fn on_props(prop: &mut Property, func: &mut impl FnMut(&mut Property)) {
    match prop {
        Property::ArrayProperty(arr) => {
            for entry in arr.value.iter_mut() {
                on_props(entry, func);
            }
        }
        Property::MapProperty(map) => {
            for val in map.value.values_mut() {
                on_props(val, func);
            }
        }
        Property::SetProperty(set) => {
            for entry in set.value.value.iter_mut() {
                on_props(entry, func);
            }
            for entry in set.removed_items.value.iter_mut() {
                on_props(entry, func);
            }
        }
        Property::StructProperty(struc) => {
            for entry in struc.value.iter_mut() {
                on_props(entry, func);
            }
        }
        prop => func(prop),
    }
}

/// on any possible references stashed away in properties
fn on_prop_refs(prop: &mut Property, func: &mut impl FnMut(&mut PackageIndex)) {
    on_props(prop, &mut |prop| match prop {
        Property::ObjectProperty(obj) => {
            func(&mut obj.value);
        }
        Property::DelegateProperty(del) => func(&mut del.value.object),
        Property::MulticastDelegateProperty(del) => {
            for delegate in del.value.iter_mut() {
                func(&mut delegate.object)
            }
        }
        Property::MulticastSparseDelegateProperty(del) => {
            for delegate in del.value.iter_mut() {
                func(&mut delegate.object)
            }
        }
        Property::MulticastInlineDelegateProperty(del) => {
            for delegate in del.value.iter_mut() {
                func(&mut delegate.object)
            }
        }
        _ => (),
    })
}
