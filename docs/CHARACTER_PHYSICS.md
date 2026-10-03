# Character physics research and Rust prototype

Research date: 3 October 2026. This is a behavioural recreation, not a reconstruction of Roblox's proprietary engine. Community numbers below are historical reports, not measurements we have reproduced. BeamNG integration is untested.

## Evidence and working values

| Quantity | Evidence | Prototype treatment |
| --- | --- | --- |
| Walk speed | Humanoid default is 16 studs/s [1] | 4.8 m/s at the current mesh scale |
| Jump settings | JumpPower defaults to 50; JumpHeight to 7.2; UseJumpPower selects which setting applies [1] | Power-based launch approximation; height mode not implemented |
| Gravity | Workspace default 196.2 studs/s² [2] | 58.86 m/s² for avatar simulation only |
| Physical scale | Roblox standard is 0.28 m/stud, but internally units are interpreted consistently by the experience [3] | 0.30 m/stud matches our existing torso/limbs. This is an explicit project choice |
| Solver rate | Roblox documents 240 Hz and adaptive 60/120/240 Hz islands [4] | Fixed 240 Hz for repeatable offline runs; no adaptive islands |
| Jump correction | XAXA reported a 1.06 factor in April 2019, including a JumpPower 100 comparison [5] | Configurable launch speed initially `50 * 1.06 * scale` |
| Measured jump rise | General_Scripter reported R6 root Y=3 to Y=10.26632976532 at JumpPower 50 in February 2020 [6] | 7.26633 studs is a comparison target, not an exact regression requirement |
| Launch velocity | A June 2024 post reported approximately 53.1, with discussion about whether JumpPower denotes velocity [7] | Weak corroboration of ~53 studs/s; insufficient method/version detail |
| Acceleration, braking, air control | Forum search largely found custom scripts, rather than repeatable native Humanoid measurements | 80 m/s² grounded and 20 m/s² airborne are provisional design values |
| Mass, contact friction, step height, slope behaviour | No reliable universal Humanoid constants established in this research | 60 kg, 0.30 m sphere radius, 45° support threshold are provisional; friction and stepping pending |

The documented Humanoid properties describe an interface, not the entire controller. Do not equate JumpPower with an SI force or infer an internal force formula from its name. [1]

The naive prediction with launch speed 50 and gravity 196.2 is `v²/(2g) = 6.37105` studs. Applying the historical 1.06 factor predicts 7.15851 studs; the later 7.26633 report is about 1.5% higher. Possible explanations include sampling, controller state transitions and engine changes; none is established here. The 2020 test sampled RenderStepped, so it is not an internal substep trace. [5][6]

Our BDF1 trajectory has finite-step numerical damping. At 240 Hz the launch approximation produces roughly 7.05 studs of rise, not perfect parity with either community result. Measure launch and apex separately before changing gravity or adding a fitted multiplier. Preserve original settings, rig, timestamps and client version with every capture.

Do not change BeamNG's global gravity to match Roblox. The avatar core applies its own gravity; vehicles retain the host world's simulation. How the avatar exchanges forces with vehicles remains an integration task.

## AVBD implementation scope

