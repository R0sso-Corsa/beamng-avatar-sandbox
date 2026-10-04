# Source sword

`original_server.lua` is the unmodified Sword Server script extracted from the Doomspire place recorded in ../model_references.json (1alessandro/doomspire-brickbattle). The upstream software license is preserved in LICENSE; this does not license Roblox mesh, sound, or animation assets.

The Studio adapter is tools/roblox/doomspire_sword.server.lua. Changes: replace Other.GetPlayerFromTool with our player lookup, route activation through the lab BindableEvent, and clean up the lunge BodyVelocity after 0.5 seconds (the original place does this in its character script). Cosmetic Player.Effect branches are retained but inactive without the source game's cosmetic fixtures. Native Tool grip, touched damage and enabled state remain source behavior.

The source emits toolanim Slash/Lunge signals. The lab consumes them with classic R6 clips 129967390/129967478, above the holding animation. Slash immediately resets contact damage to 5 in this source; the transient value 10 is not silently extended. Double-click within 0.2 seconds starts a 30-damage lunge, grip out after 0.25 seconds and reset after 1 second.

physics/src/sword.rs ports the timing, contact filtering, grip state and lift request. Hosts supply contact detection, audio, animation playback and force application. The lift is 14.5 studs/s = 4.35 m/s at the project's 0.30 m/stud scale; the Roblox force limit is not a portable physical calibration.
