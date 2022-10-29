use unreal_asset::{
    cast,
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    properties::{object_property::ObjectProperty, Property},
    reader::asset_trait::AssetTrait,
    unreal_types::PackageIndex,
    Asset, Import,
};

/// gets all top-level actor exports within a map (all exports which are direct children of PersistentLevel)
pub fn get_actors(asset: &Asset) -> Vec<PackageIndex> {
    match asset
        .exports
        .iter()
        .find(|ex| cast!(Export, LevelExport, ex).is_some())
    {
        Some(ex) => ex
            .get_base_export()
            .create_before_serialization_dependencies
            .clone(),
        None => Vec::new(),
    }
}

/// gets all exports related to the given export
fn get_actor_exports(asset: &Asset, actor: PackageIndex, offset: usize) -> Vec<Export> {
    // get references to all the actor's children
    let mut child_indexes: Vec<PackageIndex> = match asset.get_export(actor) {
        Some(ex) => ex
            .get_base_export()
            .create_before_serialization_dependencies
            .iter()
            .filter(|dep| dep.is_export())
            // dw PackageIndex is just a wrapper around i32 which is cloned by default anyway
            .copied()
            .collect(),
        None => Vec::new(),
    };
    // add the top-level actor reference
    child_indexes.insert(0, actor);

    // get all the exports from those indexes
    let mut children: Vec<Export> = child_indexes
        .iter()
        .filter_map(|index| asset.get_export(*index))
        // i'm pretty sure i have to clone here so i can modify then insert data
        .cloned()
        .collect();

    let package_offset = (offset + 1) as i32;
    // for each PackageIndex, update references in the exports to what they will be once added
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

/// adds an actor to a map where the actor is already present
pub fn clone_actor(asset: &mut Asset, actor: PackageIndex) {
    let len = asset.exports.len();
    let len_package = (len + 1) as i32;
    let mut children = get_actor_exports(asset, actor, len);

    // make sure the actor has a unique object name
    let mut name = children[0].get_base_export().object_name.content.clone();
    while asset.search_name_reference(&name).is_some() {
        name.push('0');
    }
    children[0].get_base_export_mut().object_name = asset.add_fname(&name);

    // add the actor to persistent level
    if let Some(level) = asset
        .exports
        .iter_mut()
        .find_map(|ex| cast!(Export, LevelExport, ex))
    {
        level.index_data.push(len_package);
        level
            .get_base_export_mut()
            .create_before_serialization_dependencies
            .push(PackageIndex::new(len_package));
    }

    // actually add the exports ;p
    asset.exports.append(&mut children);
}

// look at me using idiomatic closures (-3-)
/// performs the provided closure on all of an export's possible references to other exports
fn on_export_refs<F: FnMut(&mut PackageIndex)>(export: &mut Export, mut func: F) {
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in norm.properties.iter_mut() {
            update_props(prop, &mut func);
        }
    }
    let export = export.get_base_export_mut();
    // calls the function on every entry in a list of PackageIndexes
    let mut foreach = |vec: &mut Vec<PackageIndex>| {
        for reference in vec.iter_mut() {
            func(reference);
        }
    };
    foreach(&mut export.create_before_create_dependencies);
    foreach(&mut export.create_before_serialization_dependencies);
    foreach(&mut export.serialization_before_create_dependencies);
    func(&mut export.outer_index);
}

/// performs the provided closure on any possible references stashed away in properties
fn update_props<F: FnMut(&mut PackageIndex)>(prop: &mut Property, func: &mut F) {
    match prop {
        Property::ObjectProperty(ObjectProperty { value, .. }) => {
            func(value);
        }
        Property::ArrayProperty(arr) => {
            for entry in arr.value.iter_mut() {
                update_props(entry, func);
            }
        }
        Property::MapProperty(map) => {
            for val in map.value.values_mut() {
                update_props(val, func);
            }
        }
        Property::SetProperty(set) => {
            for entry in set.value.value.iter_mut() {
                update_props(entry, func);
            }
            for entry in set.removed_items.value.iter_mut() {
                update_props(entry, func);
            }
        }
        Property::StructProperty(struc) => {
            for entry in struc.value.iter_mut() {
                update_props(entry, func);
            }
        }
        _ => (),
    }
}

