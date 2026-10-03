# Work that does not require BeamNG

This is the current offline backlog, not a promise that every possible future feature is necessary before the first game test. Checked items have implementation and offline evidence; unchecked items remain work.

## Completed and validated on this machine

- [x] Rust capsule movement, mesh collision, camera and block planning foundations.
- [x] Fixed-step input scheduling with C batch/control/reset APIs.
- [x] Gear command/cooldown/opt-in/reset C APIs.
- [x] Decode six original R6 clips and normalize legacy pose paths.
- [x] Rust pose interpolation, blending and rigid retargeting.
- [x] Idle/walk/jump/fall/climb state selection with transition hints.
- [x] Native track upload, sampling, animation selection and reset interfaces.
- [x] Native avatar catalogue configuration, server join/select/leave, snapshots, client apply/get/reset.
- [x] Transactional rejection of invalid track replacement and duplicate/stale snapshots.
- [x] Verify original clip keys and intermediate/loop samples through the compiled shared library.
- [x] Run Lua/UI, source packaging, geometry and external C host checks.
- [x] Prepare Windows C-host build script, cross-platform CI and transfer archives.

## Next work that can be coded locally

- [ ] Connect animation selection to uploaded clips as a full multi-bone pose output with transition blending. Current selection returns clip/time; host still orchestrates sampling/blending.
- [ ] Add source-rest-frame import and per-bone alignment validation to the conversion workflow. It must consume verified offsets rather than invent them.
- [ ] Generate an animated GLB or Blender preview after rest-frame data is available; verify skin weights and inverse bind matrices.
- [ ] Add tests for unsupported easing/weights, reflecting matrices and malformed source-package limits beyond current fixtures.
- [ ] Add C ABI size/offset checks and run external C host tests on Linux and Windows.
- [ ] Add a fake host scene that runs movement, camera, animation selection and mode restoration together for integration regression checks.
- [ ] Expose projectile stepping/impact/fuse events through C transport, with host-owned collision and damage.
- [ ] Add native error details to the existing local telemetry rather than reporting only numeric status codes.
- [ ] Benchmark mesh indexing with representative offline map geometry when a legal/local geometry export is available.
- [ ] Add avatar-selection UI against a mock approved catalogue; actual engine rendering remains separate.
- [ ] Add a fake authenticated server/client transport to test catalogue reconnects and missing-pack behavior together. It will not prove BeamMP compatibility.

## Needs Roblox Studio, but not BeamNG

- [ ] Run the existing exporter to capture the default R6 Motor6D C0/C1 offsets.
- [ ] Confirm omitted legacy easing defaults and motion against actual Studio playback.
- [ ] Validate target axis alignments, rig deformation and feet placement visually.
- [ ] Capture movement traces to calibrate walking, braking, air control and jumps.
- [ ] Export a test user avatar and check supported textures/accessories with the local package validator.

## Needs Windows or remote CI, but not BeamNG

- [ ] Run tools/build_windows.ps1 on a Windows x64 Rust/MSVC environment.
- [ ] Publish all local changes to GitHub and execute the configured CI workflow.
- [ ] Check DLL exports, ABI layouts and loader architecture with a standalone Windows host.
- [ ] Verify source and local-asset transfer archives on the Windows machine.

## Optional expansion; not required for first player test

- [ ] Building rotation, appearance editing and undo/redo.
- [ ] Coupled friction, rotating platforms, continuous moving collision and articulated AVBD.
- [ ] R15, custom animation packs, accessory animation and broader avatar import.
- [ ] Persistence and network protocol/version hardening.

## Boundary

None of these offline checks prove native loading, map collision extraction, rendering, camera takeover, vehicle coupling, or BeamMP integration. Those are separate runtime gates in TOMORROW_TEST_PLAN.md. Imported clips/rig remain local; original source assets are not in the public source ZIP.
