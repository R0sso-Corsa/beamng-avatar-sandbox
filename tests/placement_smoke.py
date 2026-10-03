"""Placement plan regression checks; requires lupa, no BeamNG runtime."""
from pathlib import Path
from lupa import LuaRuntime
root = Path(__file__).resolve().parents[1]
lua = LuaRuntime(unpack_returned_tuples=True)
lua.globals().placement = lua.execute((root / 'mod/lua/ge/extensions/avatarSandbox/placement.lua').read_text())
lua.execute("""
local player={min={x=5,y=5,z=0},max={x=6,y=6,z=2}}
local centre={x=-0.1,y=0.2,z=0.5}
local preview=assert(placement.preview(centre,player,{}))
assert(preview.min.x==-1 and preview.max.z==1)
assert(placement.status().count==0)
local id,cube=placement.place(centre,player,{})
assert(id and placement.status().count==1)
cube.min.x=999; assert(placement.get(id).min.x==-1)
assert(not placement.place(centre,player,{}))
assert(not placement.preview({x=5.5,y=5.5,z=0.5},player,{}))
assert(not placement.preview({x=1,y=0,z=0},player,{{min={x=1,y=0,z=0},max={x=2,y=1,z=1}}}))
assert(not placement.preview(centre,player,{false}))
assert(not placement.preview({x=0/0,y=0,z=0},player,{}))
assert(placement.place({x=0.1,y=0.2,z=0.5},player,{})) -- face-touching permitted
assert(placement.remove(id)); assert(not placement.remove(id))
assert(placement.place(centre,player,{}))
placement.onClientEndMission(); assert(placement.status().count==0)
assert(not placement.remove(id))
""")
print('Placement snapping, overlap, isolation, removal and cleanup passed')
