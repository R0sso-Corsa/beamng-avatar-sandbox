## Rust driver and catalogue update

`physics/src/driver.rs` owns 240 Hz input batching, diagonal normalization, heading rotation, jump edges, pause input release and bounded catch-up. `physics/src/catalogue.rs` owns approved avatar selection, revisioned snapshots, validation and missing-pack fallback. Lua delegates to injected Rust adapters; it no longer implements those rules. The driver now exposes C input control/heading/reset/batch APIs; the catalogue now also exposes versioned C configuration/snapshot APIs. A native binding, authenticated transport and engine rendering remain required; no live BeamNG or BeamMP compatibility is claimed.

# Cross-game support: C ABI v1

The dependency-free Rust core now builds as `rlib`, `cdylib` and `staticlib`. A game plugin can call the same simulation through `physics/include/avatar_physics.h`; it does not need to embed Rust or copy the physics into its game engine. This interface is a foundation, not a working plugin for any game.

## Contract

- Check `avatar_abi_version() == 1` before use. Create returns an opaque, process-local handle (zero means failure); destroy each instance when unloading or restarting.
- Use metres, seconds and Z-up. Position is the capsule centre. Convert the host's scale, axes and origin at its adapter boundary, including every collision vertex and input direction. The default capsule is 1.53 m tall with a 0.30 m radius.
- Movement is a horizontal direction/analog vector in simulation coordinates, with diagonal magnitude normalized. Jump is 0 or 1; holding it does not repeatedly jump. Sample controls at host frame rate, pass the current input to each fixed step.
- Accumulate elapsed simulation time and call `avatar_step` once per 1/240 second. A host should cap its catch-up work and explicitly choose how to handle excessive backlog. Pause should not accumulate catch-up time. Interpolate previous/current snapshots for rendering.
- `avatar_set_mesh` copies and validates triangles and builds the cached index. Upload static geometry on scene changes, not every frame. IDs must be unique per instance. A failed upload preserves the previous mesh; success clears contact history. An empty mesh means free fall.
- Call `avatar_get_state` for position, velocity and grounded state. Drive the visible avatar, camera and animation in the host; this library performs no rendering or host-engine calls.
- Return codes: 0 success, -1 unknown handle, -2 invalid input or failed physics step, -3 unavailable registry. Failed steps preserve state. Invalid/stale handles are rejected. Pointer validity is the caller's responsibility: nonnull pointers alone do not prove memory is readable/writable. Never pass untrusted raw addresses.

The C ABI supports default or custom profiles and static mesh stepping. Obtain `AvatarProfileV1` defaults with `avatar_default_profile_v1`, adjust fields and create with `avatar_create_with_profile_v1`. Invalid profiles return a zero handle. Existing instances are not modified; recreate to change profile settings. The frozen v1 layout matches the header. `metres_per_stud` is metadata: changing it does not automatically rescale other fields or uploaded geometry. The translating-platform prototype is also exposed through `avatar_step_platform_v1`; see the contract below. ABI v1 does not expose per-game gravity changes, articulated AVBD, vehicle forces or networking. All instances serialize through one mutex; benchmark before using many avatars. Allocation failure is not a recoverable ABI error. Calls must finish before unloading the library.

## Runnable C host check

Build from the repository root:

```sh
cargo build --release --manifest-path physics/Cargo.toml
```

On macOS/Linux, with a C compiler and Rust installed:

```sh
cc physics/examples/c_host.c -Iphysics/include -Lphysics/target/release \
  -lavatar_physics -Wl,-rpath,"$PWD/physics/target/release" -o /tmp/avatar-c-host
/tmp/avatar-c-host
```

The check actually links against the built library and asserts walking, grounding, jumping and stale-handle rejection. It passed on the current macOS host, alongside the Rust unit tests. Windows and Linux builds have not been tested; build for the same OS and architecture as the target game. Shipping builds should arrange platform-appropriate library loading paths rather than relying on the example's local rpath.

## Game adapter responsibilities

