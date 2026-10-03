# R6 skeleton mapping

Source: [Roblox R6 Rigged by p0x38](https://sketchfab.com/3d-models/roblox-r6-rigged-fd308c3375a94116b5958085ee6d77b8), listed as CC BY-SA 4.0. Credit p0x38 and retain that licence for redistributed modified model assets. The downloaded model is not committed here.

`bone_mapping.json` records the inspected GLB's six body joints, parent indices and local bind transforms. End bones are auxiliary, not additional R6 body parts. Preserve the GLB's ancestor transforms and inverse bind matrices; the local transforms are not world-space pivots. No clips are embedded. Names/hierarchy pass inspection; skin weights, scale and deformation still need a visual test.

Regenerate with `python3 tools/inspect_r6_rig.py /path/to/roblox_r6_rigged.glb --output assets/source/r6_rig/bone_mapping.json`.

## Export original clips

In Roblox Studio, select an R6 rig and accessible local KeyframeSequence objects. Paste `tools/roblox/export_r6_animation.lua` into the Command Bar. Alternatively fill its animationIds with IDs from the rig's actual Animate script. Copy the resulting ServerStorage StringValue's Value to `assets/private/r6_animations.json` (ignored by Git). The exporter does not assume that published animation assets are accessible.

The JSON retains source joint C0/C1 offsets, hierarchical poses, timestamps, loop/priority, weight and easing. The source relation is `childWorld = parentWorld * C0 * poseTransform * inverse(C1)`. Retarget joint-local deltas through matching rest frames; simply applying Roblox pose matrices to arbitrary Blender bones can rotate limbs incorrectly. Convert metres using our 0.30 m/stud scale and choose an explicit Y-up to Z-up basis/facing conversion before applying to the existing mesh.

Next: import an actual clip, verify rest-pose and arm/leg rotation against Studio, then implement Rust interpolation and animation-state blending. This mapping/exporter is not animation playback or a BeamNG-renderable rig.

## Local source clips and Rust sampler

Six asset responses were downloaded through Roblox asset delivery to `assets/private/r6_animations/`; their local manifest records IDs, sizes and SHA-256 hashes. These are source files, not converted clips, and are not redistributed. Import them into Studio, verify each KeyframeSequence and use the exporter above.

`physics/src/animation.rs` supplies typed local pose tracks with shortest-path quaternion interpolation and pose blending. Only linear and hold interpolation are supported at present; other Roblox easing must be baked or implemented before claiming matching playback. The sampler does not parse RBXM/JSON, retarget bind frames, choose character states or apply skinning. No clip has been applied to the supplied mesh yet.
