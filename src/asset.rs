use std::fs;
use std::io;
use std::path::Path;

use unreal_asset::error::Error;
use unreal_asset::flags::EPackageFlags;
use unreal_asset::Asset;

/// creates an asset from the specified path and version
pub fn open(file: impl AsRef<Path>, version: i32) -> Result<Asset, Error> {
    let bulk = file.as_ref().with_extension("uexp");
    let mut asset = Asset::new(
        fs::read(&file)?,
        match bulk.exists() {
            true => Some(fs::read(bulk)?),
            // the none option is given as some uassets may not use the event driven loader
            false => None,
        },
    );
    asset.engine_version = version;
    asset.parse_data()?;
    Ok(asset)
}

/// saves an asset's data to the specified path
pub fn save(asset: &Asset, path: impl AsRef<Path>) -> Result<(), Error> {
    let mut main = io::Cursor::new(Vec::new());
    let mut bulk = main.clone();
    asset.write_data(
        &mut main,
        match asset.use_separate_bulk_data_files {
            true => Some(&mut bulk),
            false => None,
        },
    )?;
    fs::write(
        path.as_ref().with_extension(
            match EPackageFlags::from_bits_truncate(asset.package_flags)
                .intersects(EPackageFlags::PKG_CONTAINS_MAP)
            {
                true => "umap",
                false => "uasset",
            },
        ),
        main.into_inner(),
    )?;
    // if the asset has no bulk data then the bulk cursor will be empty
    if asset.use_separate_bulk_data_files {
        fs::write(path.as_ref().with_extension("uexp"), bulk.into_inner())?;
    }
    Ok(())
}
