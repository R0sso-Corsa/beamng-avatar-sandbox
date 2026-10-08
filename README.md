# BeamNG Avatar Sandbox

An experimental BeamNG.drive mod project for a controllable Roblox-style avatar, player interaction, and block placement on existing BeamNG maps.

## Status

The active development route is now **two games running together**: native
Roblox Studio character simulation, a Rust localhost companion, and a BeamNG
adapter. Real Studio → relay → synthetic-host pose/input/collision exchange has
been tested on macOS. The BeamNG half still requires in-game testing; this is not
yet a playable BeamNG avatar mod. See [setup and evidence](docs/PASSTHROUGH.md)
and [the bridge contract](docs/CONTRACT.md).

An initial Lua mod skeleton is implemented: explicit loading, enable/disable actions, mod-owned interaction targets, reach validation, and lifecycle cleanup. A packager and offline smoke checks are included. No BeamNG engine tests have been run, and this is not yet a playable avatar mod. Roblox avatar conversion, animation playback, engine targeting, and runtime block collision remain pending.

## Development

An engine-neutral Rust character physics prototype now lives in `physics/`. It implements a limited AVBD contact solver against static planes, with walking and jumping. See [character physics research](docs/CHARACTER_PHYSICS.md) for sourced Roblox metrics, provisional tuning, test commands and integration limits. It is not yet connected to BeamNG; the Lua ZIP does not include or load it.

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for packaging, installation, console commands, and engine acceptance checks. Build the test ZIP with `python3 tools/package_mod.py`; run offline logic checks with `lua tests/smoke.lua` from this repository's root.

## First playable milestone

Load a stock BeamNG map, move around as a visible blocky character, aim at a nearby block, place it, jump onto it, and remove it.

Player movement and interaction come first. Map creation is outside the initial scope.

## Planned features

- Visible player avatar with walking, jumping, and a third-person camera.
- Nearby-object targeting, highlighting, and an interaction action.
- One stationary block type with placement preview, placement, and removal.
- Roblox avatar import through a Studio-assisted export and conversion process, starting with R6.
- Compatible idle, walk, and jump animations, followed by run, fall, and landing transitions.

## Proposed approach

Primary route: Studio owns native movement and animations; the companion relays
poses and host input/collision snapshots. The host must render the avatar,
provide map geometry and integrate controls/camera. The first implementation
uses official Studio scripting, not the protected Roblox desktop player's
executable. A local Studio Play session must remain running.

The existing Rust recreation remains available as a separate optional backend:

Use the portable Rust core for avatar movement, with a separate BeamNG adapter for collision queries, input and visual transforms. Native Rust loading or IPC from BeamNG remains to be validated. Prove animation playback with one character before generalising avatar import. Use a Lua gameplay extension for interactions and evaluate collision-enabled TSStatic objects for placed blocks. Validate these choices against the installed BeamNG version; they are not confirmed implementation decisions.

Avatar appearance, animation, and collision are separate concerns. Roblox scripts and engine services will not run directly in BeamNG. An avatar importer must convert geometry, textures, body transforms, and compatible animation data rather than copy Roblox gameplay scripts.

## Initial limits

Single-player, one stock map for initial testing, one avatar, and one stationary block type. R15, layered clothing, dynamic heads, custom animation packs, movable physics blocks, multiplayer, direct username import, and save/load are later work.

Keep game files, exported avatars, textures, and third-party animation assets out of this repository unless redistribution is permitted. Users supply their own compatible assets. This is an unofficial project, not affiliated with BeamNG or Roblox.

## Tracking

See [ROADMAP.md](ROADMAP.md) for milestones and acceptance checks.

## References

- [BeamNG Lua extensions](https://docs.beamng.com/modding/programming/extensions/)
- [BeamNG TSStatic objects](https://documentation.beamng.com/modding/levels/level_classes/tsstatic/)
- [Example walking-mode character mod](https://www.beamng.com/threads/mita-miside-unicycle-mod.102090/)
- [Roblox avatar creation API](https://create.roblox.com/docs/reference/engine/classes/Players)

The Rust core also provides a versioned C interface for other game adapters. See [cross-game support](docs/CROSS_GAME_SUPPORT.md) for build instructions, the C host check and current limits.
