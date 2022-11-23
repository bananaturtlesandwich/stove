use unreal_asset::{
    cast,
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    properties::Property,
    unreal_types::PackageIndex,
    Asset,
};

impl super::Actor {
    /// adds an actor to a map where the actor is already present
    pub fn duplicate(&self, asset: &mut Asset) {
        let len = asset.exports.len();
        let len_package = (len + 1) as i32;
        let mut children = self.get_actor_exports(asset, len);

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

    /// gets all exports related to the given actor
    fn get_actor_exports(&self, asset: &Asset, offset: usize) -> Vec<Export> {
        // get references to all the actor's children
        let mut child_indexes: Vec<PackageIndex> = asset.exports[self.export]
            .get_base_export()
            .create_before_serialization_dependencies
            .iter()
            .filter(|dep| dep.is_export())
            // dw PackageIndex is just a wrapper around i32 which is cloned by default anyway
            .cloned()
            .collect();
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
}

/// performs the provided closure on all of an export's possible references to other exports
fn on_export_refs(export: &mut Export, mut func: impl FnMut(&mut PackageIndex)) {
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in norm.properties.iter_mut() {
            update_props(prop, &mut func);
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

/// performs the provided closure on any possible references stashed away in properties
fn update_props(prop: &mut Property, func: &mut impl FnMut(&mut PackageIndex)) {
    match prop {
        Property::ObjectProperty(obj) => {
            func(&mut obj.value);
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

// transferring is being developed over at:
// https://github.com/bananaturtlesandwich/blue-fire-rando/blob/master/src/map_utils.rs
