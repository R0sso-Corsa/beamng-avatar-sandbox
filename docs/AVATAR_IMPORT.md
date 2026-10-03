## Rust driver and catalogue update

`physics/src/driver.rs` owns 240 Hz input batching, diagonal normalization, heading rotation, jump edges, pause input release and bounded catch-up. `physics/src/catalogue.rs` owns approved avatar selection, revisioned snapshots, validation and missing-pack fallback. Lua delegates to injected Rust adapters; it no longer implements those rules. These modules currently expose Rust APIs, not C bindings. A native binding, authenticated transport and engine rendering remain required; no live BeamNG or BeamMP compatibility is claimed.

# Local R6 source imports and multiplayer plan

The initial import configuration accepts an exported OBJ with separate Head, Torso, LeftArm, RightArm, LeftLeg and RightLeg groups, optional MTL and local PNG/JPEG textures. Coordinates are metres, Z-up. This is a source package, not a rigged BeamNG avatar; Studio extraction, accessory conversion, skinning, animations and engine mesh/material conversion remain pending.

Put an `avatar.json` beside the source files:

```json
{"version":1,"rig":"R6","units":"metres","up":"Z","mesh":"avatar.obj","files":["avatar.obj","avatar.mtl","face.png"]}
```

List only files actually present. Package with:

```sh
python3 tools/package_avatar.py /path/to/export /path/to/avatar-source.zip
```

The tool checks the six body groups, finite/bounded vertices, local material/texture references, allowed file extensions, symlinks, file-count and byte limits. It adds SHA-256 file checksums and prints a content identity derived from the canonical manifest. It does not validate all OBJ topology or decode textures, so conversion and server approval must perform further checks. Export assets you are authorized to use. No executable avatar scripts are accepted. Changing the source or manifest changes the identity; ZIP timestamps do not affect it.

## How multiplayer players receive characters

Proposed first release: the administrator converts and approves avatar packs before a session, then includes the approved engine assets in the server's client mod bundle. BeamMP documents distributing client mods through `Resources/Client`; see [server mod setup](https://docs.beammp.com/guides/mod-creation/server/getting-started/). This provides a shared asset set when players join. It is not live upload of arbitrary avatars mid-session.

A future server-side avatar catalogue maps approved content IDs to engine asset paths. A client requests an avatar ID, and the server checks it against that catalogue before announcing the player-ID/avatar-ID association. Clients select the matching cached assets, falling back to the noob when missing or incompatible. New arrivals receive the current association list. Disconnect clears the player's avatar. Reconnection and catalogue revision need explicit resynchronization.

Appearance changes are separate from movement packets: transmit an approved ID on spawn/change, then compact position, facing and animation updates with sequence numbers. Do not send meshes or textures every frame, or let client-supplied paths decide what to load. BeamMP exposes custom client/server events, which could carry these messages; see [client events](https://docs.beammp.com/scripting/mod-reference/) and [server events](https://docs.beammp.com/scripting/server/latest-server-reference/). The pure Lua protocol below implements appearance bookkeeping, but live BeamMP transport and rendering remain pending.

Personal packs that are not already on the server would need an administrator upload/conversion workflow, followed by redistribution and client reconnect/reload as appropriate. Automatic Roblox fetching, runtime uploads and peer-to-peer file exchange are not implemented. All players need the avatar mod and approved assets to see custom characters. Without that client support they will not automatically render them.

## Validation status

The Python smoke check packages the existing noob source, verifies repeatable identity/checksums and rejects path traversal. This is offline only. Avatar selection UI, fallback rendering, Roblox Studio exporter, BeamNG conversion/loading and live BeamMP registration/synchronization still need implementation and runtime tests.


## Implemented catalogue and mock-tested appearance protocol

`avatarCatalogue.lua` provides `newServer(approvedHashes)` and `newClient(installedHashes)`. Catalogues accept up to 256 lowercase SHA-256 IDs; `noob` is built in. They contain IDs only, never client-supplied asset paths. The server `join`, `select` and `leave` methods return complete versioned snapshots with monotonically increasing revisions, covering up to 256 players. Initial selection is noob. Selection requires an approved ID and an already connected player. The authenticated player ID must come from the server connection context, not the request payload.

Clients atomically apply validated full snapshots, rejecting duplicate players, malformed IDs and stale/equal revisions. Unknown installed packs resolve to noob while retaining the requested ID. A snapshot replaces all prior associations, so disconnected players disappear and late joiners receive current appearances. `get` returns a copy. Call `reset` on disconnect/reconnect before accepting snapshots from a new server session. Catalogue changes currently require recreation/reinitialization.

The host must send selection requests to the server, then broadcast returned snapshots after accepted changes and send a snapshot to new clients. Only accept snapshots from the server channel, with size/rate limits before decoding. No event handlers, JSON codec, rate limiting, asset loading or rendering are attached by this module. Full snapshots are a simple first implementation; switch to deltas only if measured traffic warrants it. No mesh files travel through this protocol.

Mock-client tests cover approval rejection, late joining, appearance changes, stale packets, malformed-snapshot rollback, missing-pack fallback, defensive copies, disconnect cleanup and revision reset. The placement and avatar catalogue extensions are packaged in the Lua ZIP, but that ZIP still excludes avatar models.
