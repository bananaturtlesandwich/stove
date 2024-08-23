use super::*;

pub fn duplicate(
    _: Trigger<triggers::Duplicate>,
    mut notif: EventWriter<Notif>,
    mut commands: Commands,
    mut map: NonSendMut<Map>,
    registry: Res<Registry>,
    consts: Res<Constants>,
    selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
    mut cubes: Query<&mut Handle<wire::Wire>>,
) {
    let Some((map, _, export_names, _)) = &mut map.0 else {
        return;
    };
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to duplicate".into(),
            kind: Warning,
        });
        return;
    }
    for (entity, actor, ..) in selected.iter() {
        match cubes.get_mut(entity) {
            Ok(mut mat) => {
                commands.entity(entity).remove::<actor::Selected>();
                *mat = consts.unselected.clone_weak();
            }
            Err(_) => {
                commands.entity(entity).remove::<actor::SelectedBundle>();
            }
        }
        let len = map.asset_data.exports.len();
        let insert = unreal_asset::types::PackageIndex::new(len as i32 + 1);
        actor.duplicate(map, export_names);
        let (path, new) = actor::Actor::new(map, insert).unwrap();
        export_names[len] = new.name.clone();
        notif.send(Notif {
            message: format!("{} duplicated", actor.name),
            kind: Warning,
        });
        match path {
            Some(ref path) => {
                let (mesh, material) = &registry.meshes[path];
                commands.spawn((
                    actor::SelectedBundle::default(),
                    MaterialMeshBundle {
                        mesh: mesh.clone_weak(),
                        material: material
                            .as_ref()
                            .map(|mat| registry.mats[mat].clone_weak())
                            .unwrap_or(consts.grid.clone_weak()),
                        transform: actor.transform(map),
                        ..default()
                    },
                    bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                    new,
                ));
            }
            None => {
                commands
                    .spawn((
                        actor::Selected,
                        MaterialMeshBundle {
                            mesh: consts.cube.clone_weak(),
                            material: consts.selected.clone_weak(),
                            transform: actor.transform(map),
                            ..default()
                        },
                        bevy::pbr::wireframe::NoWireframe,
                        new,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            consts.bounds.clone_weak(),
                            SpatialBundle {
                                visibility: Visibility::Hidden,
                                ..default()
                            },
                            bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                        ));
                    });
            }
        }
    }
}

pub fn delete(
    _: Trigger<triggers::Delete>,
    mut notif: EventWriter<Notif>,
    mut commands: Commands,
    mut map: NonSendMut<Map>,
    selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
) {
    let Some((map, ..)) = &mut map.0 else { return };
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to delete".into(),
            kind: Warning,
        });
        return;
    }
    for (entity, actor, ..) in selected.iter() {
        actor.delete(map);
        notif.send(Notif {
            message: format!("{} deleted", actor.name),
            kind: Warning,
        });
        commands.entity(entity).despawn_recursive()
    }
}

pub fn focus(
    _: Trigger<triggers::Focus>,
    mut notif: EventWriter<Notif>,
    mut focus: ResMut<Focus>,
    selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
    mut camera: Query<&mut smooth_bevy_cameras::LookTransform, With<Camera3d>>,
) {
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to focus".into(),
            kind: Warning,
        });
        return;
    }
    let (mut pos, mut sca) = selected
        .iter()
        .fold((Vec3::ZERO, Vec3::ZERO), |(pos, sca), (_, _, trans)| {
            (pos + trans.translation, sca + trans.scale)
        });
    let len = selected.iter().len() as f32;
    pos /= len;
    sca /= len;
    camera.single_mut().target = pos;
    focus.0 = Some(pos - camera.single().look_direction().unwrap_or_default() * sca.length() * 5.0)
}

pub fn approach(
    mut focus: ResMut<Focus>,
    mut camera: Query<&mut smooth_bevy_cameras::LookTransform, With<Camera3d>>,
) {
    let Some(target) = focus.0 else { return };
    let trans = &mut camera.single_mut().eye;
    *trans += target - *trans;
    if trans.distance(target) < 1.0 {
        focus.0 = None
    }
}

pub fn copy(
    _: Trigger<triggers::Copy>,
    mut notif: EventWriter<Notif>,
    mut buffer: ResMut<Buffer>,
    selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
) {
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to copy location from".into(),
            kind: Warning,
        });
        return;
    }
    buffer.0 = selected
        .iter()
        .fold(Vec3::ZERO, |pos, (_, _, trans)| pos + trans.translation)
        / selected.iter().len() as f32;
    notif.send(Notif {
        message: "location copied".into(),
        kind: Success,
    });
}

pub fn paste(
    _: Trigger<triggers::Paste>,
    mut notif: EventWriter<Notif>,
    mut map: NonSendMut<Map>,
    buffer: Res<Buffer>,
    mut selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
) {
    let Some((map, ..)) = &mut map.0 else { return };
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to paste location to".into(),
            kind: Warning,
        });
        return;
    }
    let offset = buffer.0
        - selected
            .iter()
            .fold(Vec3::ZERO, |pos, (_, _, trans)| pos + trans.translation)
            / selected.iter().len() as f32;
    for (_, actor, mut trans) in selected.iter_mut() {
        actor.add_location(map, offset);
        trans.translation += offset;
    }
    notif.send(Notif {
        message: "location pasted".into(),
        kind: Success,
    });
}

