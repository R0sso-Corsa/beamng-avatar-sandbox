-- Rust-backed scheduling facade. Adapter executes batches against host geometry.
local M = {}
local backend, lastError
local methods={'reset','setControl','setHeading','advance','status'}
local function call(name,...)
  if not backend then return false,'No Rust driver adapter' end
  local ok,a,b=pcall(backend[name],...)
  if ok and a~=false then return a,b end
  lastError=ok and tostring(b or 'Adapter rejected operation') or tostring(a)
  local t=type(extensions)=='table' and extensions.avatarSandbox_telemetry
  if t then t.record('error','driver',lastError) end
  M.stop()
  return false,lastError
end
function M.stop()
  local old=backend; backend=nil
  if old then pcall(old.reset) end
end
function M.attach(adapter)
  if type(adapter)~='table' then return false,'Expected Rust driver adapter' end
  for _,name in ipairs(methods) do
    if type(adapter[name])~='function' then return false,'Missing adapter method: '..name end
  end
  M.stop(); backend=adapter; lastError=nil
  return call('reset')
end
function M.setControl(name,value) return call('setControl',name,value) end
function M.setHeading(value) return call('setHeading',value) end
function M.advance(dt)
  if not backend then return true,0 end
  return call('advance',dt)
end
function M.status()
  if not backend then return {attached=false,lastError=lastError} end
  local state=call('status')
  if type(state)~='table' then return {attached=false,lastError=lastError} end
  state.attached=true; state.lastError=lastError
  return state
end
function M.onUpdate(dtReal,dtSim) return M.advance(dtSim) end
function M.onClientEndMission() M.stop() end
function M.onExtensionUnloaded() M.stop() end
return M
