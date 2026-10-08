# Character physics research and Rust prototype

Research date: 3 October 2026. This is a behavioural recreation, not a reconstruction of Roblox's proprietary engine. Community numbers below are historical reports, not measurements we have reproduced. BeamNG integration is untested.

Update, 6 October: [native R6 calibration](NATIVE_CALIBRATION.md) now includes local
Studio measurements, independent Rust replays, an opt-in feedback motor and
camera spring. The historical prototype defaults below remain available.
The selected flat-ground and airborne fits do not establish complete parity.

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
| Mass, contact friction, step height, slope behaviour | No reliable universal Humanoid constants established in this research | 60 kg, 0.30 m sphere radius, 45° support threshold are provisional; passive static/dynamic friction 0.8/0.6 are project tuning; 0.30 m stair limit is a project setting |

The documented Humanoid properties describe an interface, not the entire controller. Do not equate JumpPower with an SI force or infer an internal force formula from its name. [1]

The naive prediction with launch speed 50 and gravity 196.2 is `v²/(2g) = 6.37105` studs. Applying the historical 1.06 factor predicts 7.15851 studs; the later 7.26633 report is about 1.5% higher. Possible explanations include sampling, controller state transitions and engine changes; none is established here. The 2020 test sampled RenderStepped, so it is not an internal substep trace. [5][6]

Our BDF1 trajectory has finite-step numerical damping. At 240 Hz the launch approximation produces roughly 7.05 studs of rise, not perfect parity with either community result. Measure launch and apex separately before changing gravity or adding a fitted multiplier. Preserve original settings, rig, timestamps and client version with every capture.

Do not change BeamNG's global gravity to match Roblox. The avatar core applies its own gravity; vehicles retain the host world's simulation. How the avatar exchanges forces with vehicles remains an integration task.

## AVBD implementation scope