pub fn deselect(
    _: Trigger<triggers::Deselect>,
    consts: Res<Constants>,
    selected: Query<Entity, With<actor::Selected>>,
    mut cubes: Query<&mut Handle<wire::Wire>>,
    mut commands: Commands,
) {
    for entity in selected.iter() {
        match cubes.get_mut(entity) {
            Ok(mut mat) => {
                commands.entity(entity).remove::<actor::Selected>();
                *mat = consts.unselected.clone_weak();
            }
            Err(_) => {
                commands.entity(entity).remove::<actor::SelectedBundle>();
            }
        }
    }
}

pub fn fullscreen(_: Trigger<triggers::Fullscreen>, mut windows: Query<&mut Window>) {
    use bevy::window::WindowMode;
    let mut window = windows.single_mut();
    window.mode = match window.mode {
        WindowMode::Windowed => WindowMode::BorderlessFullscreen,
        _ => WindowMode::Windowed,
    };
}

pub fn hide(_: Trigger<triggers::Hide>, mut hidden: ResMut<Hidden>) {
    hidden.0 = !hidden.0
}

#[test]
fn aes() {
    let key = "0x620E8AD508F57F0E1A40BBE1929A490EDA59CA40FEFE4745D1D594F7F2C2E0CA";
    assert_eq!(
        key.trim_start_matches("0x"),
        "620E8AD508F57F0E1A40BBE1929A490EDA59CA40FEFE4745D1D594F7F2C2E0CA"
    );
    let _ = hex::decode(key.trim_start_matches("0x")).unwrap();
}

pub fn load_paks(
    _: Trigger<triggers::LoadPaks>,
    mut notif: EventWriter<Notif>,
    appdata: Res<AppData>,
    mut paks: ResMut<Paks>,
) {
    let Some(pak) = appdata.pak else { return };
    use aes::cipher::KeyInit;
    let key = match hex::decode(appdata.paks[pak].1.trim_start_matches("0x")) {
        Ok(key) if !appdata.paks[pak].1.is_empty() => aes::Aes256::new_from_slice(&key).ok(),
        Ok(_) => None,
        Err(_) => {
            notif.send(Notif {
                message: "aes key is invalid hex".into(),
                kind: Warning,
            });
            None
        }
    };
    #[cfg(target_os = "windows")]
    #[link(name = "oo2core_win64", kind = "static")]
    extern "C" {
        fn OodleLZ_Decompress(
            compBuf: *const u8,
            compBufSize: usize,
            rawBuf: *mut u8,
            rawLen: usize,
            fuzzSafe: u32,
            checkCRC: u32,
            verbosity: u32,
            decBufBase: u64,
            decBufSize: usize,
            fpCallback: u64,
            callbackUserData: u64,
            decoderMemory: *mut u8,
            decoderMemorySize: usize,
            threadPhase: u32,
        ) -> i32;
    }
    let path = &appdata.paks[pak].0;
    let Ok(files) = std::fs::read_dir(path) else {
        return;
    };
    paks.1 = path.into();
    // use std::io::Write;
    // let mut log = std::fs::File::create("paks.log").unwrap();
    paks.2 = files
        .filter_map(Result::ok)
        .map(|dir| dir.path())
        .filter_map(|path| {
            let mut pak_file = std::io::BufReader::new(std::fs::File::open(path.clone()).ok()?);
            let mut pak = repak::PakBuilder::new();
            if let Some(key) = key.as_ref() {
                pak = pak.key(key.clone());
            }
            #[cfg(target_os = "windows")]
            {
                pak = pak.oodle(|| {
                    Ok(|comp_buf, raw_buf| unsafe {
                        OodleLZ_Decompress(
                            comp_buf.as_ptr(),
                            comp_buf.len(),
                            raw_buf.as_mut_ptr(),
                            raw_buf.len(),
                            1,
                            1,
                            0,
                            0,
                            0,
                            0,
                            0,
                            std::ptr::null_mut(),
                            0,
                            3,
                        )
                    })
                });
            }
            let pak = pak.reader(&mut pak_file).ok()?;
            // let _ = writeln!(
            //     &mut log,
            //     "{path:?}\n{}\n{:?}",
            //     pak.mount_point(),
            //     pak.files()
            // );
            Some((path, pak))
        })
        .collect();
    // obtain game name
    for (_, pak) in paks.2.iter() {
        let mut split = pak.mount_point().split('/').peekable();
        while let Some((game, content)) = split.next().zip(split.peek()) {
            if game != "Engine" && content == &"Content" {
                paks.0 = game.into();
                return;
            }
        }
        for entry in pak.files() {
            let mut split = entry.split('/').take(2);
            if let Some((game, content)) = split.next().zip(split.next()) {
                if game != "Engine" && content == "Content" {
                    paks.0 = game.into();
                    return;
                }
            }
        }
    }
    if let Ok(dir) = std::fs::read_dir(path) {
        for dir in dir.filter_map(Result::ok) {
            if !dir.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let name = dir.file_name().to_string_lossy().into();
            if name == "Engine" {
                continue;
            }
            if dir.path().join("Content").exists() {
                paks.0 = name;
            }
        }
    }
}
