> Portability update: camera and building-plan logic now live in Rust (`physics/src/camera.rs`, `physics/src/building.rs`) and are exposed through the C ABI. Lua camera/building/placement files are backend facades and cannot run standalone. Earlier Lua implementation descriptions below are historical; engine integration remains pending.

# Portable Doomspire-inspired gear foundation

The [Doomspire Brickbattle Tools wiki](https://doomspire-brickbattle.fandom.com/wiki/Tools) lists Sword, Slingshot, Rocket Launcher, Trowel, Bomb, Superball and Paintball Gun. `physics/src/gear.rs` supplies all seven as an independently implemented Rust inventory/command system. No Roblox scripts or gear assets are included. Hosts supply collision, projectile motion, damage, destruction, sounds, animations and rendering; these are commands, not working BeamNG weapons.

Equip with `GearSystem::equip`, advance cooldowns using simulation time, and activate with finite origin, aim direction and a compatible mode. Directions are normalized. Cooldowns belong to each gear and survive switching. Failed validation does not consume cooldown. Successful command emission consumes it even if a host later rejects the effect, so validate host prerequisites before activation. Multiplayer hosts must authorize commands and damage.

Wiki-derived defaults: sword slash/lunge damage 10/30 and ~0.75 s cooldown; slingshot damage 16 and 0.2 s; rocket damage 100 and 7 s; trowel 4 s; bomb damage 100 and 5 s; superball damage 55 and 2 s; paintball damage 20 and 0.5 s. Rocket speed 60 studs/s and blast radius 4 studs become 18 m/s and 1.2 m at the project's 0.30 scale. Trowel wall dimensions 4x1x3 studs become 1.2x0.3x0.9 m. Slingshot bounce damage becomes zero; superball bounce damage halves.

These are community-reported prototype defaults, not verified current Roblox behavior. The bomb's 3.8-second fuse is explicitly uncertain in the wiki, and an update-log entry reports a different sword cooldown. Unknown projectile speeds and paintball gravity are represented as `None`, requiring host configuration rather than invented constants. Paintball head/torso damage, paint/debris, sword idle contact and movement tricks, gradual wall construction/welding, explosion impulses, projectile lifetimes and bomb radius are not implemented. Core projectile commands cannot become playable without these host-side decisions.

## Optional building-tools gear

`Gear::BuildingTools` is disabled by default. `set_building_tools_enabled(true)` allows equipping and emitting place/remove commands; disabling it unequips it. This is our mod-owned placement/removal tool, now based on F3X-style editing, independently implemented for mod-owned parts. It is separate from the Trowel.

BeamNG's optional `avatarSandbox_buildingTools` action toggles the user setting (bind manually). Load the planner separately with `extensions.load("avatarSandbox_placement")`. The building-tools extension delegates plans only while enabled and resets permission on mission end/unload. It does not spawn engine objects; see [block placement](BLOCK_PLACEMENT.md). No Rust gear transport or BeamNG combat adapter is attached yet. The Rust gear API is currently not exposed through the C ABI.

Offline checks cover all seven gears, cooldowns, normalized/invalid aim, sword lunge, opt-in removal and revocation; Lua checks cover default-disabled building access and mission reset. No gameplay parity or in-game gear tests have been run.


## F3X-inspired editing

The user-selected [Building Tools by F3X reference](https://roblox.fandom.com/wiki/Building_Tools_by_F3X) guides the optional editor. The first offline subset adds world-axis move, symmetric resize and clone, alongside new cubes and delete. Calls operate on one plan ID and revalidate player, vehicle and other-part AABB overlap before committing. Dimensions have a 1 cm minimum. Failed edits preserve the existing part; clone failures leave no new part. The user enable setting gates all editing calls.

This is a partial F3X-inspired foundation, not the original plugin or its code/assets. Selection widgets, snapping increments for edits, local axes, rotation/pivots, paint/material/surface tools, anchor/collision toggles, non-cube parts, mesh/texture, welding, lighting, decoration, undo/redo and export remain pending. These features need explicit host support, especially rotated collision and dynamic parts. Runtime creation/rendering remains unimplemented. Existing cube plans are currently axis-aligned. Move/resize/clone and rejected-overlap rollback pass offline tests.


## Offline projectile and bomb simulation

`Projectile` now integrates constant-gravity trajectories and exposes a proposed segment for a host collision sweep. `advance` accepts the host's first impact fraction, unit normal and player-hit flag. Rockets emit explosion events on impact; player hits emit nominal damage. Superball surface bounces reflect velocity and halve damage; slingshot bounces reduce damage to zero. Paintballs expire on surface contact; painting/debris still require host code. Lifetimes are configured by the host. Unknown speed/gravity settings are explicit host choices, not measured defaults (paintball falls back to configured gravity until calibrated).

A collision segment is a chord of the ballistic trajectory, so keep simulation steps small. Projectile radius, swept collision queries, target IDs, body-region modifiers, owner filtering, damage application, explosion falloff and impulses remain host responsibilities. Impact response discards the remaining travel in that step instead of processing multiple bounces; lifespan still consumes the step's elapsed time. No projectile assets or C ABI transport are attached. `BombTimer` emits nominal explosion damage once when the provisional fuse elapses; the host supplies radius, location and impulse.

Offline regression checks cover gravity, rocket gravity immunity, reflected/halved superball hits, invalid-hit rollback, expiry, rocket explosion and one-shot bomb timers. All 17 Rust tests pass; BeamNG combat remains unimplemented.
