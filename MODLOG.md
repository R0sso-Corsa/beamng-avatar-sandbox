# Mod development journal

## 8 October 2026 — native Studio passthrough route

User chose to run both games instead of extending the Rust recreation as the
main gameplay route. Preserve existing Rust work as an optional reference.
Use a local Studio Play session, official scripting APIs and a Rust relay;
no protected RobloxPlayer hooks or leaked source. Native player movement and
animations stay in Studio. First slice is state/intent/box-collision exchange,
not a compositor or complete BeamNG integration.

SkyCraft README/design reviewed as the nearest reported precedent. Universal
Modder knowledge searches found no Roblox or BeamNG notes. Runtime ownership,
units, bounded channels, freshness and restart contract are in docs/CONTRACT.md.
Each full test needs Windows, an owned BeamNG installation, Roblox Studio with
a local place and the companion. Mac can test Studio + companion + stand-in.
BeamNG is unavailable here. Actual guest HTTP transport must be proven in Studio.


### Verified first slice

Local Studio 0.742.0.7421053 successfully used HttpService with the Rust relay.
Native R6 PlayerModule/Animate remained active. Recorded walk/arm transforms,
wall collision, jump/freefall/landing, graceful disable and abrupt-host-loss
recovery. Final updated-code run: 20 distinct snapshots, 4.8 m/s maximum walk,
root height 0.8993–0.9000 m over the mirrored floor, final X7.7051 m against wall
X8 and final horizontal speed below0.002 m/s. Controls enabled, zero
MoveDirection and Jump=false after peer loss. Bootstrap floor was removed for
this acceptance pass. First floorless startup fell before host connection and
respawned; rejected as a steady movement test. Keep a small spawn safety floor.

Mac default UDP rejected a 25 KB test datagram; capped the protocol at8 KiB,
reserved reply overhead and limited part snapshots to16. Larger meshes require
a separate channel. A rejected HTTP request initially closed with unread body,
resetting the connection; bounded body consumption now returns clean rejection.
Full offline validation passed with3 bridge +28 existing Rust tests, real
HTTP/UDP boundaries/deadlines, Lua facade, C host and271 saved animation samples.
Log remains private: passthrough-validation-2026-10-08.log. Native captures and
session secrets stay in ignored exports/imports. No recovered engine source or
assets entered these changes. No BeamNG execution, renderer, stock-map geometry,
normal RobloxPlayer integration, Windows/Linux runtime or multiplayer claimed.


Saved a separate private r6-passthrough.rbxl with the generated noob rig, current
scripts/config and a small bootstrap platform. Original native-camera.rbxlx was
not overwritten. Studio left in Edit mode and the companion stopped. Credentials
and fixture are ignored and file permissions restricted; do not publish the place.


Final restoration fix preserves PlayerModule's prior controlsEnabled state.
Verified in a separate native two-second trial (10 snapshots): previously
disabled controls stayed disabled after release, then explicitly restored for
local use. Saved updated private fixture and stopped Play and relay again.
