-- Transport-independent appearance protocol. The host owns authentication/rendering.
local M = {}
local function id(value)
  return type(value)=='string' and #value==64 and value:match('^[0-9a-f]+$')~=nil
end
local function player(value)
  return type(value)=='number' and value%1==0 and value>=0 and value<=2147483647
end
local function catalogue(entries)
  if type(entries)~='table' then return nil end
  local result={noob=true}
  local count=0
  for _,value in pairs(entries) do
    if not id(value) then return nil end
    result[value]=true; count=count+1
    if count>256 then return nil end
  end
  return result
end
function M.newServer(approved)
  local allowed=catalogue(approved)
  if not allowed then return nil,'Invalid catalogue' end
  local active,revision={},0
  local server={}
  function server.snapshot()
    local assignments={}
    for p,a in pairs(active) do assignments[#assignments+1]={player=p,avatar=a} end
    table.sort(assignments,function(a,b) return a.player<b.player end)
    return {version=1,revision=revision,assignments=assignments}
  end
  -- authenticatedPlayer comes from server connection context, never payload.
  function server.join(authenticatedPlayer)
    if not player(authenticatedPlayer) then return nil,'Invalid player' end
    if active[authenticatedPlayer] then return nil,'Player already joined' end
    local count=0
    for _ in pairs(active) do count=count+1 end
    if count>=256 then return nil,'Too many players' end
    active[authenticatedPlayer]='noob'; revision=revision+1
    return server.snapshot()
  end
  function server.select(authenticatedPlayer,avatar)
    if not player(authenticatedPlayer) or not active[authenticatedPlayer] then return nil,'Unknown player' end
    if type(avatar)~='string' or not allowed[avatar] then return nil,'Avatar not approved' end
    if active[authenticatedPlayer]~=avatar then active[authenticatedPlayer]=avatar; revision=revision+1 end
    return server.snapshot()
  end
  function server.leave(authenticatedPlayer)
    if not player(authenticatedPlayer) or not active[authenticatedPlayer] then return nil,'Unknown player' end
    active[authenticatedPlayer]=nil; revision=revision+1
    return server.snapshot()
  end
  return server
end
function M.newClient(installed)
  local available=catalogue(installed)
  if not available then return nil,'Invalid local catalogue' end
  local active,revision={},-1
  local client={}
  -- Apply full snapshots only from the authenticated server transport.
  function client.apply(snapshot)
    if type(snapshot)~='table' or snapshot.version~=1 or type(snapshot.revision)~='number'
      or snapshot.revision%1~=0 or snapshot.revision<0 or snapshot.revision>9007199254740991
      or type(snapshot.assignments)~='table' then return false,'Invalid snapshot' end
    if snapshot.revision<=revision then return false,'Stale snapshot' end
    local candidate,count={},0
    for _,entry in pairs(snapshot.assignments) do
      if type(entry)~='table' or not player(entry.player) or candidate[entry.player]
        or (entry.avatar~='noob' and not id(entry.avatar)) then return false,'Invalid assignment' end
      candidate[entry.player]={requested=entry.avatar,resolved=available[entry.avatar] and entry.avatar or 'noob'}
      count=count+1
      if count>256 then return false,'Too many players' end
    end
    active,revision=candidate,snapshot.revision
    return true
  end
  function client.get(p)
    local entry=active[p]
    if not entry then return nil end
    return {requested=entry.requested,resolved=entry.resolved}
  end
  -- Reset on disconnect/reconnect so a new server's revision zero is accepted.
  function client.reset() active,revision={},-1 end
  return client
end
return M
