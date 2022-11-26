use unreal_asset::{
    cast,
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    reader::asset_trait::AssetTrait,
    unreal_types::PackageIndex,
    Asset,
};

impl super::Actor {
    pub fn transplant(&self, recipient: &mut Asset, donor: &Asset) {
        let mut children = self.get_actor_exports(donor, donor.exports.len());
        let first_import = recipient.imports.len();
        // resolve all import references from exports
        for child in children.iter_mut() {
            on_import_refs(child, |index| {
                if let Some(import) = donor.get_import(*index) {
                    match recipient.find_import_no_index(
                        &import.class_package,
                        &import.class_name,
                        &import.object_name,
                    ) {
                        Some(existing) => index.index = existing,
                        None => *index = recipient.add_import(import.clone()),
                    }
                }
            })
        }

        // make sure the actor has a unique object name
        super::give_unique_name(
            &mut children[0].get_base_export_mut().object_name,
            recipient,
        );

        // add the actor to persistent level
        if let Some(pos) = recipient
            .exports
            .iter()
            .position(|ex| cast!(Export, LevelExport, ex).is_some())
        {
            let export_offset = recipient.exports.len() as i32 + 1;
            // update actor outer index
            children[0].get_base_export_mut().outer_index = PackageIndex::new(export_offset);
            children[0]
                .get_base_export_mut()
                .create_before_create_dependencies = vec![PackageIndex::new(export_offset)];
            let level = cast!(Export, LevelExport, &mut recipient.exports[pos]).unwrap();
            level.index_data.push(export_offset);
            level
                .get_base_export_mut()
                .create_before_serialization_dependencies
                .push(PackageIndex::new(export_offset));
        }
        // finally add the exports
        recipient.exports.append(&mut children);

        let offset = recipient.imports.len();
        // if no imports were added then taking a slice would panic
        if offset == first_import {
            return;
        }
        todo!("resolve all import references from new imports");
    }
}

/// on all of an export's possible references to imports
fn on_import_refs(export: &mut Export, mut func: impl FnMut(&mut PackageIndex)) {
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in norm.properties.iter_mut() {
            super::update_props(prop, &mut func);
        }
    }
    let export = export.get_base_export_mut();
    func(&mut export.class_index);
    func(&mut export.template_index);
    export
        .serialization_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
}
