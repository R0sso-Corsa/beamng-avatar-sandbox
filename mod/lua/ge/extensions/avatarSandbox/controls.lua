-- BeamNG UI adapter. Commands are whitelisted; no arbitrary Lua execution.
local M = {}
local function ext(name) return type(extensions)=='table' and extensions['avatarSandbox_'..name] end
local function telemetry(level,message)
  local t=ext('telemetry'); if t then t.record(level,'controls',message) end
end
function M.refresh()
  local main,driver,building,t=ext('main'),ext('driver'),ext('buildingTools'),ext('telemetry')
  local data={mod=main and main.status() or {enabled=false,characterMode='beamng',avatarActive=false},
    driver=driver and driver.status() or {attached=false},
    building=building and building.status() or {enabled=false,backendAttached=false},
    telemetry=t and t.snapshot() or {errorCount=0,events={}}}
  if type(guihooks)=='table' and type(guihooks.trigger)=='function' then guihooks.trigger('AvatarSandboxStatus',data) end
  return data
end
function M.release()
  local d=ext('driver')
  if d then for _,key in ipairs({'forward','backward','left','right','jump'}) do d.setControl(key,0) end end
end
function M.command(name)
  if name=='refresh' then return M.refresh() end
  local main,building,d=ext('main'),ext('buildingTools'),ext('driver')
  local ok,result,err
  local actions={toggleMod=function() return main.toggle() end,
    roblox=function() return main.setCharacterMode('roblox') end,
    beamng=function() M.release(); return main.setCharacterMode('beamng') end,
    building=function() return building.toggle() end,
    interact=function() return main.interact() end,
    clearTelemetry=function() local t=ext('telemetry'); if t then t.clear(); return true end; return false,'Telemetry unavailable' end}
  if actions[name] then
    ok,result,err=pcall(actions[name])
  else
    local key,edge
    if type(name)=='string' then key,edge=name:match('^(%a+)([01])$') end
    local allowed={forward=true,backward=true,left=true,right=true,jump=true}
    if not key or not allowed[key] then telemetry('error','Unknown control command'); return false,'Unknown command' end
    if not d or not main or not main.status().avatarActive then result,err=false,'Avatar adapter inactive';ok=true
    else ok,result,err=pcall(d.setControl,key,tonumber(edge)) end
  end
  if not ok or result~=true then telemetry('error',not ok and tostring(result) or tostring(err or 'Command rejected'))
  else telemetry('info','Command '..name) end
  M.refresh()
  return ok and result==true,err
end
function M.onExtensionLoaded()
  if type(extensions)=='table' and type(extensions.load)=='function' then
    for _,name in ipairs({'telemetry','main','driver','buildingTools'}) do
      if not ext(name) then extensions.load('avatarSandbox_'..name) end
    end
  end
  telemetry('info','Controls loaded; telemetry is local-only')
end
function M.onClientEndMission() M.release() end
function M.onExtensionUnloaded() M.release() end
return M
