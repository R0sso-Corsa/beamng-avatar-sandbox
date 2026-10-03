-- Pure placement plan: metre-sized cubes; no engine objects or collision created.
local M = {}
local blocks, nextId = {}, 0
local function finite(n)
  return type(n) == 'number' and n == n and math.abs(n) < math.huge
end
local function point(p)
  return type(p) == 'table' and finite(p.x) and finite(p.y) and finite(p.z)
    and math.abs(p.x) <= 1000000 and math.abs(p.y) <= 1000000 and math.abs(p.z) <= 1000000
end
local function bounds(b)
  return type(b) == 'table' and point(b.min) and point(b.max)
    and b.min.x <= b.max.x and b.min.y <= b.max.y and b.min.z <= b.max.z
end
local function copy(b)
  return {min={x=b.min.x,y=b.min.y,z=b.min.z}, max={x=b.max.x,y=b.max.y,z=b.max.z}}
end
local function overlaps(a,b)
  return a.min.x < b.max.x and a.max.x > b.min.x
    and a.min.y < b.max.y and a.max.y > b.min.y
    and a.min.z < b.max.z and a.max.z > b.min.z
end
-- Input is a desired cube centre supplied by a host picker, not a raw ray hit.
-- Player and vehicle bounds must be fresh world-space conservative AABBs.
function M.preview(desired, player, vehicles)
  if not point(desired) or not bounds(player) or type(vehicles) ~= 'table' then
    return nil, 'Expected centre, player bounds and vehicle bounds'
  end
  local lo={x=math.floor(desired.x),y=math.floor(desired.y),z=math.floor(desired.z)}
  local cube={min=lo,max={x=lo.x+1,y=lo.y+1,z=lo.z+1}}
  if overlaps(cube,player) then return nil,'Overlaps player' end
  for _, vehicle in pairs(vehicles) do
    if not bounds(vehicle) then return nil,'Invalid vehicle bounds' end
    if overlaps(cube,vehicle) then return nil,'Overlaps vehicle' end
  end
  for _, block in pairs(blocks) do
    if overlaps(cube,block) then return nil,'Overlaps placed block' end
  end
  return copy(cube)
end
-- Recompute validation at placement time; previews do not reserve a cell.
function M.place(desired, player, vehicles)
  local cube,err=M.preview(desired,player,vehicles)
  if not cube then return nil,err end
  nextId=nextId+1
  blocks[nextId]=copy(cube)
  return nextId,copy(cube)
end
-- Axis-aligned edits validate the complete replacement before committing.
function M.edit(id, replacement, player, vehicles)
  if not blocks[id] then return false,'Unknown block' end
  if not bounds(replacement) or not bounds(player) or type(vehicles)~='table' then
    return false,'Invalid bounds'
  end
  for _,axis in ipairs({'x','y','z'}) do
    if replacement.max[axis]-replacement.min[axis]<0.01 then return false,'Part too small' end
  end
  if overlaps(replacement,player) then return false,'Overlaps player' end
  for _,vehicle in pairs(vehicles) do
    if not bounds(vehicle) then return false,'Invalid vehicle bounds' end
    if overlaps(replacement,vehicle) then return false,'Overlaps vehicle' end
  end
  for other,block in pairs(blocks) do
    if other~=id and overlaps(replacement,block) then return false,'Overlaps placed block' end
  end
  blocks[id]=copy(replacement)
  return true
end
function M.clone(id, offset, player, vehicles)
  local original=blocks[id]
  if not original or not point(offset) then return nil,'Invalid clone' end
  local candidate=copy(original)
  for _,axis in ipairs({'x','y','z'}) do
    candidate.min[axis]=candidate.min[axis]+offset[axis]
    candidate.max[axis]=candidate.max[axis]+offset[axis]
  end
  -- Temporary record is removed on validation failure; IDs never recycle.
  nextId=nextId+1
  local cloneId=nextId
  blocks[cloneId]=copy(original)
  local ok,err=M.edit(cloneId,candidate,player,vehicles)
  if not ok then blocks[cloneId]=nil; return nil,err end
  return cloneId,copy(candidate)
end
function M.remove(id)
  if not finite(id) or id%1~=0 or not blocks[id] then return false,'Unknown block' end
  blocks[id]=nil
  return true
end
function M.get(id)
  if not blocks[id] then return nil end
  return copy(blocks[id])
end
function M.clear() blocks={} end
function M.status()
  local count=0
  for _ in pairs(blocks) do count=count+1 end
  return {count=count,gridMetres=1,runtimeSpawningImplemented=false}
end
function M.onClientEndMission() M.clear() end
function M.onExtensionUnloaded() M.clear() end
return M
