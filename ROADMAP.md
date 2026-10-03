# Prototype roadmap

The offline foundation below is implemented. Gameplay milestones remain pending; advance when the acceptance checks pass in the target BeamNG installation.

## Offline foundation

- [x] Create a GE Lua extension with explicit loading and enable/disable controls.
- [x] Add uniquely named input actions without overriding default bindings.
- [x] Add mod-owned target registration, reach/visibility-result validation and interaction callbacks.
- [x] Clear focus across frames and reset targets at level exit/unload.
- [x] Add a ZIP packager, offline Lua smoke checks and an engine test checklist.
- [ ] Verify extension hooks and actions in BeamNG.

## Portable character physics

- [x] Research documented Roblox settings and historical community measurements.
- [x] Add a Rust AVBD subset for one translational body and static plane contacts.
- [x] Add configurable walking/jumping, offline numerical checks and a trajectory example.
- [x] Add a Studio motion capture script (runtime verification pending).
- [ ] Collect current R6 movement traces and calibrate acceleration, braking and air control.
- [x] Add upright capsule clearance and exact infinite-plane sweep queries.
- [x] Add an offline corridor/ceiling obstacle course with clearance assertions.
- [ ] Add finite triangle/edge capsule collision, sweep-and-slide, friction and step handling.
- [ ] Validate Rust/BeamNG transport and connect movement to avatar, camera and animations.
- [ ] Verify stock map collision and moving platforms in BeamNG.
- [ ] Add full rotational/articulated AVBD and vehicle coupling when required.

## 1. Verify the player foundation

- [ ] Record BeamNG version and test environment.
- [ ] Inspect walking controller, visual mesh, camera, and input integration.
- [ ] Add one original blocky test avatar.
- [ ] Verify walking, jumping, ground collision, camera, and vehicle entry/exit on a stock map.

Acceptance: a visible controllable character navigates the map without interfering with normal driving.

## 2. Prove avatar animation

- [ ] Demonstrate supported animation playback in BeamNG.
- [ ] Connect idle, walk, and jump clips to movement state.
- [ ] Check body orientation, feet alignment, transitions, and camera clipping.

Acceptance: the test avatar visibly changes animation with movement. Record limitations before choosing the importer format.

## 3. Prove player interaction

- [ ] Target a nearby test object from the camera.
- [ ] Highlight the target and bind one interaction action.
- [ ] Enforce interaction distance and avoid acting through walls.

Acceptance: the player can select and interact with the intended nearby object reliably.

## 4. Add minimal block placement

- [ ] Create a collision-enabled cube at runtime on a stock map.
- [ ] Add placement preview, simple grid snapping, placement, and removal.
- [ ] Reject placement overlapping the player or a vehicle.
- [ ] Verify player standing/jumping and vehicle collision after placement and removal.
- [ ] Clean up mod-created objects on level changes and unloading.

Acceptance: place a cube, jump onto it, drive into it, remove it, and verify its collision disappears.

## 5. Import one Roblox avatar

- [ ] Load a test user's R6 avatar through Roblox Studio's supported avatar API.
- [ ] Establish a reproducible export and conversion process.
- [ ] Preserve supported body appearance, textures, and accessories.
- [ ] Map the imported body to the tested animation system.
- [ ] Document missing or unsupported features and asset-use requirements.

Acceptance: one converted R6 avatar completes the movement and interaction demo on a stock map.

## 6. Generalise import

- [ ] Define a local avatar package format from the working conversion.
- [ ] Add avatar selection, validation, actionable errors, and a fallback model.
- [ ] Test several different compatible R6 avatars.
- [ ] Document install/import instructions and a compatibility matrix.

Acceptance: another user can import a compatible avatar using the documented process.

## Later, when the foundation works

R15, additional animation states, custom animation packs, layered clothing, dynamic heads, movable blocks, persistence, multiplayer, and automated online avatar retrieval.
