# Local two-game bridge contract v1

Route: live state exchange. Roblox Studio runs native Humanoid physics and
animation; BeamNG draws guest transforms and supplies host collision proxies.
No binary hooks, engine embedding, captured frames or engine rewrite are used.
This first slice is a diagnostic connection, not a complete playable mashup.

Guest target: local Studio Play session (macOS 0.742.0.7421053 at setup).
Host target: BeamNG GE Lua extension; game build and LuaSocket availability
unverified until run on Windows. Companion: Rust, macOS first.
Nearest precedent: [SkyCraft](https://github.com/chasmlol/SkyCraft), especially
[its design](https://github.com/chasmlol/SkyCraft/blob/main/docs/DESIGN.md).
Our prototype exchanges six rigid body-part transforms and axis-aligned boxes
via local HTTP/UDP, rather than Minecraft meshes via native shared memory.

## Ownership

| System | Owner and first-slice behavior |
| --- | --- |
| Movement / animation | Native Studio player; host movement intent overrides local controls only during a fresh enabled lease. |
| Camera | Native Studio camera is exported. BeamNG camera takeover is pending; the diagnostic extension leaves host controls/camera untouched. |
| Host collision → guest | Host sends complete snapshots of up to 32 static AABBs. Studio creates invisible anchored Parts. No stock-map extraction yet. |
| Guest collision → host | Pending; exported body parts are visual diagnostics, not BeamNG physical bodies. |
| Inventory / gears | Studio retains native scripts; gear commands, effects and asset rendering are pending. |
| Vehicles / NPCs / damage | Host retains its systems; no bridge interaction yet. |
| Saves / menus | Each application keeps its own. No game saves written by the relay. |

## Units and axes

Wire coordinates: metres, seconds, right-handed X/Y horizontal and Z up.
Studio point `p` maps to `hostOrigin + scale * (p.X, -p.Z, p.Y)`.
Directions map to `(v.X, -v.Z, v.Y)`; velocities also multiply by scale.
Default scale is 0.3 metres/stud, a project choice. All peers use the same
session configuration. Inverse point is `(q.X, q.Z, -q.Y) / scale` after
subtracting hostOrigin. Box size is `(size.X, size.Z, size.Y) / scale`.
Parts carry position, size, right/up/back unit vectors and RGB [0,1].
Snapshots are atomic; every part and camera value belongs to that snapshot.

## Channels and protocol

UTF-8 JSON, protocol version 1. Each message contains `version`, `token`,
`session`, `sequence` (positive integer ≤2^53-1), `kind` (host/guest), `payload`.
HTTP guest: POST `/exchange` on 127.0.0.1:28741, sequential at 5 Hz.
UDP host: request/reply on 127.0.0.1:28742, up to 20 Hz, nonblocking game thread.
Both ports are configurable. Limit 8 KiB/message, 16 parts, 32 static boxes.
HTTP headers are bounded to 8 KiB; no chunked requests or persistent connections.
The relay keeps only the latest snapshot per role; old/duplicate sequences are
rejected. A 512-byte payload margin keeps replies within the packet cap. Empty
colliders may be [] or {} for Lua JSON interoperability. No discrete events are sent in this version, so replacing state cannot
lose a queued damage/use event. Invalid input preserves the previous state.

Reply contains version/session, echoed request sequence, `ok`, `active`,
`peerSequence`, `peerAgeMs`, and peer `payload` (null if stale/absent).
`active` requires both peers fresh within one second and host `enabled=true`.
Monotonic relay receipt time determines freshness, not either game's clock.
Clients also expire their local lease after one second. No catch-up/replay.

## Lifecycle

Start companion with a private generated config, start Studio Play, then start
host adapter with the same config. Normal BeamNG remains the default.
Pause/mission exit/unload sends disabled state, clears commands and closes socket.
Guest disconnect stops remote movement and re-enables local Studio controls.
Relay shutdown makes peer leases expire. Restarting either peer requires a new
session/config and restarting both adapters, preventing delayed packets from
reviving an old player. Regenerating config rotates the local token.

Local-only: no Internet listeners, no official Roblox game server connections,
no usernames, account cookies, assets or executable data in protocol/logs.
Studio scripts explicitly refuse non-Studio sessions. Session credentials stay
under ignored imports/exports and must not be committed or published with a place.

## First acceptance slice

Real Studio player → Rust relay → synthetic host, with monotonically increasing
snapshots, coordinate checks, native movement over a mirrored floor, stopping on
host disable/crash and unchanged local controls after release. Then run the
prepared extension in BeamNG and verify one marker before adding a renderer.
HTTP at 5 Hz is deliberately a connectivity prototype, not responsive final
control. Mesh/material/texture transfer, depth-correct rendering, robust map
collision, focus/pause arbitration and BeamMP remain separate acceptance gates.
