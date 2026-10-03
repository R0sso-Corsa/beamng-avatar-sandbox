# BeamNG integration feasibility and first-session checklist

## Native loading is unresolved

The versioned C library is reusable by hosts that permit native plugins. It is not evidence that BeamNG.drive can load it. In an [October 2022 developer response](https://www.beamng.com/threads/loading-lua-c-dll.88622/), BeamNG team member tdev explained that the mod Lua sandbox blocked loading via `ffi.load`/`package.loadlib` and treated a workaround as a security issue. This historical statement is stronger evidence than assuming that LuaJIT implies unrestricted FFI. That is not evidence that external DLL injection is blocked. Injection, Lua loading and supported native interfaces are separate mechanisms. The current installed game build has not been tested here; no supported custom-library route was established in this research.

The [official extension documentation](https://docs.beamng.com/modding/programming/extensions/) supports GE Lua extensions and explicit extension loading. This is our current integration surface. Do not ship the Rust library in the mod ZIP expecting it to execute. A companion Rust process could be investigated if a supported local communication route exists, but that transport, latency, packaging and collision data access are all unresolved. No sidecar connection is implemented.

## Read-only runtime inventory

Install the packaged mod on a BeamNG-capable machine, open the console in the GE Lua context, then run:

```lua
extensions.load("avatarSandbox_diagnostics")
extensions.avatarSandbox_diagnostics.printReport()
```

Record the game version manually and save the `avatarSandboxDiagnostics` log lines. The inventory records Lua/JIT version and the presence of extension, scene-tree, vector, camera-position and static-raycast symbols. It does not invoke those APIs, load native code, change settings, collect paths or open network connections. Symbol presence only identifies candidates for further testing; it does not verify signatures or behavior. The extension returns a report table for local inspection.

The offline smoke check exercises collection and logging in a generic Lua runtime, with a raycast stub that errors if called. It does not validate BeamNG itself.

## First playable integration gates

1. Confirm extension loading and input actions on the installed build.
2. Inspect shipped GE Lua source for supported camera, avatar rendering and collision APIs, recording the version and actual call signatures. Do not infer triangle extraction from raycast availability: capsule sweeps need finite geometry or a separately validated host collision adapter.
3. Establish a supported Rust transport/loading route. If unavailable, keep the Rust core as the portable reference and assess a limited Lua controller separately; porting the entire solver is not automatically justified.
4. Build an isolated flat-floor spawn/walk/jump scene before stock-map movement. Connect controls, fixed-step timing, visual transform and camera; establish pause, reset and scene unload behavior.
5. Verify stock terrain, walls, ceilings, stairs and dynamic objects. Only then test vehicle interaction and BeamMP transport.

All runtime gates remain open. No avatar movement, native loading, map extraction or rendering is implemented by this diagnostic extension.


## Prepared input and timing driver

`avatarSandbox_driver` is a separate GE extension with no native backend attached by default. Load with `extensions.load("avatarSandbox_driver")`. A future adapter calls `attach(function(input, dt) ... return true end)` and owns the physics instance, collision data and rendering. This callback must synchronously commit one atomic step; errors or any return other than `true` detach it. Mission end and extension unload detach it, clearing inputs and timing.

Five manually bindable actions feed forward/backward/left/right/jump values. The driver normalizes diagonals and rotates input by an explicitly supplied horizontal heading (`setHeading`), where zero is forward +Y and right +X. Camera orientation mapping remains pending. The action definitions and `onUpdate(dtReal, dtSim)` hook arguments still need verification in BeamNG.

Only simulation delta advances physics at 240 Hz; real delta is ignored. A zero simulation delta discards residual time and clears controls, requiring fresh input after pause. At most 16 steps run per update; excess whole-step backlog is dropped and counted in `status().droppedSeconds`, while `alpha` exposes the residual interpolation fraction. A quick jump press/release is retained until a step executes. A mock adapter verifies timing, diagonal speed, heading, quick taps, pause, backlog, errors and mission cleanup offline. This driver does not move the player until an actual adapter is attached.
