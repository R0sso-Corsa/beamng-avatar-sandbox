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
- [x] Add finite triangle face/edge/vertex queries and conservative capsule sweeps.
- [x] Add sweep-and-slide and an offline freestanding-box mesh course.
- [x] Add prototype swept stair climbing with ceiling/height checks.
- [x] Verify walkable ramp seams and steep-surface support rejection offline.
- [x] Add cached static-mesh bounds indexing and unfiltered parity checks.
- [x] Add bounded shallow-overlap recovery, grounded stair descent and passive ground friction with offline regression fixtures.
- [x] Add a translating support-platform prototype with carry, jump inheritance and blocked-carry regression checks.
- [ ] Broaden stair/slope/seam coverage and implement coupled friction, rotating platforms and continuous moving-surface collision.
- [ ] Benchmark full map geometry and integrate dynamic geometry updates.
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

- [x] Add offline one-metre grid planning, player/vehicle AABB overlap checks, plan removal and lifecycle cleanup.

- [ ] Create a collision-enabled cube at runtime on a stock map.
- [ ] Add placement preview, simple grid snapping, placement, and removal.
- [ ] Reject placement overlapping the player or a vehicle.
- [ ] Verify player standing/jumping and vehicle collision after placement and removal.
- [ ] Clean up mod-created objects on level changes and unloading.

Acceptance: place a cube, jump onto it, drive into it, remove it, and verify its collision disappears.

## 5. Import one Roblox avatar

- [x] Define and validate a local R6 source package with checksums and import limits (engine conversion pending).

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


## Cross-game portability

- [x] Build Rust, shared and static libraries with a versioned C ABI.
- [x] Add managed handles, transactional static mesh upload, stepping and state snapshots.
- [x] Run a separate C host smoke check on macOS.
- [x] Expose validated configurable profiles through the C ABI and verify custom speed in the external C host.
- [x] Expose the translating-platform prototype through the C ABI and verify carry/jump and validation in the external C host.
- [ ] Validate Windows/Linux builds and a real game adapter.
- [ ] Verify BeamNG native loading and host collision/rendering integration.

See [cross-game support](docs/CROSS_GAME_SUPPORT.md) for the contract and runnable host check.

- [x] Research BeamNG native-loading constraints and add a read-only GE capability inventory.
- [ ] Run the inventory on an installed BeamNG build and validate host APIs.

See [BeamNG integration](docs/BEAMNG_INTEGRATION.md) for runtime gates and console commands.

- [x] Prepare mock-tested GE input/fixed-step driver with pause, backlog and lifecycle handling.
- [ ] Verify action bindings, simulation delta and camera heading on BeamNG.

See [block placement](docs/BLOCK_PLACEMENT.md) for the offline planner and remaining engine responsibilities.

## Portable gears

- [x] Add the seven Doomspire wiki gears as Rust equip/use commands with per-gear cooldowns.
- [x] Add default-disabled building-tools opt-in and Lua placement-plan access.
- [x] Expose gear equip/use/cooldown/reset commands through the C ABI.
- [ ] Implement and validate projectile, damage, destruction, animation and visual adapters.
- [ ] Verify gear behavior and building tools in BeamNG.

See [gear scope and defaults](docs/GEARS.md).

- [x] Base optional building tools on F3X-style editing; add offline move, symmetric resize and clone with overlap rollback.
- [ ] Add selection UI, rotation, appearance tools and undo/redo, then validate engine edits.

See [avatar import and multiplayer delivery](docs/AVATAR_IMPORT.md) for the source format and proposed server-approved pack workflow.

- [x] Add avatar-ID catalogues and mock-tested server-approved appearance snapshots, join/change/leave synchronization and missing-pack resolution.
- [ ] Connect avatar selection UI, engine asset loading/rendering and live BeamMP events.

- [x] Add offline projectile trajectories, host-fed impact/bounce events, lifetimes and one-shot bomb timers.
- [ ] Connect projectile collision shapes, damage/destruction and visual effects to host adapters.


## Optional character mode and Roblox-style camera

