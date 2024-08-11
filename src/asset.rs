use std::{fs::File, io::BufReader, path::Path};

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

/// creates an asset from the specified path and version
pub fn open(file: impl AsRef<Path>, version: EngineVersion) -> Result<super::Asset, Error> {
    Asset::new(
        BufReader::new(File::open(&file)?),
        File::open(file.as_ref().with_extension("uexp"))
            .ok()
            .map(BufReader::new),
        version,
        None,
    )
}

/// saves an asset's data to the specified path
pub fn save<C: std::io::Read + std::io::Seek>(
    asset: &mut Asset<C>,
    path: impl AsRef<Path>,
) -> Result<(), Error> {
    asset.rebuild_name_map();
    asset.write_data(
        &mut File::create(&path)?,
        asset
            .asset_data
            .use_event_driven_loader
            .then(|| File::create(path.as_ref().with_extension("uexp")))
            .transpose()?
            .as_mut(),
    )
}

pub fn get<T>(
    game: &str,
    pak: &repak::PakReader,
    pak_file: &std::path::Path,
    cache: Option<&std::path::Path>,
    path: &str,
    version: unreal_asset::engine_version::EngineVersion,
    func: impl Fn(
        unreal_asset::Asset<super::Wrapper>,
        Option<super::Wrapper>,
    ) -> Result<T, unreal_asset::error::Error>,
) -> Result<T, unreal_asset::error::Error> {
    let pak_file = &mut std::io::BufReader::new(std::fs::File::open(pak_file)?);
    let path = path
        .replace("/Game", &format!("{}/Content", game))
        .replace("/Engine/", "Engine/Content/");
    let make = |ext: &str| path.to_string() + ext;
    let (mesh, exp, bulk, uptnl) = (
        make(".uasset"),
        make(".uexp"),
        make(".ubulk"),
        make(".uptnl"),
    );
    let cache_path = |path: &str| cache.unwrap().join(path.trim_start_matches('/'));
    match cache {
        Some(_)
            if cache_path(&mesh).exists() ||
            // try to create cache if it doesn't exist
            (
                std::fs::create_dir_all(cache_path(&path).parent().unwrap()).is_ok() &&
                pak.read_file(&mesh, pak_file, &mut std::fs::File::create(cache_path(&mesh))?).is_ok() &&
                // we don't care whether these are successful in case they don't exist
                pak.read_file(&exp, pak_file, &mut std::fs::File::create(cache_path(&exp))?).map_or(true,|_| true) &&
                pak.read_file(&bulk, pak_file, &mut std::fs::File::create(cache_path(&bulk))?).map_or(true,|_| true) &&
                pak.read_file(&uptnl, pak_file, &mut std::fs::File::create(cache_path(&uptnl))?).map_or(true,|_| true)
             ) =>
        {
            func(
                unreal_asset::Asset::new(
                    super::Wrapper::File(std::io::BufReader::new(std::fs::File::open(
                        cache_path(&mesh),
                    )?)),
                    std::fs::File::open(cache_path(&exp))
                        .ok()
                        .map(std::io::BufReader::new)
                        .map(super::Wrapper::File),
                    version,
                    None,
                )?,
                std::fs::File::open(cache_path(&bulk))
                    .ok()
                    .map_or_else(|| std::fs::File::open(cache_path(&uptnl)).ok(), Some)
                    .map(std::io::BufReader::new)
                    .map(super::Wrapper::File),
            )
        }
        // if the cache cannot be created fall back to storing in memory
        _ => func(
            unreal_asset::Asset::new(
                super::Wrapper::Bytes(std::io::Cursor::new(pak.get(&mesh, pak_file).map_err(
                    |e| {
                        unreal_asset::error::Error::no_data(match e {
                            repak::Error::Oodle => "oodle paks are unsupported atm".to_string(),
                            e => format!("error reading pak: {e}"),
                        })
                    },
                )?)),
                pak.get(&exp, pak_file)
                    .ok()
                    .map(std::io::Cursor::new)
                    .map(super::Wrapper::Bytes),
                version,
                None,
            )?,
            pak.get(&bulk, pak_file)
                .ok()
                .map_or_else(|| pak.get(&uptnl, pak_file).ok(), Some)
                .map(std::io::Cursor::new)
                .map(super::Wrapper::Bytes),
        ),
    }
}
