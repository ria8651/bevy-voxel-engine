# Data Layout

## Voxel Data

Each voxel has two bytes, the first is the material id and the second is some flags.

```
01000101 01101111
         ││││├──╯
automata─╯││││
portal────╯│││
animation──╯││
collision───╯│
other────────╯
```

If the automata flag is set then the rest of the data byte is automata data. If the portal flag is set then the material becomes a portal id. If the animation flag is set the voxel will be destroyed at the beginning of the next frame. If the collision flag is set the voxel will be used for collision detection.
