# BeamNG integration feasibility and first-session checklist

## Native loading is unresolved

The versioned C library is reusable by hosts that permit native plugins. It is not evidence that BeamNG.drive can load it. In an [October 2022 developer response](https://www.beamng.com/threads/loading-lua-c-dll.88622/), BeamNG team member tdev explained that sandboxing prevents custom C DLL loading and treated a workaround as a security issue. This historical statement is stronger evidence than assuming that LuaJIT implies unrestricted FFI. The current installed game build has not been tested here; no supported custom-library route was established in this research.

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
