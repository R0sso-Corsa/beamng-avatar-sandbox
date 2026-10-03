"""Camera math and character choice tested without BeamNG."""
from pathlib import Path
from lupa import LuaRuntime
root=Path(__file__).resolve().parents[1]
lua=LuaRuntime(unpack_returned_tuples=True)
lua.globals().cameraModule=lua.execute((root/'mod/lua/ge/extensions/avatarSandbox/camera.lua').read_text())
lua.globals().main=lua.execute((root/'mod/lua/ge/extensions/avatarSandbox/main.lua').read_text())
lua.execute("""
local c=cameraModule.new()
local focus={x=0,y=0,z=1}; local eye={x=0,y=0,z=1.4}
assert(not c.pose(focus,eye)); assert(c.setActive(true))
assert(c.pose(focus,eye).position.y==-3)
assert(c.zoom(-6)); local p=assert(c.pose(focus,eye))
assert(p.firstPerson and p.hideLocalHead and p.position.z==eye.z)
assert(c.zoom(1)); assert(not c.pose(focus,eye).firstPerson)
assert(c.zoom(100)); assert(c.status().distance==15)
assert(c.look(math.pi/2,100)); assert(c.status().pitch<=math.rad(80))
local blocked=assert(c.pose(focus,eye,0.2)); assert(blocked.actualDistance<3)
assert(c.status().distance==15) -- obstruction doesn't change user zoom
assert(not c.pose(focus,eye,-1)); assert(not c.zoom(0/0))
assert(main.status().characterMode=='beamng')
assert(not main.setCharacterMode('roblox')); assert(main.setEnabled(true))
assert(main.setCharacterMode('roblox')); assert(not main.status().avatarActive)
assert(main.setCharacterMode('beamng'))
local enter,leave=0,0
assert(main.attachCharacterAdapter({enter=function() enter=enter+1; c.setActive(true); return true end,
 leave=function() leave=leave+1; c.setActive(false); return true end}))
assert(main.setCharacterMode('roblox')); assert(main.status().avatarActive)
assert(main.setCharacterMode('roblox')); assert(enter==1)
assert(main.setEnabled(false)); assert(leave==1 and not c.status().active)
assert(main.status().characterMode=='beamng')
assert(main.setEnabled(true)); assert(main.setCharacterMode('roblox'))
main.onClientEndMission(); assert(leave==2 and main.status().characterMode=='beamng')
assert(main.attachCharacterAdapter({enter=function() return false end,leave=function() return true end}))
assert(main.setEnabled(true)); assert(not main.setCharacterMode('roblox'))
assert(main.status().characterMode=='beamng')
""")
print('Camera zoom/orbit/obstruction and character-mode lifecycle passed')
