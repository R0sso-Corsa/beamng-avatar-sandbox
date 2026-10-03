"""Fixed-step/input regressions in generic Lua; requires lupa."""
from pathlib import Path
from lupa import LuaRuntime
root = Path(__file__).resolve().parents[1]
lua = LuaRuntime(unpack_returned_tuples=True)
lua.globals().driver = lua.execute((root / 'mod/lua/ge/extensions/avatarSandbox/driver.lua').read_text())
lua.execute("""
local steps, last = 0, nil
local function step(input, dt) steps=steps+1; last=input; assert(dt==1/240); return true end
assert(driver.attach(step))
assert(driver.setControl('forward',1)); assert(driver.setControl('right',1))
for i=1,60 do assert(driver.advance(1/60)) end
assert(steps==240 and math.abs(last.movement[1]^2+last.movement[2]^2-1)<1e-10)
assert(driver.advance(0)); assert(driver.advance(1/240)); assert(last.movement[1]==0)
assert(driver.setControl('jump',1)); assert(driver.setControl('jump',0))
assert(driver.advance(1/480)); assert(driver.advance(1/480)); assert(last.jump)
assert(driver.advance(1/240)); assert(not last.jump)
assert(driver.setControl('forward',1)); assert(driver.setHeading(math.pi/2))
assert(driver.advance(1/240)); assert(math.abs(last.movement[1]+1)<1e-10)
local before=steps; assert(driver.advance(1)); assert(steps-before==16)
assert(driver.status().droppedSeconds>0.9 and driver.status().alpha<1)
assert(not driver.advance(-1)); assert(not driver.setControl('forward',0/0))
assert(driver.attach(function() error('mock failure') end))
assert(not driver.advance(1/240)); assert(not driver.status().attached)
assert(driver.attach(step)); driver.onClientEndMission(); assert(not driver.status().attached)
""")
print('Driver timing, input, pause, backlog and failure checks passed')
