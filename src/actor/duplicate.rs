use super::*;

impl Actor {
    /// adds an actor to a map where the actor is already present
    pub fn duplicate(&self, asset: &mut Asset, export_names: &mut Vec<String>) {
        let len = asset.asset_data.exports.len();
        let mut children = self.get_actor_exports(asset, len);

        // make sure the actor has a unique object name
        give_unique_name(&mut children[0].get_base_export_mut().object_name, asset);

        let actor_ref = PackageIndex::new(len as i32 + 1);
        // add the actor to persistent level
        if let Some(level) = asset
            .asset_data
            .exports
            .iter_mut()
            .find_map(|ex| cast!(Export, LevelExport, ex))
        {
            level.actors.push(actor_ref);
            level
                .get_base_export_mut()
                .create_before_serialization_dependencies
                .push(actor_ref);
        }

        export_names.extend(
            children
                .iter()
                .map(|ex| ex.get_base_export().object_name.get_owned_content()),
        );
        // actually add the exports ;p
        asset.asset_data.exports.append(&mut children);
    }
}
