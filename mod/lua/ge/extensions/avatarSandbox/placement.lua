-- Compatibility facade only; Rust owns validation, plans and grid behavior.
local M = {}
local backend=nil
function M.attachBackend(value)
  if type(value)~='table' then return false,'Rust placement backend required' end
  for _,name in ipairs({'preview','place','remove','get','move','resize','clone','clear'}) do
    if type(value[name])~='function' then return false,'Incomplete placement backend' end
  end
  backend=value;return true
end
local function call(name,...)
  if not backend then return nil,'Rust placement backend required' end
  return backend[name](...)
end
function M.preview(...) return call('preview',...) end
function M.place(...) return call('place',...) end
function M.remove(...) return call('remove',...) end
function M.get(...) return call('get',...) end
function M.clear() if backend then return backend.clear() end end
function M.onClientEndMission() M.clear() end
function M.onExtensionUnloaded() M.clear();backend=nil end
return M
