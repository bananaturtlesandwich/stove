use std::{fs::File, path::Path};

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

/// creates an asset from the specified path and version
pub fn open(file: impl AsRef<Path>, version: EngineVersion) -> Result<Asset<File>, Error> {
    let mut asset = Asset::new(
        File::open(&file)?,
        File::open(file.as_ref().with_extension("uexp")).ok(),
    );
    asset.set_engine_version(version);
    asset.parse_data()?;
    Ok(asset)
}

/// saves an asset's data to the specified path
pub fn save(asset: &mut Asset<File>, path: impl AsRef<Path>) -> Result<(), Error> {
    loop {
        match asset.write_data(
            &mut File::create(path.as_ref().with_extension("umap"))?,
            asset
                .use_separate_bulk_data_files
                .then_some(&mut File::create(path.as_ref().with_extension("uexp"))?),
        ) {
            Ok(_) => break Ok(()),
            Err(e) if e.to_string().starts_with("name reference for ") => {
                asset.add_fname(
                    e.to_string()
                        .trim_start_matches("name reference for ")
                        .trim_end_matches(" not found"),
                );
            }
            e => break e,
        }
    }
}
