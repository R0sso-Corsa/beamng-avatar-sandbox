"""Local diagnostics and UI command routing, tested without BeamNG."""
from pathlib import Path
from lupa import LuaRuntime
root=Path(__file__).resolve().parents[1]
lua=LuaRuntime(unpack_returned_tuples=True)
def load(name):
    return lua.execute((root/f'mod/lua/ge/extensions/avatarSandbox/{name}.lua').read_text())
t=load('telemetry');main=load('main');driver=load('driver');building=load('buildingTools')
lua.globals().extensions=lua.table(avatarSandbox_telemetry=t,avatarSandbox_main=main,avatarSandbox_driver=driver,avatarSandbox_buildingTools=building)
controls=load('controls')
for i in range(120):
    assert t.record('error','test','x'*600)
snapshot=t.snapshot()
assert snapshot['eventCount']==100 and snapshot['errorCount']==120
snapshot['events'][1]['message']='changed'
assert len(t.snapshot()['events'][1]['message'])==512
assert controls.command('arbitrary_lua')[0] is False
assert controls.command('forward1')[0] is False
assert controls.command('toggleMod')[0] is True
main.attachCharacterAdapter(lua.eval('{enter=function() error("mock enter failure") end,leave=function() return true end}'))
assert controls.command('roblox')[0] is False
assert t.snapshot()['errorCount']>=123
assert controls.refresh()['mod']['characterMode']=='beamng'
t.clear();assert t.snapshot()['eventCount']==0
print('Telemetry bounds, copied reports, command whitelist and transition errors passed')
