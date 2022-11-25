use unreal_asset::{
    cast,
    exports::{Export, ExportBaseTrait},
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
        let mut id: u16 = match name.rfind(|ch: char| ch.to_digit(10).is_none()) {
            Some(index) if index != name.len() => name
                .drain(index + 1..)
                .collect::<String>()
                .parse()
                .unwrap_or_default(),
            _ => 0,
        };
        while asset
            .search_name_reference(&format!("{}{}", &name, id))
            .is_some()
        {
            id += 1;
        }
        children[0].get_base_export_mut().object_name = asset.add_fname(&(name + &id.to_string()));

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
}
