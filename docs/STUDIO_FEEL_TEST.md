# Local Studio feel test

Open the locally saved `assets/private/r6-feel-test.rbxl` and press Play (F5).
WASD moves, Space jumps, right-drag orbits, and scrolling zooms into first person.
The noob R6 character uses Roblox's idle, walk, jump, fall and climb clips.
Blocks of 0.5, 1, 2, 3 and 4 studs and a wall provide collision/jump reference targets.

This is a native Humanoid / PlayerModule reference, not the Rust runtime.
WalkSpeed 16, JumpPower 53, gravity 196.2 and MaxSlopeAngle 45 match current
profile settings, but acceleration, contact solving, step handling and camera
smoothing remain Roblox implementations. It cannot validate BeamNG integration.

The client is `tools/roblox/feel_test.client.lua`. To install in another local
baseplate, run `tools/roblox/setup_feel_test.lua` in the Command Bar with
`CLIENT_SOURCE` set to that client file's contents. Save locally before Play.
The setup replaces its named test objects and moves R6Reference into ServerStorage.

Verified in Studio: client starts as R6 and logs
`AVATAR_FEEL_TEST_READY R6 16 53 196.1999969482422`; avatar, course and control HUD
are visible. Manual movement and camera feel are for the user to evaluate.

## Integrated lab

`test_lab.server.lua`, `test_lab.client.lua` and `setup_test_lab.lua` extend the
same local place. Server-created blocks, projectile parts and avatar changes
replicate using Roblox networking. Set SERVER_SOURCE and LAB_CLIENT_SOURCE to
those files before running the installer in the edit-mode Command Bar.

- Animation: move, stop, jump, fall, land, climb the truss, and respawn rapidly.
- Camera: scroll into first person, orbit by the wall, enter the rear corridor.
- Metrics: live speed/acceleration, sampled root rise and airtime, stopping
  displacement, camera distance and instantaneous frame rate. Spawn falls also
  count as airborne intervals; these are reference readings, not calibrated
  measurements. Use “Log current measurements” to retain a snapshot in Output.
- Avatar: switch noob/alternate color + welded hat; toggle the face and a local
  smile T-shirt sample; force a missing-preset fallback to the noob palette.
  This exercises presentation, not the production import/download pipeline.
- Gear keys 1–7: sword, slingshot, rocket, bomb, superball, trowel, paintball.
  Click to activate toward the mouse. These are simple visual/physics prototypes,
  not the production Rust gear logic: sword displays a strike, trowel places a
  wall, balls are native rigid bodies, paintball colors lab blocks. Rocket/bomb
  explosions are visual only, with pressure and joint destruction disabled.
- B toggles building; Q cycles place/move/rotate/resize/delete. Click terrain to
  place, or a lab block to edit. Move raises 2 studs, rotate turns 15 degrees,
  resize widens 1 stud (max 12). The server enforces ownership, 40-stud reach,
  cooldowns and a 100-block cap. This is a compact F3X-style interaction probe,
  not the full building editor.
- R respawns; V switches avatar. Onscreen buttons provide the same lab actions.
- Local errors appear on the HUD; Output contains startup and measurement logs.
  Nothing is uploaded by the diagnostic code.

For multiplayer use Studio's local server/client test with two clients. Check
both HUDs report two players, watch the other avatar move/jump/switch outfit,
place/edit a block from one client, and confirm the other cannot edit it.
Join/leave and respawn repeatedly. Roblox handles character/Animator replication;
this does not test BeamMP or native Rust catalogue transport.

Verified in single-client Play: startup, HUD, and `smoke_test_lab.lua` passed
place, ownership replication, resize, accessory, missing-preset fallback and
delete assertions. Move/rotate requests also ran without errors. Two-client
presentation, every gear trajectory and subjective movement/camera feel remain
manual checks. Run the smoke script in the client Command Bar during Play.
