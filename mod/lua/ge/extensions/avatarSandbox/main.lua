-- GE Lua extension. Engine picking, avatar rendering and animations are pending.
local M = {}
local enabled = false
local characterMode = "beamng"
local modeAdapter = nil
local avatarActive = false
local targets = {}
local focus = nil
local reach = 3

local function finite(n)
  return type(n) == 'number' and n == n and math.abs(n) < math.huge
end

local function position(p)
  return type(p) == 'table' and finite(p.x) and finite(p.y) and finite(p.z)
end

local function report(message)
  if type(log) == 'function' then log('I', 'avatarSandbox', message) end
end

local function reset()
  focus = nil
  targets = {}
  characterMode = "beamng"
  avatarActive = false
end

-- Adapter enter/leave must atomically claim/restore controls, camera and visuals.
function M.attachCharacterAdapter(adapter)
  if avatarActive then return false,'Leave avatar mode before replacing adapter' end
  if type(adapter)~='table' or type(adapter.enter)~='function' or type(adapter.leave)~='function' then
    return false,'Expected enter/leave adapter'
  end
  modeAdapter=adapter
  return true
end
function M.setCharacterMode(mode)
  if mode~='beamng' and mode~='roblox' then return false,'Unknown character mode' end
  if mode=='roblox' and not enabled then return false,'Enable mod first' end
  local wantActive=mode=='roblox' and modeAdapter~=nil
  if wantActive~=avatarActive then
    local callback=wantActive and modeAdapter.enter or modeAdapter.leave
    local ok,result=pcall(callback)
    if not ok or result~=true then return false,'Character adapter transition failed' end
    avatarActive=wantActive
  end
  characterMode=mode
  if mode=='beamng' and type(extensions)=='table' and extensions.avatarSandbox_driver then
    extensions.avatarSandbox_driver.stop()
  end
  return true
end
function M.toggleCharacterMode()
  return M.setCharacterMode(characterMode=='beamng' and 'roblox' or 'beamng')
end

function M.setEnabled(value)
  if type(value) ~= 'boolean' then return false, 'Expected a boolean' end
  if not value then
    local ok,err=M.setCharacterMode('beamng')
    if not ok then return false,err end
  end
  enabled = value
  focus = nil
  report(enabled and 'Enabled (interaction foundation only)' or 'Disabled')
  return true
end

function M.toggle()
  return M.setEnabled(not enabled)
end

-- Only mod-owned targets are registered. No stock map objects are modified.
function M.registerTarget(id, label, callback)
  if type(id) ~= 'string' or id == '' or type(label) ~= 'string'
      or type(callback) ~= 'function' then
    return false, 'Expected target id, label and callback'
  end
  if targets[id] then return false, 'Target already registered' end
  targets[id] = {label = label, callback = callback}
  return true
end

function M.unregisterTarget(id)
  if type(id) ~= 'string' then return false, 'Expected a target id' end
  targets[id] = nil
  if focus and focus.id == id then focus = nil end
  return true
end

-- A future engine picker supplies a fresh visible hit each frame.
-- Positions are plain {x, y, z} tables in metres, not BeamNG vec3 userdata.
-- Distance is measured from the player, not the third-person camera.
function M.setTarget(id, playerPosition, hitPosition, visible)
  focus = nil
  if not enabled then return false, 'Sandbox is disabled' end
  if type(id) ~= 'string' or not targets[id] then return false, 'Unknown target' end
  if not position(playerPosition) or not position(hitPosition) then
    return false, 'Expected finite player and hit positions'
  end
  if visible ~= true then return false, 'Target is obstructed' end
  local dx = playerPosition.x - hitPosition.x
  local dy = playerPosition.y - hitPosition.y
  local dz = playerPosition.z - hitPosition.z
  if dx * dx + dy * dy + dz * dz > reach * reach then
    return false, 'Target is out of reach'
  end
  focus = {id = id}
  return true
end

function M.clearTarget()
  focus = nil
end

function M.interact()
  if not enabled then return false, 'Sandbox is disabled' end
  if not focus then return false, 'No current target' end
  local id = focus.id
  local target = targets[id]
  -- A hit is consumed once. Picker must refresh it before another interaction.
  focus = nil
  if not target then return false, 'Target was removed' end
  local ok, err = pcall(target.callback, id)
  if not ok then
    report('Interaction failed: ' .. tostring(err))
    return false, 'Interaction callback failed'
  end
  return true
end

function M.status()
  local count = 0
  for _ in pairs(targets) do count = count + 1 end
  return {
    enabled = enabled,
    characterMode = characterMode,
    avatarActive = avatarActive,
    characterAdapterAttached = modeAdapter~=nil,
    reachMetres = reach,
    targetCount = count,
    targetId = focus and focus.id or nil,
    targetLabel = focus and targets[focus.id].label or nil,
    enginePickingImplemented = false,
    avatarRenderingImplemented = false
  }
end

function M.onExtensionLoaded()
  enabled = false
  reset()
  report('Loaded. Native walking is unchanged; call toggle() to enable interactions.')
end

function M.onUpdate()
  -- Never keep a hit across frames: prevents interaction after walking away.
  -- Engine picker must call setTarget after this hook, once implemented.
  focus = nil
end

function M.onClientEndMission()
  M.setCharacterMode('beamng')
  enabled = false
  reset()
end

function M.onExtensionUnloaded()
  M.setCharacterMode('beamng')
  modeAdapter=nil
  enabled = false
  reset()
end

return M
