use unreal_asset::{
    cast,
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    reader::archive_trait::ArchiveTrait,
    types::PackageIndex,
    Asset, Import,
};

impl super::Actor {
    pub fn transplant(&self, recipient: &mut Asset<std::fs::File>, donor: &Asset<std::fs::File>) {
        let mut children = self.get_actor_exports(donor, recipient.asset_data.exports.len());

        // make sure the actor has a unique object name
        super::give_unique_name(
            &mut children[0].get_base_export_mut().object_name,
            recipient,
        );

        let actor_ref = PackageIndex::new(recipient.asset_data.exports.len() as i32 + 1);
        // add the actor to persistent level
        if let Some((pos, level)) = recipient
            .asset_data
            .exports
            .iter_mut()
            // least awkward way to get position and reference
            .enumerate()
            .find_map(|(i, ex)| cast!(Export, LevelExport, ex).map(|level| (i, level)))
        {
            // update actor's level reference
            let level_ref = PackageIndex::new(pos as i32 + 1);
            children[0].get_base_export_mut().outer_index = level_ref;
            children[0]
                .get_base_export_mut()
                .create_before_create_dependencies[0] = level_ref;
            // add actor to level data
            level.actors.push(actor_ref);
            level
                .get_base_export_mut()
                .create_before_serialization_dependencies
                .push(actor_ref);
        }
        // resolve all import references from exports
        let import_offset = recipient.imports.len() as i32;
        let mut imports = Vec::new();
        for child in children.iter_mut() {
            on_import_refs(child, |index| {
                let Some(import) = donor.get_import(*index) else {
                    return;
                };
                index.index = match recipient.find_import_no_index(
                    &import.class_package,
                    &import.class_name,
                    &import.object_name,
                ) {
                    // sometimes e.g for GEN_VARIABLEs you want those imports
                    Some(existing)
                        if donor.get_import(import.outer_index).is_some_and(|imp| {
                            recipient
                                .get_import(PackageIndex::new(existing))
                                .and_then(|existing| recipient.get_import(existing.outer_index))
                                .is_some_and(|import| {
                                    imp.class_package.eq_content(&import.class_package)
                                        && imp.class_name.eq_content(&import.class_name)
                                        && imp.object_name.eq_content(&import.object_name)
                                })
                        }) =>
                    {
                        existing
                    }
                    _ => {
                        -import_offset
                            - match imports.iter().position(|imp: &Import| {
                                imp.class_package.eq_content(&import.class_package)
                                    && imp.class_name.eq_content(&import.class_name)
                                    && imp.object_name.eq_content(&import.object_name)
                            }) {
                                Some(existing) => existing + 1,
                                None => {
                                    imports.push(import.clone());
                                    // this actually pads perfectly so no need for + 1
                                    imports.len()
                                }
                            } as i32
                    }
                }
            })
        }
        // finally add the exports
        recipient.asset_data.exports.append(&mut children);

        // resolve all import references from exports
        let mut i = 0;
        // use a while loop because the vector is expanding while the operation occurs & imports.len() updates every loop
        while i < imports.len() {
            let Some(parent) = donor.get_import(imports[i].outer_index) else {
                i += 1;
                continue;
            };
            imports[i].outer_index.index = match recipient.find_import_no_index(
                &parent.class_package,
                &parent.class_name,
                &parent.object_name,
            ) {
                Some(existing)
                    if donor.get_import(parent.outer_index).is_some_and(|imp| {
                        recipient
                            .get_import(PackageIndex::new(existing))
                            .and_then(|existing| recipient.get_import(existing.outer_index))
                            .is_some_and(|import| {
                                imp.class_package.eq_content(&import.class_package)
                                    && imp.class_name.eq_content(&import.class_name)
                                    && imp.object_name.eq_content(&import.object_name)
                            })
                    }) =>
                {
                    existing
                }
                _ => {
                    -import_offset
                        - match imports.iter().position(|import: &Import| {
                            import.class_package.eq_content(&parent.class_package)
                                && import.class_name.eq_content(&parent.class_name)
                                && import.object_name.eq_content(&parent.object_name)
                        }) {
                            Some(existing) => existing + 1,
                            None => {
                                imports.push(parent.clone());
                                // this actually pads perfectly so no need for + 1
                                imports.len()
                            }
                        } as i32
                }
            };
            i += 1;
        }
        recipient.imports.append(&mut imports);
    }
}

/// on all of an export's possible references to imports
fn on_import_refs(export: &mut Export, mut func: impl FnMut(&mut PackageIndex)) {
    if let Some(norm) = export.get_normal_export_mut() {
        for prop in norm.properties.iter_mut() {
            super::on_props(prop, &mut func);
        }
    }
    let export = export.get_base_export_mut();
    func(&mut export.class_index);
    func(&mut export.template_index);
    // not serialization_before_serialization because only the first few map exports have those
    export
        .serialization_before_create_dependencies
        .iter_mut()
        .for_each(&mut func);
    export
        .create_before_serialization_dependencies
        .iter_mut()
        .for_each(&mut func);
}
