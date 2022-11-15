# <img src="assets/pot.ico" width="50" /> `stove` - an editor for cooked unreal engine 4 maps
*<h1 align="center">still an extremely heavy work in progress</h1>*
# Credits
- [localcc](https://github.com/localcc) for their [rust rewrite](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_asset) of [UAssetAPI](https://github.com/atenfyr/UAssetAPI) and [atenfyr](https://github.com/atenfyr) for creating [UAssetAPI](https://github.com/atenfyr/UAssetAPI) in the first place
- [fedor](https://github.com/not-fl3) and [emilk](https://github.com/emilk) for their minimal yet easy-to-use [miniquad](https://crates.io/crates/miniquad) and [egui](https://crates.io/crates/egui) crates
- [LongerWarrior](https://github.com/LongerWarrior) for pointing out everything I was missing in the actor duplication code
# Roadmap (from basic functionality to unrealistic ambitions)
- [x] save and open unreal map files of any version
- [x] display a selectable list of actors
- [x] allow editing of actor transforms
- [x] render each actor as an cube/sprite in a 3d scene
- [x] walk around the scene with an unreal-editor-style camera
- [ ] organise actor components in a satisfying way
- [ ] allow editing of each actor's properties
- [ ] duplicate actors in the same map
- [ ] transplant actors from a different map
- [ ] undo and redo certain actions
- [ ] insert default values (many properties left as default are cut from the map file)
- [ ] searching functionality
- [ ] discord RPC (show your internet friends what you're doing)
- [ ] display the mesh/sprite of an actor rather than a cube (would require loading pak/folder of assets)
- [ ] move actors with a gizmo/keybinds rather than directly editing the transform property
