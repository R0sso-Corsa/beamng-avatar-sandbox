# Original classic gear visual references

`model_references.json` records original handle dimensions, colors, mesh/texture
URIs, mesh scales and Tool.Grip matrices. Six definitions were extracted offline
from the publicly shared place at https://github.com/1alessandro/doomspire-brickbattle.
The missing paintball gun uses the classic definition in
https://github.com/Novetus/Novetus_src/blob/master/scripts/launcher/3DView.rbxl.
The latter is a classic reference, not proof of the current Doomspire variant.

No downloaded place scripts were executed or imported. Downloaded full places
remain under ignored assets/private/doomspire. Public files contain only visual
references; original Roblox meshes/textures are resolved by Studio, not bundled
or relicensed as project assets. Native game mesh export remains pending.

Studio gear_models.lua builds SpecialMesh handles from these definitions and
uses the original grip inverse in the hand weld. All six mesh/texture pairs
preloaded successfully using model instances; all seven server equip checks
passed. Superball is the source sphere Part and does not have a custom mesh.
Rocket projectile now references the classic missile mesh 2251534; its visual
load and trajectory have not been separately verified. Gear behavior is still
the lab prototype rather than the original scripts.

Component-reference checks:
- https://roblox.fandom.com/wiki/Sword
- https://roblox.fandom.com/wiki/Slingshot
- https://roblox.fandom.com/wiki/Rocket_Launcher
- https://roblox.fandom.com/wiki/Trowel
- https://roblox.fandom.com/wiki/Timebomb
- https://roblox.fandom.com/wiki/Paintball_Gun
