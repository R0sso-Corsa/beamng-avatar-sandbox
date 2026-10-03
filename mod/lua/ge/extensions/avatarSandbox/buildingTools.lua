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
-- F3X-inspired world-axis operations; one mod-owned part per call.
function M.move(id,offset,player,vehicles)
  local p,err=planner(); if not p then return false,err end
  local b=p.get(id); if not b or type(offset)~='table' then return false,'Invalid move' end
  for _,axis in ipairs({'x','y','z'}) do
    local n=offset[axis]
    if type(n)~='number' or n~=n or math.abs(n)==math.huge then return false,'Invalid offset' end
    b.min[axis]=b.min[axis]+n; b.max[axis]=b.max[axis]+n
  end
  return p.edit(id,b,player,vehicles)
end
-- Symmetric resize around the part centre; dimensions in metres.
function M.resize(id,size,player,vehicles)
  local p,err=planner(); if not p then return false,err end
  local b=p.get(id); if not b or type(size)~='table' then return false,'Invalid resize' end
  for _,axis in ipairs({'x','y','z'}) do
    local n=size[axis]
    if type(n)~='number' or n~=n or n<0.01 or n==math.huge then return false,'Invalid size' end
    local centre=(b.min[axis]+b.max[axis])/2
    b.min[axis]=centre-n/2; b.max[axis]=centre+n/2
  end
  return p.edit(id,b,player,vehicles)
end
function M.clone(id,offset,player,vehicles)
  local p,err=planner(); if not p then return nil,err end
  return p.clone(id,offset,player,vehicles)
end
function M.onClientEndMission() enabled=false end
function M.onExtensionUnloaded() enabled=false end
return M
