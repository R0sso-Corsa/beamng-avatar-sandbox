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