[AVBD, Giles/Diaz/Yuksel (2025)](https://graphics.cs.utah.edu/research/projects/avbd/) adds augmented-Lagrangian dual updates to vertex/body block solves. The implemented subset uses the paper's inertial target, bounded contact forces, Hessian rescaling, stabilization, dual/penalty updates and warm starting (Eqs. 2, 11–14, 18–19). A 3×3 LDLᵀ solve updates one translational body; its plane constraints have zero geometric Hessian. [8]

This is **not the complete AVBD rigid-body engine**. The AVBD solver omits rotation, articulated joints, deformables, Coulomb friction, GPU scheduling and adaptive initialization. Static triangle collision is now handled by a separate mesh adapter. It uses an inertial-target initial guess. The gameplay motor and jump launch sit outside the contact solver. Outward normals give nonpositive duals, the opposite sign convention to the paper's contact example. Initial error stabilization applies only to penetration, avoiding attraction to separated surfaces. Numerical parameters are project tuning, not claims about Roblox internals.

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
cargo run --manifest-path physics/Cargo.toml --example mesh_course > mesh_course.csv
cargo run --manifest-path physics/Cargo.toml --example stairs_course > stairs_course.csv
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

The offline obstacle course is a four-second corridor/low-ceiling scenario with clearance assertions and CSV output. It uses infinite half-spaces; finite obstacles and stairs now have offline prototypes; full map navigation remains pending.

## Finite mesh prototype

`physics/src/mesh.rs` adds two-sided triangle queries using capsule-segment to filled-triangle distance, including face, edge and vertex regions. Conservative advancement uses the convex distance function’s supporting tangent and projected closing rate to bound translation, with a 0.1 mm skin and 256-iteration budget. Sweep-and-slide retains contact normals at corners and allows five blocking contacts per move. Exhaustion or initial penetration returns an error, not an unchecked movement.

`Character::step_mesh` creates local tangent contacts for the AVBD step and guards the resulting translation using the mesh sweep. This final movement guard is kinematic, not an AVBD constraint-force solve. Position-derived velocity and mesh support are updated after sliding. Queries are transactional: errors preserve the previous character state. The API uses stable unique triangle IDs and rejects degenerate/nonfinite triangles. Raw slice queries scan every triangle. `StaticMesh` now validates geometry once and caches a median-split bounding-volume tree for nearby candidate selection.

`mesh_course` walks into a freestanding box, moves sideways and passes around it on a finite floor, with assertions and CSV output. Fourteen unit tests cover the combined core. This does not establish full map navigation: moving surfaces, coupled solver friction and broader seam/slope validation remain pending. Tangent planes are local approximations for one fixed step; large meshes and adversarial geometry have not been validated. No BeamNG tests have been run.

## Stairs, slope support and indexed geometry

The mesh adapter tries a swept up/forward/down path when grounded movement hits a steep obstruction. `Profile.step_height` defaults to 0.30 m; zero disables this optional stair rule. Landing must touch a walkable triangle and improve forward progress. Adjacent riser/tread triangles can share an edge, so landing support is checked across nearby triangles rather than relying on whichever triangle wins an equal-time hit. Ceiling sweeps remain mandatory. The discrete stair lift does not become vertical launch velocity.

Walkable support uses the triangle face slope and an upward capsule contact normal, including a rounded tread edge. A grounded downward probe up to the configured stair limit maintains support on descending treads, including rounded capsule edges. Larger drops fall normally; jumps and airborne avatars are not snapped. Mesh AVBD tangent contacts cover walkable support; steep walls and ceilings are handled by the kinematic swept guard. This remains a gameplay controller around an AVBD subset, not a fully dynamic articulated body.

`StaticMesh::new` validates unique IDs and builds a cached AABB tree. Use `Character::step_static_mesh` for indexed geometry; its conservative query envelope includes the capsule, current velocity, motor/jump travel, gravity, stair probes and ground snap. Rebuild the immutable mesh when geometry changes. Candidate triangles retain source ordering so equal-hit selection does not change just because indexing is enabled. Whole BeamNG map performance has not been measured.

Offline checks now include three 15 cm risers, a 20 cm step, rejection of a 65 cm obstacle and a low ceiling, continuous grounding across a 20-degree ramp seam, and no jump support on a 60-degree face. An index regression selects one candidate from 1,000 separated triangles and matches the raw simulation. These checks establish those fixtures only; arbitrary stair dimensions, concave seams, coupled friction and moving geometry still need work. Roblox movement calibration and BeamNG integration remain untested.


### Bounded overlap recovery and passive friction

Shallow mesh overlaps are corrected along the deepest contact normal before stepping, with a cumulative 10 cm correction budget and 32-iteration limit. Failed recovery leaves the character unchanged. Successful recovery clears velocity and cached contacts, so correction does not launch the avatar. This is intended for small spawn/geometry errors; trapped or deeply embedded starts may fail.

Passive ground friction estimates the gravity/support impulse before the contact solve. Static friction holds a resting avatar; dynamic friction reduces tangential sliding speed. The movement motor overrides this friction while walking. Coefficients are provisional, uniform character settings, not measured Roblox constants or per-material BeamNG values. This is a controller approximation, not coupled AVBD Coulomb friction rows.

New regression fixtures cover shallow floor/wall recovery, rejection without mutation beyond the recovery budget, a grounded 15 cm descent versus a falling 65 cm drop, passive holding on a 20-degree ramp, sliding slowdown on a flat floor and unchanged airborne motion. These results are offline only.

The three-riser example now runs for two seconds (480 fixed steps) to include settling with the downward ground probe. It still asserts completion, final tread height and grounding; its traversal timing is not calibrated to Roblox.


### Translating support-platform prototype

`Character::step_translating_platform` takes static triangles, one platform's previous-pose triangles and its displacement over one fixed step. Hosts must advance those platform vertices by the same displacement after success and keep triangle IDs unique across both sets. Start at rest on the platform and establish grounding with a mesh step first. Grounded character velocity is relative to its carrier; render displacement includes carrier motion. On jumping or walking off, the departure step adds carrier velocity (`displacement / DT`) to airborne velocity for subsequent integration. No force reaction is applied to the platform.

Carry is swept against static obstacles; a blocked carry returns an error with the original character unchanged. Hosts must stop or resolve the platform movement when that happens. Invalid inputs and duplicate IDs also fail without mutation. An unsupported avatar overlapping the platform's final pose is rejected rather than pushed out.

This is a translating carrier prototype, not general dynamic collision: platform motion is prescribed, rotation is unsupported, and a fast platform crossing an airborne avatar between endpoint poses can be missed. Repeated landing/attachment transitions and mixed overlapping supports need further validation. The adapter scans raw triangle slices and does not update the static BVH. It is not ready for moving BeamNG vehicles.

The offline regression checks one second of simultaneous horizontal/upward carry, jump velocity inheritance without ground snap, walking off a finite deck, blocked carry rollback, nonfinite displacement and duplicate IDs. Fourteen unit tests pass; no BeamNG moving-platform or Roblox comparison test has been run.
