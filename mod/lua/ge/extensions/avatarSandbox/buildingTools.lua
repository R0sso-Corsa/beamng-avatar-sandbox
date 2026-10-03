-- Optional user-facing access to the offline placement planner.
local M = {}
local enabled = false
function M.setEnabled(value)
  if type(value) ~= 'boolean' then return false,'Expected a boolean' end
  enabled=value
  return true
end
function M.toggle() return M.setEnabled(not enabled) end
function M.status() return {enabled=enabled,runtimeSpawningImplemented=false} end
local function planner()
  if not enabled then return nil,'Building tools disabled' end
  local p=type(extensions)=='table' and extensions.avatarSandbox_placement
  if type(p)~='table' then return nil,'Load avatarSandbox_placement first' end
  return p
end
function M.place(centre,player,vehicles)
  local p,err=planner(); if not p then return nil,err end
  return p.place(centre,player,vehicles)
end
function M.remove(id)
  local p,err=planner(); if not p then return false,err end
  return p.remove(id)
end
function M.onClientEndMission() enabled=false end
function M.onExtensionUnloaded() enabled=false end
return M
