-- Host facade: all building-plan operations are performed by the Rust backend.
local M = {}
local enabled,backend=false,nil
function M.attachBackend(value)
  if enabled then return false,'Disable building tools first' end
  if type(value)~='table' then return false,'Rust building backend required' end
  for _,name in ipairs({'setEnabled','place','remove','move','resize','clone','clear'}) do
    if type(value[name])~='function' then return false,'Incomplete building backend' end
  end
  backend=value
  return true
end
function M.setEnabled(value)
  if type(value)~='boolean' then return false,'Expected boolean' end
  if value and not backend then return false,'Rust building backend required' end
  if backend then
    local ok,err=backend.setEnabled(value)
    if ok~=true then return false,err end
  end
  enabled=value;return true
end
function M.toggle() return M.setEnabled(not enabled) end
function M.status() return {enabled=enabled,backendAttached=backend~=nil,runtimeSpawningImplemented=false} end
local function call(name,...)
  if not enabled or not backend then return false,'Building tools disabled' end
  return backend[name](...)
end
function M.place(...) return call('place',...) end
function M.remove(...) return call('remove',...) end
function M.move(...) return call('move',...) end
function M.resize(...) return call('resize',...) end
function M.clone(...) return call('clone',...) end
function M.onClientEndMission()
  if backend then pcall(backend.clear) end
  enabled=false
end
function M.onExtensionUnloaded() M.onClientEndMission(); backend=nil end
return M
