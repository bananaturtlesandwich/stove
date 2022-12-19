# <img src="assets/pot.ico" width="50" /> `stove` - an editor for cooked unreal engine 4 maps

*<h1 align="center">still an extremely heavy work in progress</h1>*

<h1 align="center"><img width="700" src="https://user-images.githubusercontent.com/71292624/208417568-840bb37d-57db-4273-84e9-b069a78964e1.png"></h1>

<details>
<summary><h1>use cases</h1></summary>

<details>
<summary><h2>general map edits</h2></summary>

stove is already a versatile tool because you can:
- visualise actors relative to each other
- see your transform edits as they happen
- duplicate, delete and transplant actors
- edit the vast majority of actor properties
</details>

<details>
<summary><h2>custom actor spawning</h2></summary>

stove allows transplanting (`ctrl + T`) of actors from other maps
- this includes maps you have cooked yourself
- this includes actors you have made yourself

therefore you can add your own actors to the map *provided you package them with the mod*
<a href="https://www.youtube.com/watch?v=gnl3OSftqno"><img width="700" src="https://user-images.githubusercontent.com/71292624/208414853-0a17badc-a4f0-4ddb-a157-677fe2fc88f4.png"></a>
</details>

</details>

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
- [ ] actor deletion
- [ ] can move actors in the viewport instead of in the properties
- [ ] multiple selection (requires above to be useful)
- [ ] searching functionality
### low priority
- [ ] display the mesh/sprite of an actor and their components rather than a cube
- [x] discord RPC (show your internet friends what you're doing)
</details>

<details>
<summary><h1>credits</h1></summary>

- [localcc](https://github.com/localcc) for their [rust rewrite](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_asset) of [UAssetAPI](https://github.com/atenfyr/UAssetAPI) and [atenfyr](https://github.com/atenfyr) for creating [UAssetAPI](https://github.com/atenfyr/UAssetAPI) in the first place
- [fedor](https://github.com/not-fl3) and [emilk](https://github.com/emilk) for their minimal yet easy-to-use [miniquad](https://crates.io/crates/miniquad) and [egui](https://crates.io/crates/egui) crates
- [LongerWarrior](https://github.com/LongerWarrior) for pointing out everything I was missing in the actor duplication code
</details>