- [x] Keep mod enablement separate from character choice; default to normal BeamNG and allow Roblox/BeamNG switching while enabled.
- [x] Add mock-tested adapter enter/leave transitions and return to BeamNG on disable/mission exit.
- [x] Add Classic-inspired camera orbit, pitch limits, zoom-to-first-person and host-fed obstruction handling offline.
- [ ] Add a visible character-mode selector and bind camera mouse/orbit/zoom controls in BeamNG.
- [ ] Apply camera poses, hide the local head in first person and restore native camera/input/vehicle state on leaving avatar mode.
- [ ] Verify stock gameplay remains available with the mod enabled and Roblox character mode switched off.

Requirement: users can choose the Roblox character or the typical BeamNG experience without uninstalling or disabling the mod. Camera and avatar takeover only occur in Roblox character mode; they must be restored when leaving it.

## Rust portability consolidation

- [x] Move camera orbit/zoom/first-person behavior into Rust and expose the C camera interface.
- [x] Move block planning, overlap validation, move/resize/clone/remove into Rust and expose C edit/query operations.
- [x] Replace Lua camera/building implementations with backend facades; verify with Rust, external C and Lua mock checks.
- [x] Move reusable input scheduling and avatar catalogue validation into Rust; replace Lua copies with adapter facades.
- [ ] Connect Rust driver/catalogue APIs through native host bindings and authenticated multiplayer transport.
- [x] Expose building preview in the C API.
- [x] Expose gear transport in the C API.
- [ ] Supply supported game adapters for camera, controls, geometry and rendering.

## Debugging and on-screen controls

- [x] Add local bounded telemetry, error counts and copyable UI debug reports.
- [x] Add a BeamNG UI app for mod/mode/building/interaction and held movement controls with backend status.
- [x] Verify telemetry routing, command whitelist, input release and UI teardown offline.
- [ ] Verify the app in BeamNG, including layout, live status and focus/pause behavior.
- [ ] Route future Rust/engine adapter failures into telemetry and add camera bindings.

See [telemetry and controls](docs/TELEMETRY_AND_CONTROLS.md).

## Original R6 animation import

- [x] Inspect supplied Sketchfab skeleton and record six-part bind mapping.
- [x] Download six original source clips locally; decode and convert to portable pose tracks.
- [x] Normalize legacy torso-only paths and reject duplicate/conflicting keys.
- [x] Compile/sample all six clips with Rust quaternion interpolation and rigid retarget tests.
- [x] Add Rust idle/walk/jump/fall/climb selection, speed scaling, transition hints and lifecycle reset.
- [ ] Capture source Motor6D offsets in Studio and verify target bone-axis alignments.
- [ ] Apply clips to the rig, verify skin weights/deformation, and blend actual rendered poses.
- [ ] Expose animation pose/state transport to the host and validate live playback.

See [rig workflow](assets/source/r6_rig/README.md). Clips are converted locally, not applied to the mesh.

## Offline handoff preparation

- [x] Expose driver controls, heading, reset and fixed-step input batches through C ABI.
- [x] Expose read-only building preview through C ABI and verify it does not allocate parts.
- [x] Add a unified offline validation runner and rebuild the Lua/UI install ZIP.
- [x] Prepare Windows native build/C-host check script and cross-platform CI configuration.
- [ ] Execute Windows/Linux validation and remote CI; configuration alone is not validation.
- [ ] Publish recent local changes to GitHub (local Git HTTPS credentials unavailable).

See [tomorrow's runtime gates](docs/TOMORROW_TEST_PLAN.md).

- [x] Expose rigid pose blending/retargeting through C ABI and validate from the external C host.
- [x] Expose clip upload/sampling, animation state and catalogue snapshots through native transport.

## Native animation and catalogue completion

- [x] Expose clip track upload/sample/reset and animation state selection through the C ABI.
- [x] Expose approved/installed catalogues, server changes/snapshots and client apply/get/reset through C ABI.
- [x] Verify actual converted clip keys and looping/intermediate samples via the shared library.

See [the complete offline checklist](docs/OFFLINE_CHECKLIST.md) for remaining local, Studio and Windows work.

- [x] Capture authentic R6 joint offsets, part rest transforms and current Humanoid defaults in Studio 0.741.19.7411056.
- [x] Verify all six animation assets are accessible and use Linear easing in Studio; save static walk reference pose locally.
- [ ] Validate actual playback, feet clearance and downloaded rig deformation before accepting the retarget.

- [x] Evaluate six real AnimationTracks in Studio edit mode and compare 180 joint samples against Rust.
- [ ] Resolve climb rotation mismatch (0.008181 radians maximum); five other clips pass strict comparison.
