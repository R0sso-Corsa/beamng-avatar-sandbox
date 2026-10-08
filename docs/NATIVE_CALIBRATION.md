# R6 native reference and Rust calibration

Measured on macOS in Studio 0.741.19.7411056, 6 October 2026. These are
local native Humanoid observations and independent Rust replays. They do not
establish complete Roblox or BeamNG parity.

## Reproduce

Run `tools/roblox/capture_native_reference.server.lua` in the **server** command
bar of a disposable Studio play session. It creates an API-generated R6 model,
assigns server ownership, captures 17 trials, removes its temporary objects,
and retains the result in `_G.NativeReferenceResult`. Retrieve one trial at a
time with `HttpService:JSONEncode(_G.NativeReferenceResult.trials[index])`;
large all-at-once tool/Output returns can truncate. Stop play when finished.
The recording includes the actual Humanoid settings; the API-created model's
MaxSlopeAngle was 89, whereas the earlier lab clone used 45.

The committed fixture contains measurements only: no account identity, avatar
assets, installed player scripts, binary offsets or decompiled functions.
Full local raw recordings and static-analysis evidence remain outside this repo.

```sh
cargo build --release --manifest-path physics/Cargo.toml
python3 tools/compare_native_motion.py
python3 tests/native_reference_smoke.py
```

Inputs are scheduled at PreSimulation, outputs at PostSimulation. Captures are
frame-level samples, not an internal 240 Hz trace. The comparison retains the
observed first-frame command latency explicitly. Training and subsequent
validation captures are separate.

## Measured results

| Check | Result and scope |
| --- | --- |
| Walking | Inputs 0, 0.5 and 1 settle at 0, 8 and 16 studs/s at WalkSpeed16. Independent runs also cover WalkSpeed8 and 32. |
| Velocity feedback | Flat traces fit a capped 150 Hz response at the Rust 240 Hz step. |
| Surface cap | Floor friction 0.05/0.5/1, weight100, gives fitted caps 129.725/1231.165/2454.987 studs/s². These are this rig/surface fixture, not universal coefficients. |
| Rust ground replay | Velocity RMSE below 0.001 studs/s on the three material captures and their repeat runs. Default provisional motor errors were above 4 studs/s. |
| High-speed braking | WalkSpeed32 full input has 0.522 studs/s RMSE in the independent run. Some frame intervals contain a different effective movement advance; unresolved timing remains visible. |
| JumpPower50 | Native root rises 7.26859 studs. The calibrated Rust launch matches apex within 0.00001 studs and airborne velocity within 0.001 studs/s on the recorded trials. |
| Other jumps | Power25 also matches the checked airborne trajectory. Power53 has a one-substep-sized velocity difference and a different Jumping-state duration. Height-mode recordings differ between trials. These are pending, not parity checks. |
| Landing | Full vertical-trace errors remain substantial around contact/state transitions. The airborne fit does not validate landing. |
| Camera spring | Independent 4.5 Hz critically damped spring matches both captured clear zoom transitions within 0.00001 studs maximum distance error. |
| Camera wheel | Native zoom-in snaps below one stud to 0.5. From 0.5, successive outward wheel steps settle at 1.75 and 3.625 studs; an inward step returns to 1.75. |
| Obstruction | Tested opaque wall at four studs limits native camera distance to about 3.6. Transparency0.8 and CanCollide=false allow return to 12.5. Near-plane/corner behavior still belongs to the host query. |
| Turning | AutoRotate=false keeps heading; true approaches requested 45/90/180° directions. Native angular response is recorded; Rust still lacks the corresponding rotational controller. |
| Slopes | With MaxSlopeAngle45, 44.9° and 45° ramps climbed; 45.1° and 50° slid during settling. Running/FloorMaterial alone does not mean walkable support. |
| Steps/ceilings/seams | Earlier two-stud step traversed; subsequent 2.5/3/3.25/3.5-stud obstacles blocked. Low ceilings limit/suppress jumps. Tested gaps 0/0.1/0.5/2 studs were crossed; the two-stud gap caused a temporary drop. |
| Supports | Translating platform carries the player. At rotation0.5 rad/s and radius5, velocity matches the local tangent within 0.009 studs/s component RMSE after settling. |
| Moving jump | First Jumping sample retains support velocity5 studs/s; airborne zero-input braking then reduces it. |
| Blocked carry | Wall begins above the platform. Platform continues at5 studs/s while character stops at the wall; the earlier platform-wall collision trial was rejected. |

The native camera place loads the actual PlayerModule. Installed camera-feature
flag names and their queried values are recorded in fixture metadata. In the
dynamic-subject wheel fixture, local head transparency did not follow zoom,
so that capture validates zoom only. Head/accessory visibility needs a separate
host implementation and acceptance check.

## Use the calibrated options

Existing defaults and frozen C struct layouts remain unchanged. Opt in:

1. Obtain `avatar_studio_r6_profile_v1`, create with that profile, then set
   `avatar_motor_response_v1(handle, 150)`.
2. The preset uses the recorded default Plastic cap, provisional air cap, and a
   one-gravity-step launch offset for the JumpPower50 fixture. Its capsule, mass,
   slope limit and stair rule remain project choices.
3. Use `avatar_motor_caps_v1` to change ground/air caps without resetting motion
   when the host selects a new surface. Material selection/mixing is host-fed;
   the core does not inspect a game's material database. Feedback mode supplies
   braking itself and does not add the legacy passive-friction pre-step.
4. Call `avatar_camera_classic_v1(handle, metres_per_stud)`, activate the camera,
   and supply orbit/zoom input. Positive wheel steps zoom out.
5. Call `avatar_camera_advance_v1` once per host frame, dt in [0,1], with a
   safe boom length in metres or -1 for clear. The host length must already
   include shape/near-plane clearance. Obstruction clamps immediately; clearing
   a wall recovers with the spring. Query the pose with obstruction=-1 after
   supplying that safe length, to avoid applying clearance twice.

Classic starts at 12.5 studs, looks downward15°, limits pitch to ±80°, and caps
the project's zoom at50 studs. First-person radius settles at0.5 studs about the
host-supplied eye position. Absolute distance input is available through
`avatar_camera_distance_v1`. Pose queries do not advance time.

The host must apply transforms, hide local body parts, filter the local avatar
out of camera queries, and restore native controls/camera on leaving character
mode. These APIs alone do not take control of a game.

## Next acceptance gates

Continue resolving jump-state timing, high-speed braking, airborne input latency,
landing, rotational control, contact-aware friction and moving/rotating geometry.
The recorded contact-selection path uses multiple floor probes and averaged
results; it is not the prototype capsule's swept-stair rule. Matching selected
flat traces does not validate arbitrary terrain or rebuild the whole engine.

BeamNG loading, stock-map collision/rendering/camera takeover and vehicle
interaction require the target game. Windows/Linux C-host execution and
multiplayer ownership remain separate environment checks.
