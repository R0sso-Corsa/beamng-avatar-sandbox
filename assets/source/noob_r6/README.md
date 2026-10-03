# Noob R6-style test character

Original body geometry inspired by the classic noob appearance, with the downloaded Sketchfab head by sdfgh13s. See ATTRIBUTION.md. The original Roblox face texture remains local and user-supplied.

Open `noob_r6.obj` with `noob_r6.mtl` beside it in Blender or another OBJ-compatible editor. Import with object/group splitting enabled to retain Head, Torso, LeftArm, RightArm, LeftLeg and RightLeg. Face geometry belongs to Head. The model uses flat colours and a simple original smile.

Coordinates: metres, Z up, facing -Y, feet at Z=0. Total height is 1.53 m. Left/right are the character's own sides. The head uses the supplied Sketchfab mesh, uniformly scaled to 0.33 m high and approximately 0.3294 m wide. Its 846 triangles and source normals are retained. It is centred on the neck and recoloured yellow.

Suggested joint positions (X, Y, Z) in metres:

| Joint | Position |
| --- | --- |
| Root | 0, 0, 0.9 |
| Neck | 0, 0, 1.2 |
| Left shoulder | 0.3, 0, 1.2 |
| Right shoulder | -0.3, 0, 1.2 |
| Left hip | 0.15, 0, 0.6 |
| Right hip | -0.15, 0, 0.6 |

The OBJ is an editable source mesh, not a rigged or animated model and not yet a BeamNG walking-mode part. Armature setup, animation clips, BeamNG mesh/material conversion and controller attachment are pending. Source assets are intentionally outside `mod/` and excluded from the test ZIP.

Regenerate with `python3 tools/create_noob_model.py`. `preview.svg` is a front-view design reference.

## Original Roblox face texture

To replace the generated smile with your own local Roblox default texture:

```sh
python3 tools/create_noob_model.py --face-texture /path/to/Roblox/content/textures/face.png
```

This creates a separate variant in `assets/private/noob_r6/` with the PNG, OBJ, MTL and preview. A curved UV-mapped transparent panel replaces the smile geometry and remains grouped with Head. Its front-view proportions match the PNG's native width/height ratio, with its longest side set to 0.30 m independently of the head width. UVs use the central 80% of the PNG on each axis to reduce transparent padding without changing its aspect ratio. The texture file is copied unchanged. Keep the three model files together when importing. The material references both colour and alpha; check transparency after import because OBJ readers differ in their support for alpha maps. BeamNG material conversion remains pending.

The private variant and its embedded preview are ignored by Git. The shared source model retains the original generated smile as a fallback. Roblox's texture is user-supplied and is not redistributed by this repository.

Body ratios follow the supplied classic noob reference: torso 0.6 × 0.3 × 0.6 m; each arm and leg 0.3 × 0.3 × 0.6 m. The face display size includes the original PNG’s transparent margins; the visible smile is smaller than the full panel. The decal is projected onto the downloaded mesh using a grid of surface samples; transparent corners may extend past its rounded outline.
