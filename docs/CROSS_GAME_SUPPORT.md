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

The C ABI supports default or custom profiles and static mesh stepping. Obtain `AvatarProfileV1` defaults with `avatar_default_profile_v1`, adjust fields and create with `avatar_create_with_profile_v1`. Invalid profiles return a zero handle. Existing instances are not modified; recreate to change profile settings. The frozen v1 layout matches the header. `metres_per_stud` is metadata: changing it does not automatically rescale other fields or uploaded geometry. The translating-platform adapter remains Rust-only. ABI v1 does not expose per-game gravity changes, articulated AVBD, vehicle forces or networking. All instances serialize through one mutex; benchmark before using many avatars. Allocation failure is not a recoverable ABI error. Calls must finish before unloading the library.

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

The check actually links against the built library and asserts walking, grounding, jumping and stale-handle rejection. It passed on the current macOS host, alongside 15 Rust unit tests. Windows and Linux builds have not been tested; build for the same OS and architecture as the target game. Shipping builds should arrange platform-appropriate library loading paths rather than relying on the example's local rpath.

## Game adapter responsibilities

Each game still needs a supported native plugin/loading route, input mapping, mesh extraction, unit/axis conversion, fixed-step scheduling and avatar rendering. For a source-controlled Rust game, using the Rust crate directly is simpler. For C/C++ hosts, the header/library boundary provides reuse. Managed-language hosts can bind this ABI, but their calling convention and struct layout must match the header. No native BeamNG loading path or other engine binding has been verified here.

Next: expose platform state across the ABI, validate a small host scene on another OS, and establish BeamNG's supported integration route before connecting gameplay. The same simulation can be shared across games; assets, host collision APIs, camera and animations remain game-specific.

BeamNG-specific feasibility and the read-only runtime inventory are documented in [BeamNG integration](BEAMNG_INTEGRATION.md). Historical developer guidance indicates custom DLL loading is sandboxed; the C ABI does not by itself resolve that restriction.
