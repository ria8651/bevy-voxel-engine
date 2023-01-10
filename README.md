# Bevy Voxel Engine

## About

Bevy Voxel Engine is a voxel renderer for the [bevy](https://bevyengine.org/) game engine. 

## Features

* Real time voxel rendering on low end gpus
* Ray traced lighting on high end gpus
* Loading of [magica voxel](https://ephtracy.github.io/index.html?page=mv_main) .vox files
* Real time voxelization of textured meshes
* Basic ray-cast based gpu physics engine
* Real time cellular automata (not user customizable yet)
* Portals!

<img width="45%" alt="ray-traced-rendering" src="https://user-images.githubusercontent.com/66388895/211429077-fb4434f5-7a95-4f79-afa1-d13857560470.png"> <img width="45%" alt="voxel-rendering" src="https://user-images.githubusercontent.com/66388895/211426758-bb3ea28d-f7ab-4d3c-a74b-a27c62301166.png">
<img width="45%" alt="voxelization" src="https://user-images.githubusercontent.com/66388895/211429206-c4c25c31-93b0-42ec-a341-48115e35db85.gif"> <img width="45%" alt="portals" src="https://user-images.githubusercontent.com/66388895/211430628-eb242645-becb-4426-bff1-b52b6de93fbd.gif">
<img width="45%" alt="sand" src="https://user-images.githubusercontent.com/66388895/211436215-181f2a9e-0e77-41ab-9b82-f698080c1d56.gif">

## Try it yourself

Clone the repo and run
```
cargo run --release --example features
```
for the portal demo or

```
cargo run --release --example sand
```
for the sand demo.

## Licence

I haven't picked one yet, just create an issue if you want this released under a particular one.
