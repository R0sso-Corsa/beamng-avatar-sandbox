# Offline block placement plan

`mod/lua/ge/extensions/avatarSandbox/placement.lua` is a pure GE Lua planning extension. It creates no scene objects, previews or collision. Load with `extensions.load("avatarSandbox_placement")` for future host integration.

`preview(desiredCentre, playerBounds, vehicleBounds)` returns cube bounds or `nil, reason`. `place` repeats validation and returns a plan ID plus a copy of the bounds. `get` returns a copy, `remove` deletes a plan and `clear` clears all plans. IDs are not recycled during the extension lifetime. Mission end and unload clear plans. These are plan IDs, not BeamNG scene-object IDs.

Cubes are one metre wide, axis-aligned, with minimum coordinates floored to integer grid cells. The desired centre chooses the containing cell: (-0.1, 0.2, 0.5) becomes bounds (-1, 0, 0) to (0, 1, 1). Grid origin is world zero. A host picker must offset a surface hit to obtain a desired cube centre; ray-hit conversion is not implemented.

Player bounds are mandatory, and vehicle bounds are a collection of conservative world-space `{min={x,y,z}, max={x,y,z}}` AABBs. All coordinates must be finite and within one million metres. Positive-volume intersection with the player, any supplied vehicle or another planned block is rejected; touching faces is allowed. Player capsule/rotated-vehicle AABBs can reject some actually clear positions. The host must supply every relevant vehicle and fresh bounds; this module cannot discover omitted vehicles. Reach, line of sight, terrain penetration and stock-map object overlap are not checked here.

The offline test covers negative grid coordinates, preview without mutation, duplicate-cell rejection, player/vehicle overlaps, invalid bounds, adjacent face contact, defensive copies, removal and mission cleanup. Run `tests/placement_smoke.py` with Python and `lupa` available. It does not verify engine behavior.

Before runtime placement is connected: obtain a valid visible hit, check reach, supply fresh bounds, create the collision-enabled engine cube, then maintain an explicit plan-to-object mapping. Roll back the plan if object creation fails; remove engine collision before treating removal as complete. Scene unload must destroy engine objects as well as clear these plans. Test standing/jumping and vehicle collision in BeamNG. All these integration steps remain pending.
