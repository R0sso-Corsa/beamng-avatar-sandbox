# Noob R6-style test character

Original geometry inspired by the classic yellow, blue and green noob appearance. No Roblox assets or textures were downloaded.

Open `noob_r6.obj` with `noob_r6.mtl` beside it in Blender or another OBJ-compatible editor. Import with object/group splitting enabled to retain Head, Torso, LeftArm, RightArm, LeftLeg and RightLeg. Face geometry belongs to Head. The model uses flat colours and a simple original smile.

Coordinates: metres, Z up, facing -Y, feet at Z=0. Total height is 1.53 m. Left/right are the character's own sides. The head is an original classic-style cylinder with bevelled rims, 0.36 m in diameter and 0.33 m tall; it is not an extracted Roblox mesh.

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

This creates a separate variant in `assets/private/noob_r6/` with the PNG, OBJ, MTL and preview. A curved UV-mapped transparent panel replaces the smile geometry and remains grouped with Head. Its front-view proportions match the PNG's native width/height ratio, with its longest side set to 0.35 m independently of the head width. The texture file is copied unchanged. Keep the three model files together when importing. The material references both colour and alpha; check transparency after import because OBJ readers differ in their support for alpha maps. BeamNG material conversion remains pending.

The private variant and its embedded preview are ignored by Git. The shared source model retains the original generated smile as a fallback. Roblox's texture is user-supplied and is not redistributed by this repository.

Body ratios follow the supplied classic noob reference: torso 0.6 × 0.3 × 0.6 m; each arm and leg 0.3 × 0.3 × 0.6 m. The face display size includes the original PNG’s transparent margins; the visible smile is smaller than the full panel. The panel is intentionally slightly taller than the head, with transparent margins above and below.
