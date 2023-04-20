use std::{fs::File, path::Path};

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

/// creates an asset from the specified path and version
pub fn open(file: impl AsRef<Path>, version: EngineVersion) -> Result<Asset<File>, Error> {
    Asset::new(
        File::open(&file)?,
        File::open(file.as_ref().with_extension("uexp")).ok(),
        version,
    )
}

/// saves an asset's data to the specified path
pub fn save<C: std::io::Read + std::io::Seek>(
    asset: &mut Asset<C>,
    path: impl AsRef<Path>,
) -> Result<(), Error> {
    asset.rebuild_name_map()?;
    asset.write_data(
        &mut File::create(&path)?,
        Some(&mut File::create(path.as_ref().with_extension("uexp"))?),
    )
}
