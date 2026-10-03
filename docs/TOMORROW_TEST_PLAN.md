# BeamNG handoff and first test session

This is a prototype readiness package, not a playable character release. The install ZIP contains Lua/UI only. No native loader, rigged avatar renderer or live game adapter is bundled. The Rust library must be built for Windows and connected using a mechanism verified on the installed game.

## Prepare on the Windows machine

1. Copy this repository and the generated mod ZIP. Keep source clips and downloaded rig local; they are not in the Git source archive or install ZIP.
2. Record BeamNG version, OS and architecture. Use a stock map and test normal driving/walking first.
3. In an x64 Visual Studio Developer PowerShell with Rust's MSVC toolchain installed, run `./tools/build_windows.ps1`. This builds a DLL and runs an external C host. It does not inject or load anything into BeamNG. This script is prepared but has not been executed on Windows.
4. Install one copy of `dist/beamng-avatar-sandbox.zip` through the user mods directory. Do not install an unpacked duplicate.
5. Run the GE console capability inventory before attempting native integration:

```lua
extensions.load('avatarSandbox_diagnostics')
dump(avatarSandbox_diagnostics.collect())
extensions.load('avatarSandbox_main')
dump(avatarSandbox_main.status())
```

Inventory symbol presence is not proof that an API works. Save the report and console errors. Bind mod actions only to unused keys; add the Avatar Sandbox Controls UI app if it appears in the app selector.

## Runtime gates, in order

- Confirm Lua extensions/UI load, toggle mod enablement, and keep stock gameplay functional. Roblox mode without a character adapter must not silently take over control.
- Verify a supported native-loading method, ABI version 1 and matching 64-bit architecture. Test create/state/destroy using a minimal host connection before running callbacks every frame. Do not assume GE Lua can load the DLL directly.
- Capture actual world collision geometry or validated engine collision queries. The current Rust character steps against supplied triangle meshes; it cannot collide with a map it has not received.
- Implement a host adapter for character entry/exit, input ownership, camera/geometry/rendering. Lua `driver.attach` now takes a backend table with `reset`, `setControl`, `setHeading`, `advance`, `status`; the old single-step function is no longer accepted.
- Driver C batching returns at most 16 input records, not stepped character states. Execute each through the character stepping API, commit atomically, reset/release inputs on failure. Use simulation delta, pause with dt=0, clear on mode exit. Driver heading uses +Y-forward/+X-right Z-up; the source mesh faces -Y, so apply an explicit visual-facing conversion.
- Spawn one visible rig before enabling the player controller. Apply bind pose first, then a single walk clip. Preserve inverse bind matrices and auxiliary end bones. Confirm feet, scale, rotation, shoulder pivots and skin weights.
- Connect animation selection from `animation_state::Animator`: supply confirmed grounded/climbing state, one-shot jumped event and actual planar speed in metres/second. On selection changes retain the last rendered pose and blend over the suggested duration. This remains a Rust API, with no C animation binding yet.
- Apply camera poses and obstruction sweep results; hide local head in first person. Restore the native camera/input/vehicle state on exit or failure.
- Test interaction using a real raycast and registered targets. Then test a single host-created collision-enabled cube; preview/planning alone does not spawn engine geometry.
- Finally test unload, map change, pause, input focus loss and repeated mode switches. Verify no leftover callbacks, objects, held input or camera takeover.

## Animation preparation independent of BeamNG

The six local originals are decoded and normalized to seven pose tracks each. `tools/roblox/export_r6_animation.lua` still needs a Studio run to capture original Motor6D C0/C1 offsets. Supply verified child-axis alignments for the downloaded rig; no offsets are inferred from arbitrary bone orientations. Rust handles sampling and rigid retarget math, but full posed/skinned rendering and visual Studio comparison are pending.

Animation state defaults are prototype choices: 0.3-second jump grace, speed-scaled walk/climb, 0.1-second transitions and 0.3-second fall transition. They have offline tests, not a claim of exact Roblox playback. Source clip looping remains separate from state selection. Climb requires a host detector; airborne motion alone must never imply climbing.

## Offline evidence

Run `python tools/validate_offline.py` with Cargo, C compiler, Node and Python+lupa available. Add `--private-assets` when local converted clips are present. Current macOS result: 24 Rust tests, Lua facades/interaction/telemetry, UI checks, source-package/model checks, external C ABI host and all six converted clip sampling checks pass. The workflow under `.github/workflows/offline.yml` is prepared for Windows/Linux/macOS; remote CI has not run yet.

## Deferred until the basic player works

Gear equip/use/cooldown/reset commands now have C transport; projectile effects still need host collision/damage/rendering. Catalogue validation is Rust-only and needs native bindings plus authenticated BeamMP events. Dynamic geometry, rotating platforms, articulated AVBD, broad map benchmarks and network authority are not solved by the install ZIP. Avoid building multiplayer or avatar selection around an unverified renderer.

Record each gate as pass/fail with game version, logs, reproduction steps and a screenshot. Do not mark roadmap gameplay milestones complete solely from offline checks.

## Additional native interfaces

The C header now includes rigid pose blend/retarget operations and gear equip, activation, cooldown advance and reset. The external macOS C host verifies success, invalid pose input, rocket command flags, cooldown rejection/recovery and building-tools opt-in/removal. These are commands and transforms only; they do not load clips, render a rig, execute damage, or provide multiplayer authority. Catalogue, clip upload/sampling and animation-state C transport remain pending.
