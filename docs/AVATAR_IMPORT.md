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

Appearance changes are separate from movement packets: transmit an approved ID on spawn/change, then compact position, facing and animation updates with sequence numbers. Do not send meshes or textures every frame, or let client-supplied paths decide what to load. BeamMP exposes custom client/server events, which could carry these messages; see [client events](https://docs.beammp.com/scripting/mod-reference/) and [server events](https://docs.beammp.com/scripting/server/latest-server-reference/). This describes our planned protocol, not built-in avatar synchronization.

Personal packs that are not already on the server would need an administrator upload/conversion workflow, followed by redistribution and client reconnect/reload as appropriate. Automatic Roblox fetching, runtime uploads and peer-to-peer file exchange are not implemented. All players need the avatar mod and approved assets to see custom characters. Without that client support they will not automatically render them.

## Validation status

The Python smoke check packages the existing noob source, verifies repeatable identity/checksums and rejects path traversal. This is offline only. Avatar selection UI, fallback rendering, Roblox Studio exporter, BeamNG conversion/loading and BeamMP registration/synchronization still need implementation and runtime tests.
