-- Local-only bounded diagnostics. No networking, positions or user identifiers.
local M = {}
local events,sequence,errors={},0,0
function M.record(level,component,message)
  if level~='info' and level~='warning' and level~='error' then return false end
  if type(component)~='string' or type(message)~='string' then return false end
  sequence=sequence+1
  if level=='error' then errors=errors+1 end
  events[#events+1]={sequence=sequence,level=level,component=component:sub(1,64),message=message:sub(1,512)}
  if #events>100 then table.remove(events,1) end
  return true
end
function M.snapshot()
  local copy={}
  for _,e in ipairs(events) do copy[#copy+1]={sequence=e.sequence,level=e.level,component=e.component,message=e.message} end
  return {version=1,errorCount=errors,eventCount=#copy,lastSequence=sequence,events=copy}
end
function M.clear() events={}; errors=0 end
return M
