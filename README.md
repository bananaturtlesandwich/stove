# <img src="assets/pot.ico" width="50" /> `stove` - an editor for cooked unreal engine 4 maps

*<h1 align="center">still an extremely heavy work in progress</h1>*

<h1 align="center"><img width="700" src="https://user-images.githubusercontent.com/71292624/208417568-840bb37d-57db-4273-84e9-b069a78964e1.png"></h1>

with stove you can:
- visualise actors relative to each other
- edit actor properties
- see your transform edits as they happen
- duplicate and delete actors
- transplant existing and custom actors from other maps

<a href="https://www.youtube.com/watch?v=gnl3OSftqno"><img width="700" src="https://user-images.githubusercontent.com/71292624/208414853-0a17badc-a4f0-4ddb-a157-677fe2fc88f4.png"></a>

<details>
<summary><h1>roadmap</h1></summary>

### basic functionality
- [x] save and open unreal map files of any version
- [x] display a selectable list of actors
- [x] allow editing all of an actor's transforms
- [x] render each actor as a cube/sprite in a 3d scene
- [x] walk around the scene with an unreal-editor-style camera
- [x] duplicate actors in the same map
- [x] transplant actors from a different map
- [x] edit the properties of actors and their components
- [ ] insert default values (properties left as default are cut from the map)
### convenience
- [ ] undo and redo any action
- [x] actor deletion
- [ ] can move actors in the viewport instead of just in the properties
- [ ] multiple selection (requires above to be useful)
- [ ] searching
### low priority
- [ ] display the mesh/sprite of an actor and their components rather than a cube
- [x] discord RPC (show your internet friends what you're doing)
</details>

# credits

- [localcc](https://github.com/localcc) for their [rust rewrite](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_asset) of [UAssetAPI](https://github.com/atenfyr/UAssetAPI) and [atenfyr](https://github.com/atenfyr) for creating [UAssetAPI](https://github.com/atenfyr/UAssetAPI) in the first place
- [fedor](https://github.com/not-fl3) and [emilk](https://github.com/emilk) for their minimal yet easy-to-use [miniquad](https://crates.io/crates/miniquad) and [egui](https://crates.io/crates/egui) crates
- [LongerWarrior](https://github.com/LongerWarrior) for pointing out everything I was missing in the actor duplication code
