-- Rust catalogue facade. Host supplies authenticated transport and conversions.
local M = {}
local function create(backend,factory,entries,methods)
  if type(backend)~='table' or type(backend[factory])~='function' then
    return nil,'No Rust catalogue adapter'
  end
  local instance,err=backend[factory](entries)
  if not instance then return nil,err end
  for _,name in ipairs(methods) do
    if type(instance[name])~='function' then return nil,'Missing adapter method: '..name end
  end
  return instance
end
-- Player IDs must originate in authenticated server connection context.
function M.newServer(approved,backend)
  return create(backend,'newServer',approved,{'snapshot','join','select','leave'})
end
-- Only authenticated server snapshots should reach apply().
function M.newClient(installed,backend)
  return create(backend,'newClient',installed,{'apply','get','reset'})
end
return M
