use unreal_asset::{
    cast,
    exports::{Export, ExportBaseTrait},
    unreal_types::PackageIndex,
    Asset,
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