[AVBD, Giles/Diaz/Yuksel (2025)](https://graphics.cs.utah.edu/research/projects/avbd/) adds augmented-Lagrangian dual updates to vertex/body block solves. The implemented subset uses the paper's inertial target, bounded contact forces, Hessian rescaling, stabilization, dual/penalty updates and warm starting (Eqs. 2, 11–14, 18–19). A 3×3 LDLᵀ solve updates one translational body; its plane constraints have zero geometric Hessian. [8]

This is **not the complete AVBD rigid-body engine**. It omits rotation, articulated joints, deformables, Coulomb friction, collision detection, GPU scheduling and adaptive initialization. It uses an inertial-target initial guess. The gameplay motor and jump launch sit outside the contact solver. Outward normals give nonpositive duals, the opposite sign convention to the paper's contact example. Initial error stabilization applies only to penetration, avoiding attraction to separated surfaces. Numerical parameters are project tuning, not claims about Roblox internals.

An independent implementation was written from the equations; no reference source is bundled. The [author's reference solver](https://github.com/savant117/avbd-demo3d/blob/main/source/solver.cpp) was inspected for sequencing and conventions. [9]

## Relationship to the Skate/MW2 mashup

The [Skate Rust project](https://github.com/SK8-ENGINE/skate-3-rust-engine) is a Rust/Bevy recreation based on prior reverse-engineering work and explicitly describes parity as unfinished. Its assets are supplied separately. [10]

The [actual mashup](https://github.com/chasmlol/2010-rust-rewrite-mashup) combines it with IW4L, a Rust rewrite of MW2. That provides source-level control of the host runtime. It does not establish a drop-in Rust plugin interface for the retail BeamNG executable. [11]

We follow the reusable simulation approach: Rust accepts movement input and host-supplied surfaces and returns position/velocity/grounded state. Rendering, animation, camera, map collision queries and native loading belong to the host adapter. No Skate code was copied or merged. No full Roblox engine rewrite is required for this character prototype.

## Run and inspect

From repository root:

```sh
cargo test --manifest-path physics/Cargo.toml
cargo run --manifest-path physics/Cargo.toml --example trajectory > trajectory.csv
cargo run --manifest-path physics/Cargo.toml --example obstacle_course > course.csv
```

`physics/src/lib.rs` is a dependency-free Rust library. `Character::step` advances exactly 1/240 second. Input is world-space XY; Z is up. Analog magnitude is preserved and diagonal speed is capped. Jump triggers on a press edge while supported, with no automatic repeat, coyote time or jump buffering.

The collision proxy is an upright capsule, 1.53 m tall with 0.30 m radius. Position is its centre; the mesh foot origin is `position.z - height/2`. Plane support uses `radius + abs(normal.z) * (height/2 - radius)`, so caps and sides have different clearance. `sweep_plane` returns the exact first crossing fraction against an infinite plane; the contact solver includes predicted crossings. This is not finite triangle/edge collision or a complete movement sweep-and-slide system.

Planes represent infinite supporting half-spaces, not complete map triangles. A host must select local, relevant surfaces and retain stable IDs only for unchanged geometry. Supplying every face of a level as an infinite plane would constrain the avatar incorrectly. Moving platforms are unsupported. Character teleport/restart should create a fresh instance to clear contact history.

The tests check free fall, diagonal speed, landing, wall blocking, ballistic jump rise, held-jump behaviour, capsule support/sweeps, low-ceiling contact, removed-surface cleanup and invalid-input rejection. The trajectory example exercises walking and a jump without a game installation. These are offline numerical checks, not Roblox/BeamNG parity tests.

## Measurement and next integration gates

`tools/roblox/capture_motion.client.lua` is an opt-in Studio LocalScript for a controlled R6 test place. It captures 10 seconds of PostSimulation samples and prints JSON metadata plus batches of 30 samples to avoid Output truncation. Set the trial label and manually record Workspace's stepping method in the script; that property is NotScriptable. It uses normal player controls: record separate trials for idle, walk start/stop, diagonal movement, jump from rest, running jump and airborne direction changes. Repeat on flat ground, slopes, steps and moving platforms; do not mix these conditions in one trial. PostSimulation is frame-level sampling, not guaranteed 240 Hz sampling. This script has not been run in Studio here.

Before tuning, compare acceleration curves, launch speed, apex displacement and time, flight duration, stopping distance and landing error. Obtain multiple repeats at several render rates, then fit profiles with recorded uncertainty. Pin evidence to the tested engine version.

Before BeamNG playability: verify a supported Rust loading/IPC mechanism on the target installation, map coordinate/scale conversion, finite capsule sweeps, stairs/slopes, contact lifetime, camera/animation state output and safe enable/disable ownership of movement. Avoid simultaneous native walking and Rust control. Add vehicle/platform force coupling only after static-world movement works. Other games require their own adapters; compatibility is not automatic.

## Sources

1. [Roblox Humanoid reference](https://create.roblox.com/docs/reference/engine/classes/Humanoid)
2. [Roblox Workspace.Gravity reference](https://create.roblox.com/docs/reference/engine/classes/Workspace#Gravity)
3. [Roblox physical units](https://create.roblox.com/docs/physics/units)
4. [Roblox adaptive timestepping](https://create.roblox.com/docs/physics/adaptive-timestepping)
5. [World Panel Settings discussion, April 2019, posts 28–30](https://devforum.roblox.com/t/world-panel-settings-now-available/265335?page=2)
6. [Calculating maximum player jump distance, February 2020](https://devforum.roblox.com/t/calculating-maximum-player-jump-distance/455105)
7. [What exactly is JumpPower?, June 2024](https://devforum.roblox.com/t/what-exactly-is-jumppower/3029994)
8. [AVBD full paper](https://graphics.cs.utah.edu/research/projects/avbd/Augmented_VBD-SIGGRAPH25.pdf), especially §§3.1–3.7 and Algorithm 1
9. [Author's AVBD 3D solver reference](https://github.com/savant117/avbd-demo3d)
10. [SK8-ENGINE project](https://github.com/SK8-ENGINE/skate-3-rust-engine), inspected main `7ae67f269c024ed0b1aa945701e04fa5bf419848`
11. [2010 Rust Rewrite Mashup](https://github.com/chasmlol/2010-rust-rewrite-mashup)

The offline obstacle course is a four-second corridor/low-ceiling scenario with clearance assertions and CSV output. It uses infinite half-spaces; finite obstacles, stairs and map-mesh navigation remain pending.
