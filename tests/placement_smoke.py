"""Placement plan regression checks; requires lupa, no BeamNG runtime."""
from pathlib import Path
from lupa import LuaRuntime
root = Path(__file__).resolve().parents[1]
lua = LuaRuntime(unpack_returned_tuples=True)
lua.globals().placement = lua.execute((root / 'mod/lua/ge/extensions/avatarSandbox/placement.lua').read_text())
assert lua.globals().placement.preview()[0] is None
building = lua.execute((root / 'mod/lua/ge/extensions/avatarSandbox/buildingTools.lua').read_text())
lua.globals().building=building
lua.execute("""
assert(not building.setEnabled(true))
local calls=0
assert(building.attachBackend({setEnabled=function(v) return true end,
 place=function() calls=calls+1;return 42 end,remove=function() return true end,
 move=function() return true end,resize=function() return true end,
 clone=function() return 43 end,clear=function() calls=calls+1 end}))
assert(building.setEnabled(true));assert(building.place()==42)
building.onClientEndMission();assert(not building.status().enabled and calls==2)
""")
print('Rust building backend facade opt-in and cleanup passed')
