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

## Offline conversion progress

`tools/rbx_convert` uses rbx_binary/rbx_xml to decode binary Roblox source assets to XML. These dependencies are isolated to the offline tool and locked in its Cargo.lock. `tools/convert_r6_animation.py` reads decoded XML and produces JSON plus typed Rust source for sampler verification. Run the decoder for binary files; the jump asset is already XML. Then run:

```sh
python3 tools/convert_r6_animation.py assets/private/r6_animations/walk.rbxmx assets/private/r6_animations/walk.json
python3 tests/r6_animation_smoke.py
```

All six local originals have been converted and each track compiled/sampled with the Rust runtime. Pose matrices become normalized quaternions; source axes, units, hierarchy and timestamps remain unchanged. Only unit-weight linear easing is accepted; omitted easing uses the legacy linear default, which still needs comparison against Studio. Other styles are rejected rather than silently approximated.

Walk has 22 keyframes, seven paths and a 0.666667-second duration. Idle clips contain both `/HumanoidRootPart/Torso/...` and `/Torso/...` paths. The converter now normalizes torso-only paths to the root-prefixed paths and joins their non-overlapping keys. Conflicting keys at the same time are rejected. Studio comparison is still pending. Source clips do not provide Motor6D C0/C1 offsets. The Studio exporter remains needed to capture the source rest frames, then map them to the supplied Blender skeleton. Converted data remains private/ignored and is not bundled in the mod. Character animation state selection and skinned rendering remain pending.

## Retargeting API

Rust `animation::retarget` takes target local bind pose, original Motor6D C1, a pure rotation mapping source child-part axes into target bone axes, and sampled pose delta. All translations must use the same units. It computes the rest-relative delta as `C1 * pose * inverse(C1)` and conjugates that delta through the alignment before composing with the target bind pose. Identity animation preserves the target bind; tests also cover pivot-induced translation and rigid transform inversion. C0 cancels in this rest-relative formulation, but both original offsets remain useful for checking the source skeleton.

This is a rigid-bone API: it does not apply nonuniform bone scale or skinning. Actual C1 and child-axis alignment values have not been guessed from the downloaded skeleton. Export source offsets in Studio, compare reference poses and then supply the alignment. All six normalized clips now contain seven tracks (root plus six visible body parts). The target end bones retain their rest transforms. Live pose application remains pending.

## Studio capture, 4 October 2026

Roblox Studio 0.741.19.7411056 successfully created the default R6 rig and retrieved all six clips through KeyframeSequenceProvider. `studio_joint_offsets.json` records all six measured Motor6D C0/C1 offsets; `studio_validation.json` records rest part transforms/sizes, clip accessibility/keyframe counts/looping/easing and current Humanoid defaults. All clips and newly created Pose instances use Linear easing; the default easing direction is In. These are direct Studio observations, not inferred settings.

A walk keyframe at 0.15873000025749207 seconds was applied to a local reference rig using the source Motor6D relation and saved in `assets/private/r6-reference.rbxl`. This is a static reference pose, not Animator playback and not validation of the downloaded skinned rig. Ground clearance/feet placement and target skeleton axes still require checking. Full source clips and the walk pose record stay private.

## Animator validation

`tools/roblox/validate_r6_playback.lua` was executed in Studio's Command Bar. All six AnimationTracks loaded and were evaluated through `Animator:StepAnimations` at five timestamps each. Their Motor6D transforms were captured locally in `assets/private/r6_animator_capture.json`; this is real edit-mode Animator evaluation, not normal gameplay playback. The reference was restored to rest pose and lifted to 0.05 studs above the spawn platform for inspection. This does not verify walking foot contact throughout the clip.

`tests/studio_animation_parity.py` compares captured transforms to the compiled Rust sampler. Five clips pass thresholds of 0.0001 studs and 0.001 radians across 30 joint samples each. Climb fails the strict rotation threshold: maximum difference 0.008181 radians (about 0.47 degrees), despite negligible translation differences. Normalized quaternion lerp was also checked and did not improve the discrepancy, so the sampler was not changed without evidence. `studio_parity_report.json` records the discrepancy; the strict audit currently exits nonzero. It is intentionally separate from the passing general offline suite.

Next: capture more climb timestamps, inspect legacy rotation interpolation, then validate source-to-target bone alignment/skinning. No exact Roblox rotation parity or downloaded-rig deformation acceptance is claimed.
