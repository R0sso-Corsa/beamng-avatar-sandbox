# BeamNG Avatar Sandbox

An experimental BeamNG.drive mod project for a controllable Roblox-style avatar, player interaction, and block placement on existing BeamNG maps.

## Status

An initial Lua mod skeleton is implemented: explicit loading, enable/disable actions, mod-owned interaction targets, reach validation, and lifecycle cleanup. A packager and offline smoke checks are included. No BeamNG engine tests have been run, and this is not yet a playable avatar mod. Roblox avatar conversion, animation playback, engine targeting, and runtime block collision remain pending.

## Development

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

Start by evaluating BeamNG's existing walking controller and visual mesh support. Prove animation playback with one character before generalising avatar import. Use a Lua gameplay extension for interactions and evaluate collision-enabled TSStatic objects for placed blocks. Validate these choices against the installed BeamNG version; they are not confirmed implementation decisions.

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
