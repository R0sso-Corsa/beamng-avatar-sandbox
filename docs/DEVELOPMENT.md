# Development and first engine check

## What exists

An explicitly loaded Game Engine Lua extension, two optional input actions, and a registry for mod-owned interaction targets. Target validation checks finite positions, a three-metre player reach, and a visibility result supplied by a future engine picker. Focus expires each frame and after one interaction. Disabling clears focus; level exit and extension unloading clear the registry.

This is a foundation, not the playable prototype. It does not spawn or replace a player, animate an avatar, raycast, highlight targets, place blocks, download Roblox assets, or change stock maps. Native walking remains available through BeamNG itself.

## Package and install

Run `python3 tools/package_mod.py`. Put `dist/beamng-avatar-sandbox.zip` in the BeamNG user folder's `mods` directory, located through the BeamNG launcher. For development, copy the contents of `mod/` into `mods/unpacked/beamng-avatar-sandbox/`. Do not install both copies together.

The ZIP contains `lua/` at its root, not a wrapping project folder. No automatic loading or default key overrides are included. In Options > Controls, bind **Avatar Sandbox: toggle** and **Avatar Sandbox: interact** to unused keys. Action discovery and category placement require engine verification.

## Console smoke check in BeamNG

Load a stock map. Use the **GE Lua** console (not vehicle Lua):

```lua
extensions.load('avatarSandbox_main')
avatarSandbox_main.setEnabled(true)
dump(avatarSandbox_main.status())
avatarSandbox_main.registerTarget('test', 'Console test', function(id) print('Interacted: ' .. id) end)
avatarSandbox_main.setTarget('test', {x=0,y=0,z=0}, {x=1,y=0,z=0}, true); avatarSandbox_main.interact()
extensions.unload('avatarSandbox_main')
```

Run the setTarget/interact pair together: focus is cleared on the next update. These are synthetic coordinates and a supplied visibility flag, not a real scene hit. This checks loading and callback plumbing only.

## Offline checks

From the repository root, run `lua tests/smoke.lua` with Lua 5.1+ or LuaJIT. The checks use plain Lua without BeamNG and cover out-of-range/obstructed hits, invalid coordinates, stale focus, removal, callback errors, and cleanup. They do not establish engine compatibility.

## Next implementation work

1. Inspect the target BeamNG installation's walking controller and visual mesh slots; record the game version.
2. Prove one visible original avatar and supported body animation technique before designing an import format.
3. Add a real picker in this extension's update path: sample player position, raycast from the camera, verify line of sight from the player, resolve only registered targets, then call setTarget. Clear target on every miss and when outside walking mode. Update ordering must be tested.
4. Add highlighting and one collision-enabled block only after the player foundation works.

The `visible` argument is an internal assertion from the picker, not a collision check. Do not use a UI/client-provided flag as proof of visibility or multiplayer authority.

## Avatar import boundary

R6 import remains planned. Start with a user-provided Studio export, preserve body-part transforms and supported textures/accessories, and adapt the result to the proven BeamNG renderer. Model conversion does not imply animation conversion. Do not execute scripts from imported avatar packages or include proprietary assets in the repository.

## Engine acceptance checklist

- Extension loads/unloads without console errors.
- Both actions appear and toggle state; repeated toggles do not reload/reset state unexpectedly.
- Native walking, jumping, cameras, vehicle entry/exit and normal driving remain functional.
- Synthetic console interaction runs once and failed/expired hits do not run callbacks.
- Changing levels clears targets and disables the extension.
- Unloading leaves no callbacks, model objects, bindings or map modifications owned by this extension.

Record results and game version here when a test machine becomes available. No engine acceptance checks have been run yet.

## Current handoff

Use `python tools/validate_offline.py` for the consolidated offline checks. See [tomorrow's test plan](TOMORROW_TEST_PLAN.md) for Windows build preparation, current adapter contracts and ordered runtime gates. The ZIP is Lua/UI only and does not contain a playable avatar or native loader.
