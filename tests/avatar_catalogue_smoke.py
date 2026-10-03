"""Appearance protocol tests with mock clients, no BeamMP runtime."""
from pathlib import Path
from lupa import LuaRuntime
root = Path(__file__).resolve().parents[1]
lua = LuaRuntime(unpack_returned_tuples=True)
lua.globals().catalogue = lua.execute((root / 'mod/lua/ge/extensions/avatarSandbox/avatarCatalogue.lua').read_text())
lua.execute("""
local hash=string.rep('a',64)
local server=assert(catalogue.newServer({hash}))
local full=assert(catalogue.newClient({hash}))
local missing=assert(catalogue.newClient({}))
assert(full.apply(server.join(1)))
local selected=assert(server.select(1,hash))
assert(full.apply(selected)); assert(missing.apply(selected))
assert(full.get(1).resolved==hash and missing.get(1).resolved=='noob')
assert(not server.select(1,string.rep('b',64)))
assert(not server.select(2,hash))
assert(not full.apply(selected))
local late=assert(catalogue.newClient({hash}))
assert(late.apply(server.join(2))); assert(late.get(1).resolved==hash)
assert(full.apply(server.snapshot()))
assert(not full.apply({version=1,revision=99,assignments={{player=1,avatar=hash},{player=1,avatar=hash}}}))
assert(full.get(1).resolved==hash)
assert(full.apply(server.leave(1))); assert(not full.get(1))
local copy=full.get(2); copy.resolved=hash; assert(full.get(2).resolved=='noob')
full.reset(); assert(full.apply({version=1,revision=0,assignments={}}))
""")
print('Avatar approval, synchronization, fallback, stale data and cleanup passed')
