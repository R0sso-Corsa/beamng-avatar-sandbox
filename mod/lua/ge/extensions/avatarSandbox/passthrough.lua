-- Opt-in diagnostic bridge. No camera, vehicle, native input or rendering takeover.
local M = {}
local sock, config, render = nil, nil, nil
local sequence, clock, sendAt, received, lastReply = 0, 0, 0, -math.huge, 0
local enabled, active, input, colliders, pose = false, false, {0,0}, {}, nil
local jumping, lastError = false, nil
local function finite(n) return type(n)=='number' and n==n and math.abs(n)<math.huge end
local function vector(v)
  return type(v)=='table' and #v==3 and finite(v[1]) and finite(v[2]) and finite(v[3])
end
local function packet(wantEnabled)
  sequence=sequence+1
  return jsonEncode({version=1,token=config.token,session=config.session,sequence=sequence,kind='host',
    payload={enabled=wantEnabled,movement=input,jump=jumping,colliders=colliders}})
end
function M.stop()
  if sock then
    input={0,0}; jumping=false
    pcall(function() sock:send(packet(false)) end)
    sock:close()
  end
  sock=nil; config=nil; pose=nil; active=false; enabled=false; input={0,0}; jumping=false; colliders={}
  return true
end
function M.start(session)
  if sock then return false,'Stop bridge before starting a new session' end
  if type(session)~='table' or type(session.token)~='string' or #session.token<32
      or type(session.session)~='string' or session.session=='' or not finite(session.udpPort)
      or session.udpPort%1~=0 or session.udpPort<1 or session.udpPort>65535 then
    return false,'Invalid private session config'
  end
  if type(jsonEncode)~='function' or type(jsonDecode)~='function' then return false,'JSON API unavailable' end
  local ok,socket=pcall(require,'socket')
  if not ok or type(socket.udp)~='function' then return false,'LuaSocket UDP unavailable on this build' end
  local udp,err=socket.udp()
  if not udp then return false,tostring(err) end
  udp:settimeout(0)
  local connected,why=udp:setpeername('127.0.0.1',session.udpPort)
  if not connected then udp:close(); return false,tostring(why) end
  sock=udp; config={token=session.token,session=session.session,udpPort=session.udpPort}
  sequence=0; clock=0; sendAt=0; received=-math.huge; lastReply=0; lastError=nil
  return true
end
function M.setEnabled(value)
  if type(value)~='boolean' or not sock then return false,'Start bridge first; expected boolean' end
  enabled=value
  if not value then input={0,0}; jumping=false; pose=nil; active=false end
  return true
end
function M.setInput(x,y,jump)
  if not sock or not enabled or not finite(x) or not finite(y) or type(jump)~='boolean' then return false end
  local length=math.sqrt(x*x+y*y)
  if not finite(length) then return false end
  if length>1 then x=x/length; y=y/length end
  input={x,y}; jumping=jump; return true
end
function M.setColliders(boxes)
  if not sock or type(boxes)~='table' or #boxes>32 then return false end
  local copy,ids={},{}
  for _,b in ipairs(boxes) do
    if type(b)~='table' or not finite(b.id) or b.id%1~=0 or b.id<1 or b.id>4294967295
        or ids[b.id] or not vector(b.position) or not vector(b.size) then return false end
    for i=1,3 do
      if math.abs(b.position[i])>1e6 or b.size[i]<=0 or b.size[i]>1000 then return false end
    end
    ids[b.id]=true
    copy[#copy+1]={id=b.id,position={b.position[1],b.position[2],b.position[3]},size={b.size[1],b.size[2],b.size[3]}}
  end
  colliders=copy; return true
end
function M.attachRenderer(callback)
  if callback~=nil and type(callback)~='function' then return false end
  render=callback; return true
end
function M.onUpdate(dtReal,dtSim)
  if not sock then return end
  if not finite(dtReal) or dtReal<0 or not finite(dtSim) or dtSim<0 then M.stop(); return end
  clock=clock+dtReal
  local run=enabled and dtSim>0
  if not run then input={0,0}; jumping=false; pose=nil; active=false end
  if clock>=sendAt then
    sendAt=clock+0.05
    local ok,bytes=pcall(packet,run)
    if not ok or #bytes>8192 then lastError='Snapshot encode failed or exceeded 8 KiB'; M.stop(); return end
    local sent,err=sock:send(bytes)
    if not sent then lastError=tostring(err) end
  end
  for _=1,8 do -- Bounded receive work on the game thread.
    local bytes,err=sock:receive(8193)
    if not bytes then
      if err~='timeout' then lastError=tostring(err) end
      break
    end
    if #bytes<=8192 then
      local ok,reply=pcall(jsonDecode,bytes)
      if ok and type(reply)=='table' and reply.ok==true and reply.version==1 and reply.session==config.session
          and finite(reply.sequence) and reply.sequence%1==0 and reply.sequence>lastReply and reply.sequence<=sequence then
        lastReply=reply.sequence; received=clock; active=run and reply.active==true
        pose=active and reply.payload or nil
      elseif ok and type(reply)=='table' and reply.ok==false then lastError=tostring(reply.error) end
    end
  end
  if clock-received>=1 then active=false; pose=nil end
  if active and pose and render then
    local ok=pcall(render,pose)
    if not ok then lastError='Renderer callback failed'; M.stop() end
  end
end
function M.getPose() return active and pose or nil end
function M.status()
  return {connected=sock~=nil,enabled=enabled,active=active,sequence=sequence,
    lastReply=lastReply,lastError=lastError,renderingImplemented=render~=nil,
    engineIntegrationVerified=false}
end
function M.onClientEndMission() M.stop() end
function M.onExtensionUnloaded() M.stop(); render=nil end
return M
