# <img src="assets/pot.ico" width="50" /> `stove` - an editor for cooked unreal engine 4 maps
---
**still a heavy work in progress**
---
# Credits
- [localcc](https://github.com/localcc) for their [rust rewrite](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_asset) of [UAssetAPI](https://github.com/atenfyr/UAssetAPI) and [atenfyr](https://github.com/atenfyr) for creating [UAssetAPI](https://github.com/atenfyr/UAssetAPI) in the first place
- [fedor](https://github.com/not-fl3) and [emilk](https://github.com/emilk) for their [miniquad](https://crates.io/crates/miniquad) and [egui](https://crates.io/crates/egui) crates which allowed me to build the tool how I wanted
- [LongerWarrior](https://github.com/LongerWarrior) for pointing out everything I was missing in the actor duplication code

# Roadmap (AKA unrealistic ambitions)
- [x] save and load unreal map files of any version
- [x] display a selectable list of actors
- [x] allow editing of each actor's properties
- [ ] exclude properties not in the unreal editor
- [ ] render each actor as an cube/sprite in a 3d scene
- [ ] walk around scene with an unreal-editor-style camera
- [ ] insert default values (if a value is left the same as in the blueprint then it isn't actually in the map file)
- [ ] undo and redo certain actions
- [ ] clone actors in the same map
- [ ] transfer actors from a different map
- [ ] display the mesh of an actor rather than a cube
- [ ] search functionality