Each game still needs a supported native plugin/loading route, input mapping, mesh extraction, unit/axis conversion, fixed-step scheduling and avatar rendering. For a source-controlled Rust game, using the Rust crate directly is simpler. For C/C++ hosts, the header/library boundary provides reuse. Managed-language hosts can bind this ABI, but their calling convention and struct layout must match the header. No native BeamNG loading path or other engine binding has been verified here.

Next: validate a small host scene on another OS, and establish BeamNG's supported integration route before connecting gameplay. The same simulation can be shared across games; assets, host collision APIs, camera and animations remain game-specific.

BeamNG-specific feasibility and the read-only runtime inventory are documented in [BeamNG integration](BEAMNG_INTEGRATION.md). Historical developer guidance indicates custom DLL loading is sandboxed; the C ABI does not by itself resolve that restriction.


## Translating platform call

`avatar_step_platform_v1` replaces the ordinary step for a frame containing one translating platform. It uses the uploaded mesh as static geometry, and a caller-owned triangle array describing the platform at its previous pose. Supply its displacement over exactly 1/240 second. Keep IDs unique across the static and platform sets. A null platform pointer is permitted only with zero count; pointer validity remains the host's responsibility.

Establish grounding at the initial pose with a zero-displacement platform step before carrying the avatar. After each successful call, advance the host's platform vertices and render pose by the same displacement. On failure both character and uploaded static mesh are unchanged, and the host must stop or resolve the platform motion. Never upload the same platform as static geometry too. The platform input is copied for that call and is not stored for subsequent ordinary steps.

Grounded velocity excludes carrier motion; departing avatars inherit the carrier velocity for subsequent steps. This remains the prescribed translation prototype: no rotating platforms, reaction forces or continuous collision with moving geometry. Fast platforms can cross an unsupported avatar between endpoint poses. Repeated attachment transitions still need validation. Platform stepping scans raw geometry and does not use the cached static-mesh broad phase, so keep scenes small until benchmarked.

The external C check now exercises simultaneous horizontal/upward carry, jump inheritance, null input, nonfinite displacement, conflicting IDs, rollback and destroyed handles. It passes on macOS; BeamNG and Windows/Linux runtime integration are still unverified.


## Rust camera and building core

The authoritative camera and building-plan behavior now lives in `camera.rs` and `building.rs`. `AvatarCameraPoseV1` and camera active/look/zoom/pose calls use the existing character handle. Cameras start inactive. The host provides focus/eye positions and a safe obstruction fraction (-1 means clear), consumes position/forward and first-person flags, and restores its own camera when leaving avatar mode. `first_person` also tells the host to hide the local head as appropriate. No engine orientation or visibility calls happen in Rust.

Building plans default to disabled. `avatar_building_enabled_v1`, `avatar_building_edit_v1`, `avatar_building_get_v1` and `avatar_building_clear_v1` expose place/move/resize/clone/remove on conservative bounds. The header defines operation codes, pointer requirements and output layout. Clear disables editing and removes plans without recycling IDs. Hosts supply fresh player/vehicle bounds and implement object creation/collision/rendering; none of these calls spawn objects. The Rust preview API is not separately exposed through C yet.

Lua camera and building modules now require injected backend tables; their wrappers forward calls instead of duplicating behavior. A future supported BeamNG transport must map those backend methods to the Rust/C APIs. No direct FFI loader is bundled. Existing Lua standalone camera/building prototypes have been replaced, so calls without a backend fail explicitly. Physics, gears, projectiles, camera and building operations are reusable Rust code. Input scheduling and avatar catalogue validation now live in Rust; Lua forwards to host adapters. Mode choice and host takeover/restoration belong to each game adapter. Python remains asset/development tooling.

All 24 Rust tests pass, and the external C host verifies camera zoom/first person/obstruction and building opt-in/place/move/query/clear alongside earlier physics checks. Lua mock tests verify forwarding and lifecycle behavior. Windows/Linux builds and actual game adapters remain unverified; a shared API cannot make an engine support native integration where none is available.
