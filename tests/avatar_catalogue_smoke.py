"""Catalogue factories require a host Rust adapter; no duplicate Lua rules."""
from pathlib import Path
from lupa import LuaRuntime
lua=LuaRuntime(unpack_returned_tuples=True)
root=Path(__file__).resolve().parents[1]
lua.globals().catalogue=lua.execute((root/'mod/lua/ge/extensions/avatarSandbox/avatarCatalogue.lua').read_text())
lua.execute('''
assert(catalogue.newServer({})==nil)
assert(catalogue.newClient({})==nil)
local server={snapshot=function() end,join=function() end,select=function() end,leave=function() end}
local client={apply=function() end,get=function() end,reset=function() end}
local adapter={newServer=function(entries) assert(entries[1]=='pack'); return server end,
 newClient=function(entries) assert(entries[1]=='pack'); return client end}
assert(catalogue.newServer({'pack'},adapter)==server)
assert(catalogue.newClient({'pack'},adapter)==client)
assert(catalogue.newClient({}, {newClient=function() return {} end})==nil)
''')
print('Rust catalogue facade factories passed')
