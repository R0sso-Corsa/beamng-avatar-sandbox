"""Camera math and character choice tested without BeamNG."""
from pathlib import Path
from lupa import LuaRuntime
root=Path(__file__).resolve().parents[1]
lua=LuaRuntime(unpack_returned_tuples=True)
lua.globals().cameraModule=lua.execute((root/'mod/lua/ge/extensions/avatarSandbox/camera.lua').read_text())
lua.globals().main=lua.execute((root/'mod/lua/ge/extensions/avatarSandbox/main.lua').read_text())
lua.execute("""
assert(not cameraModule.new())
local active=false; local forwarded=0
local c=assert(cameraModule.new({
 setActive=function(value) active=value; return true end,
 look=function(y,p) forwarded=forwarded+1; assert(y==1 and p==2); return true end,
 zoom=function(n) forwarded=forwarded+1; assert(n==-6); return true end,
 pose=function(f,e,h) forwarded=forwarded+1; return {firstPerson=true} end
}))
assert(c.look(1,2)); assert(c.zoom(-6)); assert(c.pose({},{}).firstPerson); assert(forwarded==3)
assert(main.status().characterMode=='beamng')
assert(not main.setCharacterMode('roblox')); assert(main.setEnabled(true))
assert(main.setCharacterMode('roblox')); assert(not main.status().avatarActive)
assert(main.setCharacterMode('beamng'))
local enter,leave=0,0
assert(main.attachCharacterAdapter({enter=function() enter=enter+1; c.setActive(true); return true end,
 leave=function() leave=leave+1; c.setActive(false); return true end}))
assert(main.setCharacterMode('roblox')); assert(main.status().avatarActive)
assert(main.setCharacterMode('roblox')); assert(enter==1)
assert(main.setEnabled(false)); assert(leave==1 and not active)
assert(main.status().characterMode=='beamng')
assert(main.setEnabled(true)); assert(main.setCharacterMode('roblox'))
main.onClientEndMission(); assert(leave==2 and main.status().characterMode=='beamng')
assert(main.attachCharacterAdapter({enter=function() return false end,leave=function() return true end}))
assert(main.setEnabled(true)); assert(not main.setCharacterMode('roblox'))
assert(main.status().characterMode=='beamng')
""")
print('Camera zoom/orbit/obstruction and character-mode lifecycle passed')
