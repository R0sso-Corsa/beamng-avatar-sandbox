-- Fixed-step host driver; attach a synchronous adapter before using controls.
local M = {}
local stepSeconds, maxSteps = 1 / 240, 16
local backend, accumulator, dropped = nil, 0, 0
local controls = {forward=0, backward=0, left=0, right=0, jump=0}
local heading, jumpPending, lastError = 0, false, nil
local function finite(n)
  return type(n) == 'number' and n == n and math.abs(n) < math.huge
end
local function clearInput()
  for key in pairs(controls) do controls[key] = 0 end
  jumpPending = false
end
function M.stop()
  backend, accumulator = nil, 0
  clearInput()
end
-- Adapter must commit one step atomically and return true; false/errors stop it.
function M.attach(step)
  if type(step) ~= 'function' then return false, 'Expected step callback' end
  M.stop()
  backend, dropped, heading, lastError = step, 0, 0, nil
  return true
end
function M.setControl(name, value)
  if controls[name] == nil or not finite(value) or value < 0 or value > 1 then
    return false, 'Invalid control'
  end
  if not backend then return false, 'No physics adapter' end
  if name == 'jump' and value > 0 and controls.jump == 0 then jumpPending = true end
  controls[name] = value
  return true
end
-- Horizontal radians: heading zero means forward +Y, right +X, Z-up.
function M.setHeading(value)
  if not finite(value) then return false, 'Invalid heading' end
  heading = value
  return true
end
function M.advance(dtSim)
  if not finite(dtSim) or dtSim < 0 then return false, 'Invalid simulation delta' end
  if not backend then return true, 0 end
  if dtSim == 0 then
    accumulator = 0
    clearInput()
    return true, 0
  end
  local total = accumulator + dtSim
  if not finite(total) then return false, 'Simulation delta overflow' end
  local due = math.floor(total / stepSeconds + 1e-9)
  local count = math.min(due, maxSteps)
  -- Drop excess whole steps deliberately; retain only the interpolation fraction.
  accumulator = math.max(0, total - due * stepSeconds)
  dropped = dropped + (due - count) * stepSeconds
  local x, y = controls.right - controls.left, controls.forward - controls.backward
  local length = math.max(1, math.sqrt(x*x + y*y))
  x, y = x / length, y / length
  local cosine, sine = math.cos(heading), math.sin(heading)
  for _ = 1, count do
    local input = {movement={cosine*x - sine*y, sine*x + cosine*y}, jump=jumpPending or controls.jump > 0}
    local ok, result = pcall(backend, input, stepSeconds)
    if not ok or result ~= true then
      lastError = ok and 'Adapter rejected step' or tostring(result)
      local t=type(extensions)=='table' and extensions.avatarSandbox_telemetry
      if t then t.record('error','driver',lastError) end
      M.stop()
      return false, lastError
    end
    jumpPending = false
  end
  return true, count
end
function M.status()
  return {attached=backend ~= nil, alpha=accumulator / stepSeconds,
    droppedSeconds=dropped, lastError=lastError, stepSeconds=stepSeconds}
end
function M.onUpdate(dtReal, dtSim) return M.advance(dtSim) end
function M.onClientEndMission() M.stop() end
function M.onExtensionUnloaded() M.stop() end
return M
