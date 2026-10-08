# Native Studio ↔ Rust ↔ BeamNG prototype

The main direction is now live state exchange: Studio runs the real Humanoid,
PlayerModule and Animate script; a companion forwards snapshots and commands.
This avoids rewriting those systems. It still needs host rendering, collision
extraction, controls and camera integration. The working guest is a local
**Roblox Studio Play session**, not the normal RobloxPlayer desktop client.

[Contract](CONTRACT.md): coordinate mapping, ownership, transport and lifecycle.
[SkyCraft](https://github.com/chasmlol/SkyCraft) is the closest documented model;
our first slice uses official Studio scripts and local HTTP/UDP instead of
Minecraft's Fabric/native shared-memory integration. Nothing from leaked Roblox
source or the private decompilation is needed or included.

## Run the local prototype

A configured local test place was saved as `assets/private/r6-passthrough.rbxl`
on the development Mac. It is ignored, contains a session token, and stays in
Edit mode after testing. Open it locally and start the companion with the
matching `imports/passthrough-session.json`; reinstall its config after rotation.


Requirements: Rust/Cargo, Python, installed Roblox Studio. Full host integration
will also require an owned BeamNG installation on Windows. The Rust companion
builds separately; the Lua/UI ZIP does not include the executable or credentials.

```sh
cargo build --release --manifest-path bridge/Cargo.toml
python3 tools/configure_passthrough.py
bridge/target/release/avatar-bridge --config imports/passthrough-session.json
```

On Windows the executable is `bridge\target\release\avatar-bridge.exe`.
The generator writes an ignored private config and
`exports/install_passthrough.lua`. Neither may be committed or published.
A second invocation refuses to overwrite an existing session; use `--rotate`
only when restarting both peers and reinstalling the Studio configuration.
`--host-origin X Y Z` maps the local Studio origin to a chosen host location.
Default scale is 0.3 m/stud; the wire is metres and Z-up.

In a **disposable local place**, enable HTTP requests in Experience Settings,
Security. Keep a spawn platform available during startup. In Edit mode paste the
generated installer into Studio's Command Bar. It installs the server/client
bridge scripts and creates a noob R6 StarterCharacter if none exists. An existing
StarterCharacter is retained; it must be R6. Use a place with the default
PlayerModule; the old feel-test lab overrides its camera/controller and is not
the isolated acceptance fixture. Do not publish a place containing the token.

Start Play and run a stand-in host in a second terminal:

```sh
python3 tools/passthrough_host.py --duration 8 --move 1 0
```

This sends a floor and wall as static collision boxes, requests native movement,
and records body-part and camera snapshots in ignored `exports/`. It is a
synthetic peer, not BeamNG. `--jump` requests jump; `--crash` omits graceful
disable so the one-second lease can be checked. Default completion disables
remote control. The synthetic harness starts sequences from wall-clock
milliseconds to permit repeat experiments; runtime freshness uses monotonic
relay receipt times. Production peer restart requires a new shared session.

In Studio, `ReplicatedStorage.AvatarPassthrough` exposes `Exchanges` and
`LastError` on the server, `ClientError` on the client. No credentials are logged.
Before the host connects, movement remains normal Studio movement. Keep the
bootstrap platform until the mirrored floor exists; otherwise a character can
fall during guest startup. Last accepted collision proxies remain anchored
through disconnect; remote movement stops and local controls return.

## Prepared BeamNG extension

Install the mod ZIP. Copy the private session JSON into the BeamNG user folder
as `settings/avatarSandboxPassthrough.json`, then in the GE Lua console:

```lua
extensions.load('avatarSandbox_passthrough')
local b = extensions.avatarSandbox_passthrough
b.start(jsonReadFile('settings/avatarSandboxPassthrough.json'))
b.setColliders({{id=1,position={0,0,-0.15},size={60,60,0.3}}})
b.setEnabled(true)
b.setInput(1,0,false)
b.status()
b.getPose()
-- Stop after the diagnostic:
b.stop()
```

Check every return value. JSON config path access, LuaSocket loading, UDP,
onUpdate timing and these engine entry points require verification in BeamNG.
The extension defaults off, performs nonblocking bounded UDP work, and stops on
mission exit/unload. It clears input on pause. It **does not** claim host controls,
change a vehicle, take over a camera, draw a model, or extract stock geometry.
`attachRenderer(function(pose) ... end)` is the boundary for a future validated
host renderer; body size is local right/up/back dimensions, not world AABB size.
The next BeamNG milestone is one logged native position, then a correctly drawn
part. Character-mode UI takeover comes after that works.

## Verification on 8 October 2026

Mac Studio 0.742.0.7421053, local place, native R6 player and default Animate.
No place published and no protected executable hooked.

- Actual server HttpService exchanged with the Rust localhost process.
- A seven-second synthetic-host run recorded 33 distinct native snapshots.
  An explicit test reset occurred early; the subsequent segment walked at
  4.8 m/s (16 studs/s), stood at root height about 0.9 m above the mirrored floor,
  and stopped around X=7.70 m at the mirrored wall centered at X=8 m.
- Six native part transforms crossed the bridge. Right-arm orientation varied
  during movement while the native Animate script remained present.
- A four-second jump/crash trial recorded 19 native snapshots and Running,
  Jumping, Freefall and Landed states. After host disappearance, MoveDirection
  became zero, Jump became false and PlayerModule controls were enabled again.
- The first attempt without a startup floor fell before the host supplied
  geometry and respawned; it was not accepted as a steady movement trial.
- A final four-second wall/crash run on the updated 8 KiB relay recorded 20
  distinct snapshots with the bootstrap floor removed. Root height stayed
  between 0.8993 and 0.9000 m; maximum horizontal speed was 4.8 m/s, final X was
  7.7051 m and final horizontal speed below 0.002 m/s. Local controls returned.
- A separate two-second native trial verified that controls disabled before
  takeover remain disabled on release, rather than being incorrectly enabled.
- Full Mac offline validation passed, including 3 bridge tests, 28 existing
  Rust core tests, transport/Lua checks, the C host and 271 saved animation
  comparisons. Remote CI and Windows/Linux execution are not claimed.
- Real HTTP/UDP tests exercise auth/version/role errors, invalid/duplicate
  snapshots, stale peers, request size and deadline. Lua mock checks exercise
  scheduling, normalization, pause and lifecycle. These do not validate BeamNG.

Five-Hz HTTP is a connectivity prototype with visible control/pose delay.
The 8 KiB cap accommodates macOS's default UDP datagram limit; larger render
assets need a separate channel. Up to 16 rigid parts and 32 static boxes fit
within the encoded payload budget, which is authoritative over object counts.
Full head geometry, face textures, gears, native shadows, arbitrary terrain,
vehicle interaction, focus arbitration and multiplayer remain unfinished.

## Stop / uninstall

Stop the host adapter, stop Studio Play, then stop the relay with Ctrl-C.
Remove `AvatarPassthrough` from ReplicatedStorage, ServerScriptService and
StarterPlayerScripts in Edit mode. Remove a StarterCharacter only if it carries
`AvatarPassthroughCreated=true` and you no longer need it. Restore HTTP settings
if changed for this test. Remove the private config/installer and the copied
BeamNG settings file when no longer needed. Existing Rust recreation files and
assets remain available separately.
