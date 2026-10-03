# Noob R6-style test character

Original geometry inspired by the classic yellow, blue and green noob appearance. No Roblox assets or textures were downloaded.

Open `noob_r6.obj` with `noob_r6.mtl` beside it in Blender or another OBJ-compatible editor. Import with object/group splitting enabled to retain Head, Torso, LeftArm, RightArm, LeftLeg and RightLeg. Face geometry belongs to Head. The model uses flat colours and a simple original smile.

Coordinates: metres, Z up, facing -Y, feet at Z=0. Total height is 1.5 m. Left/right are the character's own sides. This deliberately simple block head can be rounded later.

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
