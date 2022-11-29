# <img src="assets/pot.ico" width="50" /> `stove` - an editor for cooked unreal engine 4 maps
*<h1 align="center">still an extremely heavy work in progress</h1>*
# Credits
- [localcc](https://github.com/localcc) for their [rust rewrite](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_asset) of [UAssetAPI](https://github.com/atenfyr/UAssetAPI) and [atenfyr](https://github.com/atenfyr) for creating [UAssetAPI](https://github.com/atenfyr/UAssetAPI) in the first place
- [fedor](https://github.com/not-fl3) and [emilk](https://github.com/emilk) for their minimal yet easy-to-use [miniquad](https://crates.io/crates/miniquad) and [egui](https://crates.io/crates/egui) crates
- [LongerWarrior](https://github.com/LongerWarrior) for pointing out everything I was missing in the actor duplication code
# Roadmap
### Basic functionality
- [x] save and open unreal map files of any version
- [x] display a selectable list of actors
- [x] allow editing all of an actor's transforms
- [x] render each actor as a cube/sprite in a 3d scene
- [x] walk around the scene with an unreal-editor-style camera
- [x] duplicate actors in the same map
- [x] transplant actors from a different map
- [ ] edit the properties of actors and their components
- [ ] insert default values (properties left as default are cut from the map)
### Convenience
- [ ] undo and redo any action
- [ ] actor deletion
- [ ] move actors in the viewport instead of in the properties
- [ ] multiple selection (requires above to be useful)
- [ ] searching functionality
### Low Priority
- [ ] display the mesh/sprite of an actor and their components rather than a cube
- [ ] discord RPC (show your internet friends what you're doing)