#[allow(dead_code)]
/// `(INCOMPLETE)` adds an actor to an asset where there was not one originally
// pub fn transfer_actor(recipient: &mut Asset, actor: PackageIndex, donor: &Asset) {
//     // don't say a word about how shittily optimised this is
//     let (child_indexes, mut children) = get_actor_exports(donor, actor);
//     let mut import_indexes = Vec::new();
//     let len = recipient.imports.len();
//     // update import references in exports
//     for child in children.iter_mut() {
//         on_import_refs(child, |index| {
//             if let Some(orig) = donor.get_import(*index) {
//                 // check if already exists in import map
//                 *index = match recipient.imports.iter().position(|import| {
//                     import.class_name.content == orig.class_name.content
//                         && import.class_package.content == orig.class_package.content
//                         && import.object_name.content == orig.object_name.content
//                 }) {
//                     // if the asset already has the import then reference that
//                     Some(pos) => PackageIndex::from_import(pos as i32).unwrap(),
//                     // check if already exists in import list
//                     None => match import_indexes.iter().position(|i| i == index) {
//                         Some(pos) => PackageIndex::from_import((len + pos) as i32).unwrap(),
//                         None => {
//                             import_indexes.push(index.clone());
//                             PackageIndex::from_import((len + import_indexes.len() - 1) as i32)
//                                 .unwrap()
//                         }
//                     },
//                 }
//             }
//         });
//     }

//     // now all export references are sorted we can convert and sort out import references
//     let mut imports: Vec<Import> = import_indexes
//         .iter()
//         .filter_map(|index| donor.get_import(*index))
//         .cloned()
//         .collect();
//     // i'm tired ok >w<
//     let mut dummy = Vec::new();
//     for import in imports.iter_mut() {
//         sort_parent(&recipient, donor, import, &mut dummy);
//     }
//     imports.append(&mut dummy);
//     recipient.imports.append(&mut imports);

//     let len = recipient.exports.len();
//     let len_package = (len + 1) as i32;
//     // for each PackageIndex, update references in the exports to what they will be once added
//     for (i, child_index) in child_indexes.into_iter().enumerate() {
//         for child in children.iter_mut() {
//             on_export_refs(child, |index| {
//                 if index == &child_index {
//                     index.index = len_package + i as i32;
//                 }
//             });
//         }
//     }

//     // make sure the actor has a unique object name
//     let mut name = children[0].get_base_export().object_name.content.clone();
//     while recipient.search_name_reference(&name).is_some() {
//         name.push('0');
//     }
//     children[0].get_base_export_mut().object_name = recipient.add_fname(&name);

//     // add the actor to persistent level
//     if let Some(level) = recipient
//         .exports
//         .iter_mut()
//         .find_map(|ex| cast!(Export, LevelExport, ex))
//     {
//         level.index_data.push(len_package);
//         level
//             .get_base_export_mut()
//             .create_before_serialization_dependencies
//             .push(PackageIndex::new(len_package));
//         // update the references to persistent level in the top-level export
//     }
//     if let Some(level_pos) = recipient
//         .exports
//         .iter()
//         .position(|ex| cast!(Export, LevelExport, ex).is_some())
//     {
//         let top_level = children[0].get_base_export_mut();
//         top_level.outer_index.index = (level_pos + 1) as i32;
//         for index in top_level.create_before_create_dependencies.iter_mut() {
//             index.index = (level_pos + 1) as i32;
//         }
//     }

//     // actually add the exports ;p
//     recipient.exports.append(&mut children);
// }

/// recurses through import tree and resolves imports
fn sort_parent(recipient: &Asset, donor: &Asset, import: &mut Import, imports: &mut Vec<Import>) {
    let len = recipient.imports.len();
    if let Some(orig) = donor.get_import(import.outer_index) {
        // check if already exists in import map
        import.outer_index = match recipient.imports.iter().position(|import| {
            import.class_name.content == orig.class_name.content
                && import.class_package.content == orig.class_package.content
                && import.object_name.content == orig.object_name.content
        }) {
            // if the asset already has the import then reference that
            Some(pos) => PackageIndex::from_import(pos as i32).unwrap(),
            // check if already exists in import list
            None => match imports
                .iter()
                .position(|i| i.outer_index == import.outer_index)
            {
                Some(pos) => PackageIndex::from_import((len + pos) as i32).unwrap(),
                None => {
                    let mut orig = orig.clone();
                    sort_parent(recipient, donor, &mut orig, imports);
                    imports.push(orig.clone());
                    PackageIndex::from_import((len + imports.len() - 1) as i32).unwrap()
                }
            },
        }
    }
}

/// performs the provided closure on all of an export's possible references to imports
fn on_import_refs<F: FnMut(&mut PackageIndex)>(export: &mut Export, mut func: F) {
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in norm.properties.iter_mut() {
            update_props(prop, &mut func);
        }
    }
    let export = export.get_base_export_mut();
    // calls the function on every entry in a list of PackageIndexes
    let mut foreach = |vec: &mut Vec<PackageIndex>| {
        for reference in vec.iter_mut() {
            func(reference);
        }
    };
    foreach(&mut export.create_before_serialization_dependencies);
    foreach(&mut export.serialization_before_serialization_dependencies);
    func(&mut export.class_index);
    func(&mut export.template_index);
    func(&mut export.super_index);
}
