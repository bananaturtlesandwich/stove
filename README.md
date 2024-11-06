# <img src="assets/pot.ico" width="50" /> `stove` - an editor for cooked unreal engine maps

**<h1 align="center">[get the latest alpha build!](https://github.com/bananaturtlesandwich/stove/releases)</h1>**

<h1 align="center"><img width=900 src=https://github.com/user-attachments/assets/a480a151-c6c1-4d27-a78a-af0bdccdd754></h1>

# features
- visualise maps as they would be in-game
- edit actor properties and transforms
- duplicate and delete actors
- transplant actors from other maps (including your own!)

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
### convenience
- [ ] undo and redo
- [x] actor deletion
- [x] move actors in the viewport
- [x] searching
### advanced functionality
- [ ] insert default values (properties left as default are cut from the map)
- [ ] delete excess exports left after actor removal
- [x] duplicate and transplant all actor types (not sure why some don't work)
### aesthetic
- [x] load assets from pak folders
- [x] retrieve and display static meshes
- [ ] retrieve and display skeletal meshes
- [ ] get meshes for all types of actor
- [x] retrieve and display textures
- [ ] parse materials properly
- [x] discord RPC (show your internet friends what you're doing)
</details>

# credits

- [atenfyr](https://github.com/atenfyr) for creating the extensive [UAssetAPI](https://github.com/atenfyr/UAssetAPI) which made this project possible ❤️
- [localcc](https://github.com/localcc) for rewriting it as [unreal_asset](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_asset), allowing me to program this in [rust <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Food/Crab.png" width="20" />](https://www.rust-lang.org/)
- [LongerWarrior](https://github.com/LongerWarrior) for pointing out everything I was missing in the actor duplication code
