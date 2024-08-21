use std::{fs::File, io::BufReader, path::Path};

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

/// creates an asset from the specified path and version
pub fn open(file: impl AsRef<Path>, version: EngineVersion) -> Result<super::Asset, Error> {
    Asset::new(
        super::Wrapper::File(BufReader::new(File::open(&file)?)),
        File::open(file.as_ref().with_extension("uexp"))
            .ok()
            .map(BufReader::new)
            .map(super::Wrapper::File),
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

#[test]
fn path_with_mount() {
    let game = "OMD";
    let path = "/Game/Environments/CastleOrder/Railing/Mesh/RailingShortC";
    let mount = "../../../OMD/Content/Environments/CastleOrder/";
    let path: String = path
        .replace("/Game", &format!("{}/Content", game))
        .replace("/Engine/", "Engine/Content/")
        .trim_start_matches(mount.trim_start_matches("../../../"))
        .into();
    assert_eq!(path, "Railing/Mesh/RailingShortC")
}

pub fn get<T>(
    paks: &super::Paks,
    cache: Option<&std::path::Path>,
    path: &str,
    version: unreal_asset::engine_version::EngineVersion,
    func: impl Fn(
        unreal_asset::Asset<super::Wrapper>,
        Option<super::Wrapper>,
    ) -> Result<T, unreal_asset::error::Error>,
) -> Option<T> {
    let path = path
        .replace("/Game", &format!("{}/Content", paks.0))
        .replace("/Engine/", "Engine/Content/");
    let loose = paks.1.join(&path);
    let mesh = loose.with_extension("uasset");
    if mesh.exists() {
        if let Ok(asset) = open(mesh, version).and_then(|asset| {
            func(
                asset,
                std::fs::File::open(loose.with_extension("ubulk"))
                    .ok()
                    .map_or_else(
                        || std::fs::File::open(loose.with_extension("uptnl")).ok(),
                        Some,
                    )
                    .map(std::io::BufReader::new)
                    .map(super::Wrapper::File),
            )
        }) {
            return Some(asset);
        }
    }
    for (pak_file, pak) in paks.2.iter() {
        if let Ok(asset) = read(pak, pak_file, cache, &path, version, &func) {
            return Some(asset);
        }
    }
    None
}

fn read<T>(
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
    let path: String = path
        .trim_start_matches(pak.mount_point().trim_start_matches("../../../"))
        .into();
    let make = |ext: &str| path.clone() + ext;
    let (mesh, exp, bulk, uptnl) = (
        make(".uasset"),
        make(".uexp"),
        make(".ubulk"),
        make(".uptnl"),
    );
    let cached = |path: &str| cache.unwrap().join(path.trim_start_matches('/'));
    match cache {
        Some(_)
            if cached(&mesh).exists() ||
            // try to create cache if it doesn't exist
            (
                std::fs::create_dir_all(cached(&path).parent().unwrap()).is_ok() &&
                pak.read_file(&mesh, pak_file, &mut std::fs::File::create(cached(&mesh))?).is_ok() &&
                // we don't care whether these are successful in case they don't exist
                pak.read_file(&exp, pak_file, &mut std::fs::File::create(cached(&exp))?).map_or(true,|_| true) &&
                pak.read_file(&bulk, pak_file, &mut std::fs::File::create(cached(&bulk))?).map_or(true,|_| true) &&
                pak.read_file(&uptnl, pak_file, &mut std::fs::File::create(cached(&uptnl))?).map_or(true,|_| true)
             ) =>
        {
            func(
                unreal_asset::Asset::new(
                    super::Wrapper::File(std::io::BufReader::new(std::fs::File::open(cached(
                        &mesh,
                    ))?)),
                    std::fs::File::open(cached(&exp))
                        .ok()
                        .map(std::io::BufReader::new)
                        .map(super::Wrapper::File),
                    version,
                    None,
                )?,
                std::fs::File::open(cached(&bulk))
                    .ok()
                    .map_or_else(|| std::fs::File::open(cached(&uptnl)).ok(), Some)
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
